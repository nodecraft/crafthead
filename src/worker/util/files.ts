/**
 * File reader utility for Hytale assets
 *
 * Reads files from R2 in production, or from disk in local development.
 * This allows us to work around the size limitations of bundling assets
 * in Cloudflare Workers while still supporting local development.
 */

// @ts-expect-error - node:fs/promises is available with nodejs_compat flag
import { readFile } from 'node:fs/promises';
// @ts-expect-error - node:path is available with nodejs_compat flag
import pathModule from 'node:path';
// @ts-expect-error - node:url is available with nodejs_compat flag
import { fileURLToPath } from 'node:url';

/**
 * Reads an asset file from R2 (production) or disk (local development)
 *
 * @param filePath - The file path relative to assets/hytale/ (e.g., "Common/Characters/Animations/Default/Idle.blockyanim")
 * @param env - The Cloudflare Env object containing R2 bindings
 * @returns Promise resolving to the file contents as ArrayBuffer
 * @throws Error if the file is not found or cannot be read
 */
export async function readAssetFile(
	filePath: string,
	env: Cloudflare.Env,
): Promise<ArrayBuffer> {
	// Check if R2 binding is available (production)
	if (env.HYTALE_ASSETS) {
		const object = await env.HYTALE_ASSETS.get(filePath);
		if (!object) {
			console.log('No object for R2', filePath);
			throw new Error(`Asset file not found in R2: ${filePath}`);
		}
		console.log('serving asset from R2', filePath);
		return object.arrayBuffer();
	}

	throw new Error(`Asset file not found in R2: ${filePath}`);
}
