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

type CacheState = 'uninitialized' | 'loading' | 'loaded';

let cacheState: CacheState = 'uninitialized';
let cache: HytaleAssets | null = null;
let loadingPromise: Promise<HytaleAssets> | null = null;

/**
 * Load Hytale rendering assets
 *
 * Assets are loaded from R2 or disk asynchronously.
 * Returns cached assets after first load.
 */
export async function loadHytaleAssets(skinPath: string = 'Common/Characters/Player_Textures/Player_Greyscale.png', ctx: ExecutionContext): Promise<HytaleAssets> {
	if (cacheState === 'loaded') {
		return cache!;
	}

	if (cacheState === 'loading') {
		return loadingPromise!;
	}
	skinPath = skinPath.startsWith('Common/') ? skinPath : `Common/${skinPath}`;

	cacheState = 'loading';
	loadingPromise = (async () => {
		const playerTextureData = await readAssetFile(skinPath, env, ctx);
		const playerModelJson = await readAssetFile('Common/Characters/Player.blockymodel', env, ctx);
		const idleAnimationJson = await readAssetFile('Common/Characters/Animations/Default/Idle.blockyanim', env, ctx);

		const textureBytes = new Uint8Array(playerTextureData);

		const assets: HytaleAssets = {
			modelJson: new TextDecoder().decode(playerModelJson),
			animationJson: new TextDecoder().decode(idleAnimationJson),
			textureBytes,
		};

		cache = assets;
		cacheState = 'loaded';
		return assets;
	})();

	return loadingPromise;
}

/**
 * Check if Hytale assets are available
 */
export function hasHytaleAssets(): boolean {
	if (cacheState !== 'loaded') {
		return false;
	}
	return Boolean(cache?.modelJson) && Boolean(cache?.animationJson) && Boolean(cache?.textureBytes);
}
