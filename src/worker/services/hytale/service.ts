import { env } from 'cloudflare:workers';

import * as hytaleApi from './api';
import { loadHytaleAssets } from './assets';
import { getRequiredAssetPaths, resolveSkin } from './cosmetic-registry';
import { render_hytale_3d, render_text_avatar } from '../../../../pkg/mcavatar';
import { EMPTY, HYTALE_DEFAULT_SKIN } from '../../data';
import { IdentityKind, RequestedKind, TextureKind } from '../../request';
import { readAssetFile } from '../../util/files';
import {
	fromHex,
	offlinePlayerUuid,
	toHex,
	uuidVersion,
} from '../../util/uuid';

import type { HytaleProfile, HytaleProfileProperty } from './api';
import type { CraftheadRequest } from '../../request';
import type { CacheComputeResult } from '../../util/cache-helper';

interface TextureResponse {
	texture: Response;
	model?: string;
	textureId?: string;
}

interface HytaleTextureData {
	SKIN: {
		url: string;
		metadata?: {
			model?: string;
		};
	};
	CAPE?: {
		url: string;
	};
}

interface HytaleTexturePropertyValue {
	textures: HytaleTextureData;
}

// TODO: Update this when Hytale texture server URL is known
const HYTALE_TEXTURE_BASE_URL = 'https://textures.hytale.com/texture';

function extractDataFromTexturesProperty(property: HytaleProfileProperty | undefined): HytaleTextureData | undefined {
	if (property === undefined) {
		return undefined;
	}

	const rawJson = atob(property.value);
	const decoded: HytaleTexturePropertyValue = JSON.parse(rawJson);

	return decoded.textures;
}

async function fetchTextureFromUrl(textureUrl: string): Promise<TextureResponse> {
	const textureResponse = await fetch(textureUrl, {
		cf: {
			cacheEverything: true,
			cacheTtl: 86400,
		},
		headers: {
			'User-Agent': 'Crafthead (+https://crafthead.net)',
		},
		signal: AbortSignal.timeout(5000),
	});
	if (!textureResponse.ok) {
		throw new Error(`Unable to retrieve texture from Hytale, http status ${textureResponse.status}`);
	}

	return { texture: textureResponse, model: 'default' };
}

async function fetchTextureFromId(id: string): Promise<TextureResponse> {
	const url = `${HYTALE_TEXTURE_BASE_URL}/${id}`;
	return fetchTextureFromUrl(url);
}

async function fetchTextureFromProfile(profile: HytaleProfile, type: TextureKind): Promise<TextureResponse | undefined> {
	if (profile.properties) {
		const texturesData = extractDataFromTexturesProperty(
			profile.properties.find(property => property.name === 'textures'),
		);
		const textureUrl = type === TextureKind.CAPE ? texturesData?.CAPE?.url : texturesData?.SKIN.url;

		if (textureUrl) {
			const textureResponse = await fetch(textureUrl, {
				cf: {
					cacheEverything: true,
					cacheTtl: 86400,
				},
				headers: {
					'User-Agent': 'Crafthead (+https://crafthead.net)',
				},
				signal: AbortSignal.timeout(5000),
			});
			if (!textureResponse.ok) {
				throw new Error(`Unable to retrieve texture from Hytale, http status ${textureResponse.status}`);
			}

			return {
				texture: textureResponse,
				model: texturesData?.SKIN?.metadata?.model,
				textureId: textureUrl.split('/').pop(),
			};
		}
	}

	return undefined;
}

async function constructTextureResponse(textureResponse: TextureResponse, request: CraftheadRequest, source?: string): Promise<Response> {
	const buff = await textureResponse.texture.arrayBuffer();
	if (buff && buff.byteLength > 0) {
		const headers = new Headers();
		headers.set('X-Crafthead-Profile-Cache-Hit', source || 'miss');
		if (textureResponse.textureId) {
			headers.set('X-Crafthead-Texture-ID', textureResponse.textureId);
		}
		return new Response(buff, {
			status: 200,
			headers,
		});
	}
	return new Response(HYTALE_DEFAULT_SKIN, {
		status: 404,
		headers: {
			'X-Crafthead-Profile-Cache-Hit': 'not-found',
			'X-Crafthead-Skin-Model': 'default',
		},
	});
}

interface NormalizedRequest {
	request: CraftheadRequest;
	profile?: HytaleProfile;
}

/**
 * Normalizes the incoming request, such that we only work with UUIDs.
 * Always fetches the profile to get the username (needed for text avatars and future skin support).
 */
