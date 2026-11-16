import { env } from 'cloudflare:workers';

import { getCosmeticJsonBytes } from './cosmetic-registry';
import { load_hytale_context } from '../../../wasm';
import { readAssetFile } from '../../util/files';

/**
 * Per-request Hytale rendering assets
 */
export interface HytaleAssets {
	textureBytes: Uint8Array;
}

const textDecoder = new TextDecoder();

let renderContextPromise: Promise<void> | null = null;

/**
 * Hand the WASM renderer everything that is the same for every render (player model, idle
 * animation, cosmetic metadata). It parses these once and keeps them for the isolate's lifetime.
 */
function loadRenderContext(ctx: ExecutionContext): Promise<void> {
	if (!renderContextPromise) {
		renderContextPromise = (async () => {
			const [playerModelJson, idleAnimationJson] = await Promise.all([
				readAssetFile('Common/Characters/Player.blockymodel', env, ctx),
				readAssetFile('Common/Characters/Animations/Default/Idle.blockyanim', env, ctx),
			]);

			const cosmeticFiles = getCosmeticJsonBytes();
			load_hytale_context(
				textDecoder.decode(playerModelJson),
				textDecoder.decode(idleAnimationJson),
				cosmeticFiles.map(file => `assets/Common/${file.path}`),
				cosmeticFiles.map(file => file.bytes),
			);
		})();
		// Without this a single failed load would fail every later render in the isolate.
		renderContextPromise.catch(() => {
			renderContextPromise = null;
		});
	}
	return renderContextPromise;
}

/**
 * Load Hytale rendering assets
 *
 * Makes sure the renderer has its shared assets, and loads the base texture for this
 * request's skinPath (R2/CF cache still applies).
 */
export async function loadHytaleAssets(skinPath: string = 'Common/Characters/Player_Textures/Player_Greyscale.png', ctx: ExecutionContext): Promise<HytaleAssets> {
	const normalizedSkinPath = skinPath.startsWith('Common/') ? skinPath : `Common/${skinPath}`;

	const [, playerTextureData] = await Promise.all([
		loadRenderContext(ctx),
		readAssetFile(normalizedSkinPath, env, ctx),
	]);

	return {
		textureBytes: new Uint8Array(playerTextureData),
	};
}
