import { env } from 'cloudflare:workers';

import * as hytaleApi from './api';
import { loadHytaleAssets } from './assets';
import { getRequiredAssetPaths, resolveSkin } from './cosmetic-registry';
import { render_hytale_3d, render_text_avatar } from '../../../../pkg/mcavatar';
import { EMPTY } from '../../data';
import { IdentityKind, RequestedKind } from '../../request';
import { readAssetFile } from '../../util/files';
import {
	fromHex,
	offlinePlayerUuid,
	toHex,
	uuidVersion,
} from '../../util/uuid';

import type { HytaleProfile } from './api';
import type { CraftheadRequest } from '../../request';
import type { CacheComputeResult } from '../../util/cache-helper';

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
		case RequestedKind.Skin: {
			// No support!
			return 'no-op';
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
		if (viewType === 'no-op') {
			return new Response(EMPTY, {
				status: 404,
				headers: {
					'X-Crafthead-Profile-Cache-Hit': 'not-supported',
				},
			});
		}
		// TODO: Replace this with a deterministic skin generator
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
		// TODO: Add Sentry eventually to track errors better

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