async function normalizeRequest(incomingRequest: Request, request: CraftheadRequest): Promise<NormalizedRequest> {
	if (request.identityType === IdentityKind.TextureID) {
		return { request };
	}

	if (request.identityType === IdentityKind.Uuid) {
		// UUID provided - fetch profile to get username
		const lookup = await hytaleApi.fetchProfile(incomingRequest, request.identity);
		if (lookup.result) {
			return { request, profile: lookup.result };
		}
		return { request };
	}

	// Username provided - look up to get UUID and profile
	const normalized: CraftheadRequest = { ...request, identityType: IdentityKind.Uuid };

	const profile = await hytaleApi.lookupUsername(incomingRequest, request.identity);
	if (profile) {
		normalized.identity = profile.id;
		return { request: normalized, profile };
	}
	// The lookup failed - use offline mode UUID
	normalized.identity = toHex(await offlinePlayerUuid(request.identity));
	return { request: normalized };
}

/**
 * Fetches a texture directly from the Hytale servers. Assumes the request has been normalized already.
 * @param prefetchedProfile - If provided, skips the profile fetch (avoids double lookup for username requests)
 */
async function retrieveTextureDirect(
	incomingRequest: Request,
	request: CraftheadRequest,
	kind: TextureKind,
	prefetchedProfile?: HytaleProfile,
): Promise<TextureResponse> {
	if (request.identityType === IdentityKind.TextureID) {
		const textureResponse = await fetchTextureFromId(request.identity);
		return {
			texture: await constructTextureResponse(textureResponse, request),
		};
	}
	const rawUuid = fromHex(request.identity);
	if (uuidVersion(rawUuid) === 4) {
		let profile: HytaleProfile | null;
		let source: string;
		if (prefetchedProfile) {
			profile = prefetchedProfile;
			source = 'hit';
		} else {
			const lookup = await hytaleApi.fetchProfile(incomingRequest, request.identity);
			profile = lookup.result;
			source = lookup.source;
		}

		if (profile) {
			const textureResponse = await fetchTextureFromProfile(profile, kind);
			if (textureResponse) {
				return {
					texture: await constructTextureResponse(textureResponse, request, source),
					model: textureResponse.model,
				};
			}
			return {
				texture: new Response(HYTALE_DEFAULT_SKIN, {
					status: 404,
					headers: {
						'X-Crafthead-Profile-Cache-Hit': 'not-found',
					},
				}),
			};
		}
		return {
			texture: new Response(HYTALE_DEFAULT_SKIN, {
				status: 404,
				headers: {
					'X-Crafthead-Profile-Cache-Hit': 'not-found',
				},
			}),
		};
	}

	return {
		texture: new Response(HYTALE_DEFAULT_SKIN, {
			status: 404,
			headers: {
				'X-Crafthead-Profile-Cache-Hit': 'offline-mode',
			},
		}),
	};
}

/**
 * Maps Crafthead RequestedKind to HytaleSkinRenderer view type
 */
function mapRequestedKindToViewType(kind: RequestedKind): string {
	switch (kind) {
		case RequestedKind.Avatar:
		case RequestedKind.Helm: {
			return 'avatar';
		}
		case RequestedKind.Cube: {
			return 'cube';
		}
		case RequestedKind.Body: {
			return 'body';
		}
		case RequestedKind.Bust: {
			return 'bust';
		}
		default: {
			return 'avatar';
		}
	}
}

/**
 * Renders a Hytale avatar using the 3D renderer.
 * Falls back to text avatar if 3D rendering fails.
 */
