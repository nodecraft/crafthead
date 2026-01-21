import { env } from 'cloudflare:workers';

import { readAssetFile } from '../../util/files';

/**
 * Cached Hytale rendering assets
 */
export interface HytaleAssets {
	modelJson: string;
	animationJson: string;
	textureBytes: Uint8Array;
}

// Cache the loaded assets or the loading promise to prevent race conditions
let cache: HytaleAssets | Promise<HytaleAssets> | null = null;

/**
 * Load Hytale rendering assets
 *
 * Assets are loaded from R2 or disk asynchronously.
 * Returns cached assets after first load.
 */
export async function loadHytaleAssets(): Promise<HytaleAssets> {
	// If we have cached assets (not a promise), return them
	if (cache && !(cache instanceof Promise)) {
		return cache;
	}

	// If already loading (cache is a promise), wait for it
	if (cache instanceof Promise) {
		return cache;
	}

	// Start loading and cache the promise
	const loadingPromise = (async () => {
		const playerTextureData = await readAssetFile('Common/Characters/Player_Textures/Player_Greyscale.png', env);
		const playerModelJson = await readAssetFile('Common/Characters/Player.blockymodel', env);
		const idleAnimationJson = await readAssetFile('Common/Characters/Animations/Default/Idle.blockyanim', env);

		// Convert ArrayBuffer to Uint8Array for the texture
		const textureBytes = new Uint8Array(playerTextureData);

		const assets: HytaleAssets = {
			modelJson: new TextDecoder().decode(playerModelJson),
			animationJson: new TextDecoder().decode(idleAnimationJson),
			textureBytes,
		};

		// Replace the promise with the actual assets
		cache = assets;
		return assets;
	})();

	// Cache the promise so concurrent calls wait for the same load
	cache = loadingPromise;
	return loadingPromise;
}

/**
 * Check if Hytale assets are available
 */
export function hasHytaleAssets(): boolean {
	try {
		const assets = cache && !(cache instanceof Promise) ? cache : null;
		return Boolean(assets?.modelJson) && Boolean(assets?.animationJson) && Boolean(assets?.textureBytes);
	} catch {
		return false;
	}
}
