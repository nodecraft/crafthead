/**
 * Hytale Asset Loader
 *
 * Loads Hytale rendering assets (model, animation, texture) for use with the
 * HytaleSkinRenderer WASM module.
 *
 * For development: Assets are bundled via imports
 * For production: Assets would be loaded from R2 (TODO)
 */

// Import model, animation, and texture files
// These are bundled at build time via wrangler rules
import idleAnimationJson from '../../../../assets/hytale/Common/Characters/Animations/Default/Idle.blockyanim';
import playerModelJson from '../../../../assets/hytale/Common/Characters/Player.blockymodel';
import playerTextureData from '../../../../assets/hytale/Common/Characters/Player_Textures/Player_Greyscale.png';

/**
 * Cached Hytale rendering assets
 */
export interface HytaleAssets {
	modelJson: string;
	animationJson: string;
	textureBytes: Uint8Array;
}

// Cache the loaded assets
let cachedAssets: HytaleAssets | null = null;

/**
 * Load Hytale rendering assets
 *
 * Assets are bundled at build time, so this is effectively synchronous.
 * Returns cached assets after first load.
 */
export function loadHytaleAssets(): HytaleAssets {
	if (cachedAssets) {
		return cachedAssets;
	}

	// Convert ArrayBuffer to Uint8Array for the texture
	const textureBytes = new Uint8Array(playerTextureData);

	cachedAssets = {
		modelJson: playerModelJson,
		animationJson: idleAnimationJson,
		textureBytes,
	};

	return cachedAssets;
}

/**
 * Check if Hytale assets are available
 */
export function hasHytaleAssets(): boolean {
	try {
		return Boolean(playerModelJson) && Boolean(idleAnimationJson) && Boolean(playerTextureData);
	} catch {
		return false;
	}
}
