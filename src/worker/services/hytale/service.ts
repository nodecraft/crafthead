import { createHash } from 'node:crypto';

import pLimit from 'p-limit';

import * as hytaleApi from './api';
import { loadHytaleAssets } from './assets';
import {
	type SkinDefinition,
	getCosmeticJson,
	getCosmeticJsonBytes,
	getRequiredAssetPaths,
	resolveSkin,
} from './cosmetic-registry';
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

import type { HytaleProfile, HytaleSkin } from './api';
import type { CraftheadRequest } from '../../request';
import type { CacheComputeResult } from '../../util/cache-helper';

const CACHE_TTL_MS = 24 * 60 * 60 * 1000; // 24 hours in milliseconds

function getRenderCacheKey(skin: HytaleSkin, request: CraftheadRequest): string {
	// Hash the skin to generate a unique key per unique skin combo (so we can share renders with multiple users)
	const hash = createHash('sha256').update(JSON.stringify(skin)).digest('hex');
	return `renders/${request.requested}/${request.size}/${request.armored ? 'armor' : 'regular'}/${hash}`;
}

async function getCachedRender(cacheKey: string, env: Cloudflare.Env): Promise<Response | null> {
	if (!env.HYTALE_RENDERS_CACHE) {
		return null;
	}

	try {
		const cachedObject = await env.HYTALE_RENDERS_CACHE.get(cacheKey);
		if (!cachedObject) {
			return null;
		}

		const cachedAt = cachedObject.customMetadata?.cachedAt;
		if (cachedAt) {
			const cacheAge = Date.now() - new Date(cachedAt).getTime();
			if (cacheAge > CACHE_TTL_MS) {
				await env.HYTALE_RENDERS_CACHE.delete(cacheKey);
				return null;
			}
		}

		const cachedData = await cachedObject.arrayBuffer();
		return new Response(cachedData, {
			headers: {
				'Content-Type': 'image/png',
				'X-Crafthead-R2-Cache-Hit': 'yes',
				'X-Crafthead-R2-Cache-Age-Ms': cachedAt ? String(Date.now() - new Date(cachedAt).getTime()) : 'unknown',
			},
		});
	} catch (err) {
		console.error('Error retrieving from R2 cache:', err);
		return null;
	}
}

async function storeCachedRender(cacheKey: string, imageData: Uint8Array, env: Cloudflare.Env): Promise<void> {
	if (!env.HYTALE_RENDERS_CACHE) {
		return;
	}

	try {
		await env.HYTALE_RENDERS_CACHE.put(cacheKey, imageData, {
			httpMetadata: {
				contentType: 'image/png',
			},
			customMetadata: {
				cachedAt: new Date().toISOString(),
			},
		});
	} catch (err) {
		console.error('Error storing to R2 cache:', err);
	}
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
			console.log(`Unknown requested kind: ${kind}`);
			// Then just fallback to avatar. Hytale's skin system is pretty different.
			return 'avatar';
		}
	}
}

function generateAndReturnTextAvatar(username: string, request: CraftheadRequest) {
	const imageData = render_text_avatar(username, request.size);
	return new Response(imageData, {
		headers: {
			'Content-Type': 'image/png',
			'X-Crafthead-Renderer': 'text-avatar-fallback',
		},
	});
}

/**
 * Renders a Hytale avatar using the 3D renderer.
 * Falls back to text avatar if 3D rendering fails.
 * Uses R2 caching for 24 hours to reduce computational cost.
 */
export async function renderAvatar(incomingRequest: Request, request: CraftheadRequest, env: Cloudflare.Env, ctx: ExecutionContext): Promise<Response> {
	const { profile } = await normalizeRequest(incomingRequest, request);
	const username = profile?.name ?? request.identity;
	if (!profile?.skin) {
		// TODO: Replace with a deterministic skin generator
		return generateAndReturnTextAvatar(username, request);
	}
	const cacheKey = getRenderCacheKey(profile.skin, request);

	const cachedRender = await getCachedRender(cacheKey, env);
	if (cachedRender) {
		return cachedRender;
	}

	try {
		// Load bundled Hytale assets (base model and animation)
		const skinDefinitionJson = getCosmeticJson<SkinDefinition[]>('Cosmetics/CharacterCreator/BodyCharacteristics.json');
		// Look for their skin type
		const skinType = profile.skin.bodyCharacteristic.split('.')[0] ?? 'Default';
		const skinPath = skinDefinitionJson.find(skin => skin.Id === skinType)?.GreyscaleTexture ?? 'Common/Characters/Player_Textures/Player_Greyscale.png';
		const assets = await loadHytaleAssets(skinPath, ctx);

		const resolvedSkin = await resolveSkin(profile.skin);
		const assetPaths: string[] = [];
		const assetBytes: Uint8Array[] = [];

		const requiredAssets = getRequiredAssetPaths(resolvedSkin);

		const skinSpecificAssetSet = new Set<string>([
			...requiredAssets.models,
			...requiredAssets.textures,
			...requiredAssets.gradients,
		]);

		for (const { path, bytes } of getCosmeticJsonBytes()) {
			assetPaths.push(`assets/Common/${path}`);
			assetBytes.push(bytes);
		}

		const limit = pLimit(5);
		const skinAssetPromises = [...skinSpecificAssetSet].map(async (assetPath) => {
			const data = await limit(() => readAssetFile(assetPath, env, ctx));
			const providerPath = assetPath.startsWith('Common/')
				? `assets/${assetPath}`
				: `assets/Common/${assetPath}`;
			return { providerPath, bytes: new Uint8Array(data) };
		});

		const results = await Promise.all(skinAssetPromises);
		for (const result of results) {
			assetPaths.push(result.providerPath);
			assetBytes.push(result.bytes);
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

		ctx.waitUntil(storeCachedRender(cacheKey, imageData, env));

		return new Response(imageData, {
			headers: {
				'Content-Type': 'image/png',
				'X-Crafthead-Renderer': 'hytale-3d',
				'X-Crafthead-Has-Skin': profile?.skin ? 'true' : 'false',
				'X-Crafthead-R2-Cache-Hit': 'no',
			},
		});
	} catch (error) {
		// Fall back to text avatar on error
		console.error('Hytale 3D rendering failed:', error);
		// TODO: Add Sentry eventually to track errors better

		return generateAndReturnTextAvatar(username, request);
	}
}

export async function retrieveSkin(/*incomingRequest: Request, request: CraftheadRequest*/): Promise<Response> {
	// TODO: Return something other than a 404
	return new Response(EMPTY, {
		status: 404,
		headers: {
			'X-Crafthead-Profile-Cache-Hit': 'not-supported',
		},
	});
}

/**
 * Hytale capes are not supported yet.
 */
export function retrieveCape(/*incomingRequest: Request, request: CraftheadRequest*/): Response {
	// TODO: Return something other than a 404
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