export async function renderAvatar(incomingRequest: Request, request: CraftheadRequest): Promise<Response> {
	const { profile } = await normalizeRequest(incomingRequest, request);
	const username = profile?.name ?? request.identity;

	try {
		// Load bundled Hytale assets (base model and animation)
		const assets = await loadHytaleAssets();



		// ... (inside renderAvatar)

		let resolvedSkin: ReturnType<typeof resolveSkin> | undefined;
		const assetPaths: string[] = [];
		const assetBytes: Uint8Array[] = [];

		if (profile?.skin) {
			resolvedSkin = resolveSkin(profile.skin);
			const requiredAssets = getRequiredAssetPaths(resolvedSkin);

			console.log(`Player ${username} skin resolved:`, {
				cosmeticsCount: resolvedSkin.cosmetics.length,
				requiredAssets: {
					models: requiredAssets.models.length,
					textures: requiredAssets.textures.length,
					gradients: requiredAssets.gradients.length,
				},
			});

			const assetSet = new Set<string>([
				...requiredAssets.models,
				...requiredAssets.textures,
				...requiredAssets.gradients,
				'Cosmetics/CharacterCreator/HaircutFallbacks.json',
				'Cosmetics/CharacterCreator/Faces.json',
				'Cosmetics/CharacterCreator/Eyes.json',
				'Cosmetics/CharacterCreator/Eyebrows.json',
				'Cosmetics/CharacterCreator/Mouths.json',
				'Cosmetics/CharacterCreator/Ears.json',
				'Cosmetics/CharacterCreator/Haircuts.json',
				'Cosmetics/CharacterCreator/FacialHair.json',
				'Cosmetics/CharacterCreator/Underwear.json',
				'Cosmetics/CharacterCreator/FaceAccessory.json',
				'Cosmetics/CharacterCreator/Capes.json',
				'Cosmetics/CharacterCreator/EarAccessory.json',
				'Cosmetics/CharacterCreator/Gloves.json',
				'Cosmetics/CharacterCreator/HeadAccessory.json',
				'Cosmetics/CharacterCreator/GradientSets.json',
				'Cosmetics/CharacterCreator/Overpants.json',
				'Cosmetics/CharacterCreator/Overtops.json',
				'Cosmetics/CharacterCreator/Pants.json',
				'Cosmetics/CharacterCreator/Shoes.json',
				'Cosmetics/CharacterCreator/SkinFeatures.json',
				'Cosmetics/CharacterCreator/Undertops.json',
			]);

			for (const assetPath of assetSet) {
				const data = await readAssetFile(assetPath, env);
				const providerPath = assetPath.startsWith('Common/')
					? `assets/${assetPath}`
					: `assets/Common/${assetPath}`;
				assetPaths.push(providerPath);
				assetBytes.push(new Uint8Array(data));
			}
		} else {
			console.log(`Player ${username} has no skin configuration`);
		}

		const viewType = mapRequestedKindToViewType(request.requested);
		const defaultSkin = {
			bodyCharacteristic: 'Default.10',
			underwear: null,
			face: null,
			ears: null,
			mouth: null,
			haircut: null,
			facialHair: null,
			eyebrows: null,
			eyes: null,
			pants: null,
			overpants: null,
			undertop: null,
			overtop: null,
			shoes: null,
			headAccessory: null,
			faceAccessory: null,
			earAccessory: null,
			skinFeature: null,
			gloves: null,
			cape: null,
		};
		const skinConfigJson = JSON.stringify({ skin: profile?.skin ?? defaultSkin });

		const imageData = render_hytale_3d(
			assets.modelJson,
			assets.animationJson,
			assets.textureBytes,
			skinConfigJson,
			assetPaths,
			assetBytes,
			viewType,
			request.size,
		);

		return new Response(imageData, {
			headers: {
				'Content-Type': 'image/png',
				'X-Crafthead-Renderer': 'hytale-3d',
				'X-Crafthead-Has-Skin': profile?.skin ? 'true' : 'false',
			},
		});
	} catch (error) {
		// Fall back to text avatar on error
		console.error('Hytale 3D rendering failed:', error);

		const imageData = render_text_avatar(username, request.size);
		return new Response(imageData, {
			headers: {
				'Content-Type': 'image/png',
				'X-Crafthead-Renderer': 'text-avatar-fallback',
			},
		});
	}
}

/**
 * TEMPORARY: Returns a text avatar since real Hytale skins aren't implemented yet.
 */
export async function retrieveSkin(incomingRequest: Request, request: CraftheadRequest): Promise<Response> {
	return renderAvatar(incomingRequest, request);
}

/**
 * Hytale capes are not supported yet.
 */
export function retrieveCape(_incomingRequest: Request, _request: CraftheadRequest): Response {
	return new Response(EMPTY, {
		status: 404,
		headers: {
			'X-Crafthead-Profile-Cache-Hit': 'not-supported',
		},
	});
}

export async function fetchProfile(incomingRequest: Request, request: CraftheadRequest): Promise<CacheComputeResult<HytaleProfile | null>> {
	const { request: normalized, profile } = await normalizeRequest(incomingRequest, request);
	if (!normalized.identity || uuidVersion(fromHex(normalized.identity)) === 3) {
		return {
			result: null,
			source: 'hytale',
		};
	}
	if (profile) {
		return {
			result: profile,
			source: 'hit',
		};
	}
	return hytaleApi.fetchProfile(incomingRequest, normalized.identity);
}
