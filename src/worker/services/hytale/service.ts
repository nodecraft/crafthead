import * as hytaleApi from './api';
import { EMPTY, HYTALE_DEFAULT_SKIN } from '../../data';
import { IdentityKind, RequestedKind, TextureKind } from '../../request';
import {
	fromHex,
	javaHashCode,
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
 * Also returns the profile if it was fetched during username lookup (to avoid double lookups).
 */
async function normalizeRequest(incomingRequest: Request, request: CraftheadRequest): Promise<NormalizedRequest> {
	if (request.identityType === IdentityKind.Uuid || request.identityType === IdentityKind.TextureID) {
		return { request };
	}

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

export async function retrieveSkin(incomingRequest: Request, request: CraftheadRequest): Promise<Response> {
	const { request: normalized, profile } = await normalizeRequest(incomingRequest, request);
	const skin = await retrieveTextureDirect(incomingRequest, normalized, TextureKind.SKIN, profile);
	if (skin.texture.status === 404) {
		// Return default Hytale skin for invalid profiles
		return new Response(HYTALE_DEFAULT_SKIN, {
			headers: {
				'X-Crafthead-Profile-Cache-Hit': 'invalid-profile',
			},
		});
	}
	if ([RequestedKind.Skin, RequestedKind.Body, RequestedKind.Bust].includes(normalized.requested)) {
		skin.texture.headers.set('X-Crafthead-Skin-Model', request.model || skin.model || 'default');
	}

	return skin.texture;
}

export async function retrieveCape(incomingRequest: Request, request: CraftheadRequest): Promise<Response> {
	const { request: normalized, profile } = await normalizeRequest(incomingRequest, request);
	const cape = await retrieveTextureDirect(incomingRequest, normalized, TextureKind.CAPE, profile);
	if (cape.texture.status === 404) {
		return new Response(EMPTY, {
			status: 404,
			headers: {
				'X-Crafthead-Profile-Cache-Hit': 'invalid-profile',
			},
		});
	}
	return cape.texture;
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
