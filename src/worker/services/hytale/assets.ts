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

/**
 * Cached base assets (model and animation) - these are always the same
 */
interface BaseAssets {
	modelJson: string;
	animationJson: string;
}

type CacheState = 'uninitialized' | 'loading' | 'loaded';

let baseAssetsCacheState: CacheState = 'uninitialized';
let baseAssetsCache: BaseAssets | null = null;
let baseAssetsLoadingPromise: Promise<BaseAssets> | null = null;

const textDecoder = new TextDecoder();

/**
 * Load base assets (model and animation) - cached since they're always the same
 */
async function loadBaseAssets(ctx: ExecutionContext): Promise<BaseAssets> {
	if (baseAssetsCacheState === 'loaded') {
		return baseAssetsCache!;
	}

	if (baseAssetsCacheState === 'loading') {
		return baseAssetsLoadingPromise!;
	}

	baseAssetsCacheState = 'loading';
	baseAssetsLoadingPromise = (async () => {
		// Load model and animation in parallel
		const [playerModelJson, idleAnimationJson] = await Promise.all([
			readAssetFile('Common/Characters/Player.blockymodel', env, ctx),
			readAssetFile('Common/Characters/Animations/Default/Idle.blockyanim', env, ctx),
		]);

		const assets: BaseAssets = {
			modelJson: textDecoder.decode(playerModelJson),
			animationJson: textDecoder.decode(idleAnimationJson),
		};

		baseAssetsCache = assets;
		baseAssetsCacheState = 'loaded';
		return assets;
	})();

	return baseAssetsLoadingPromise;
}

/**
 * Load Hytale rendering assets
 *
 * Base assets (model/animation) are cached since they're always the same.
 * Texture is loaded per-request based on skinPath (R2/CF cache still applies).
 */
export async function loadHytaleAssets(skinPath: string = 'Common/Characters/Player_Textures/Player_Greyscale.png', ctx: ExecutionContext): Promise<HytaleAssets> {
	const normalizedSkinPath = skinPath.startsWith('Common/') ? skinPath : `Common/${skinPath}`;

	// Load base assets and texture in parallel
	const [baseAssets, playerTextureData] = await Promise.all([
		loadBaseAssets(ctx),
		readAssetFile(normalizedSkinPath, env, ctx),
	]);

	return {
		modelJson: baseAssets.modelJson,
		animationJson: baseAssets.animationJson,
		textureBytes: new Uint8Array(playerTextureData),
	};
}

/**
 * Check if base Hytale assets (model/animation) are cached
 */
export function hasHytaleAssets(): boolean {
	if (baseAssetsCacheState !== 'loaded') {
		return false;
	}
	return Boolean(baseAssetsCache?.modelJson) && Boolean(baseAssetsCache?.animationJson);
}
