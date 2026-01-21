// @ts-expect-error - node:fs/promises is available with nodejs_compat flag
import { readFile, readdir, stat } from 'node:fs/promises';
// @ts-expect-error - node:path is available with nodejs_compat flag
import pathModule from 'node:path';

import { EMPTY } from './data';
import { Game, RequestedKind, interpretRequest } from './request';
import * as hytaleService from './services/hytale/service';
import * as mojangService from './services/mojang/service';
import { writeDataPoint } from './util/analytics';
import { default as CACHE_BUST } from './util/cache-bust';
import { get_rendered_image } from '../../pkg/mcavatar';

import type { CraftheadRequest } from './request';

interface DirectRenderService {
	renderAvatar(incomingRequest: Request, request: CraftheadRequest): Promise<Response>;
}

type GameService = typeof mojangService | typeof hytaleService;

function decorateHeaders(interpreted: CraftheadRequest, headers: Headers, hitCache: boolean): Headers {
	const copiedHeaders = new Headers(headers);

	// Set a liberal CORS policy - there's no harm you can do by making requests to this site...
	copiedHeaders.set('Access-Control-Allow-Origin', '*');
	copiedHeaders.set('Cache-Control', 'max-age=14400');
	copiedHeaders.set('X-Crafthead-Request-Cache-Hit', hitCache ? 'yes' : 'no');
	if (!copiedHeaders.has('Content-Type')) {
		copiedHeaders.set('Content-Type', interpreted.requested === RequestedKind.Profile ? 'application/json' : 'image/png');
	} else if (copiedHeaders.get('Content-Type') !== 'application/json' && copiedHeaders.get('Content-Type')?.includes?.('text/plain') && interpreted.requested === RequestedKind.Profile) {
		copiedHeaders.set('Content-Type', 'application/json');
	} else {
		console.log(`Content-Type header already on response: ${copiedHeaders.get('Content-Type')}, not overriding.`);
	}

	return copiedHeaders;
}


function getCacheKey(interpreted: CraftheadRequest): string {
	// use old cache key for minecraft requests
	if (interpreted.game === Game.Minecraft) {
		return `https://crafthead.net/__public${CACHE_BUST}/${interpreted.requested}/${interpreted.armored}/${interpreted.model}/${interpreted.identity.toLowerCase()}/${interpreted.size}`;
	}
	// new cache key format for hytale and future games
	return `https://crafthead.net/__public${CACHE_BUST}/${interpreted.game}/${interpreted.requested}/${interpreted.armored}/${interpreted.model}/${interpreted.identity.toLowerCase()}/${interpreted.size}`;
}

const RENDER_TYPE_MAP: Record<number, string> = {
	[RequestedKind.Avatar]: 'avatar',
	[RequestedKind.Helm]: 'helm',
	[RequestedKind.Cube]: 'cube',
	[RequestedKind.Body]: 'body',
	[RequestedKind.Bust]: 'bust',
	[RequestedKind.Cape]: 'cape',
} as const;

async function renderImage(skin: Response, request: CraftheadRequest): Promise<Response> {
	const { size, requested, armored, game } = request;
	const destinationHeaders = new Headers(skin.headers);
	const slim = destinationHeaders.get('X-Crafthead-Skin-Model') === 'slim';
	const skinBuf = await skin.bytes();

	const which = RENDER_TYPE_MAP[requested];
	if (!which) {
		throw new Error('Unknown requested kind');
	}

	return new Response(get_rendered_image(skinBuf, size, which, armored, slim, game), {
		headers: destinationHeaders,
	});
}

function getService(game: Game): GameService {
	switch (game) {
		case Game.Minecraft: {
			return mojangService;
		}
		case Game.Hytale: {
			return hytaleService;
		}
		default: {
			return mojangService;
		}
	}
}

function hasRenderAvatar(service: GameService): service is GameService & DirectRenderService {
	return 'renderAvatar' in service;
}

/**
 * Get content type based on file extension
 */
function getContentType(filePath: string): string {
	const ext = pathModule.extname(filePath).toLowerCase();
	switch (ext) {
		case '.png': {
			return 'image/png';
		}
		case '.json': {
			return 'application/json';
		}
		case '.blockymodel':
		case '.blockyanim': {
			return 'application/octet-stream';
		}
		default: {
			return 'application/octet-stream';
		}
	}
}

/**
 * Recursively get all files from a directory
 */
async function getAllFiles(dirPath: string, basePath: string = dirPath): Promise<string[]> {
	const files: string[] = [];
	const entries = await readdir(dirPath, { withFileTypes: true });

	for (const entry of entries) {
		const fullPath = pathModule.join(dirPath, entry.name);
		if (entry.isDirectory()) {
			const subFiles = await getAllFiles(fullPath, basePath);
			files.push(...subFiles);
		} else {
			files.push(fullPath);
		}
	}

	return files;
}

/**
 * Upload all Hytale assets from assets/hytale/ to R2
 */
async function uploadAssetsToR2(env: Cloudflare.Env): Promise<Response> {
	if (!env.HYTALE_ASSETS) {
		return new Response(
			JSON.stringify({
				error: 'R2 bucket HYTALE_ASSETS is not available',
				success: false,
			}),
			{
				status: 500,
				headers: { 'Content-Type': 'application/json' },
			},
		);
	}

	// @ts-expect-error - process.cwd() is available with nodejs_compat flag
	const assetsDir = pathModule.join(process.cwd(), 'assets', 'hytale');
	const stats: {
		total: number;
		successful: number;
		failed: number;
		errors: Array<{ file: string; error: string; }>;
		uploaded: string[];
	} = {
		total: 0,
		successful: 0,
		failed: 0,
		errors: [],
		uploaded: [],
	};

	try {
		// Check if directory exists
		try {
			const dirStat = await stat(assetsDir);
			if (!dirStat.isDirectory()) {
				return new Response(
					JSON.stringify({
						error: `Path ${assetsDir} is not a directory`,
						success: false,
					}),
					{
						status: 400,
						headers: { 'Content-Type': 'application/json' },
					},
				);
			}
		} catch (err) {
			return new Response(
				JSON.stringify({
					error: `Assets directory not found: ${assetsDir}. Error: ${(err as Error).message}`,
					success: false,
				}),
				{
					status: 404,
					headers: { 'Content-Type': 'application/json' },
				},
			);
		}

		// Get all files recursively
		const allFiles = await getAllFiles(assetsDir);
		stats.total = allFiles.length;

		// Upload each file to R2
		for (const filePath of allFiles) {
			try {
				// Get relative path from assets/hytale/ directory
				const relativePath = pathModule.relative(assetsDir, filePath).replaceAll('\\', '/');

				// Read file content
				const fileContent = await readFile(filePath);

				// Upload to R2
				await env.HYTALE_ASSETS.put(relativePath, fileContent, {
					httpMetadata: {
						contentType: getContentType(filePath),
					},
				});

				stats.successful++;
				stats.uploaded.push(relativePath);
			} catch (err) {
				stats.failed++;
				const relativePath = pathModule.relative(assetsDir, filePath).replaceAll('\\', '/');
				stats.errors.push({
					file: relativePath,
					error: (err as Error).message,
				});
				console.error(`Failed to upload ${filePath}:`, err);
			}
		}

		return new Response(
			JSON.stringify({
				success: true,
				stats,
			}),
			{
				status: 200,
				headers: { 'Content-Type': 'application/json' },
			},
		);
	} catch (err) {
		return new Response(
			JSON.stringify({
				error: `Failed to process assets: ${(err as Error).message}`,
				success: false,
				stats,
			}),
			{
				status: 500,
				headers: { 'Content-Type': 'application/json' },
			},
		);
	}
}

async function processRequest(request: Request, interpreted: CraftheadRequest): Promise<Response> {
	const service = getService(interpreted.game);

	switch (interpreted.requested) {
		case RequestedKind.Profile: {
			const lookup = await service.fetchProfile(request, interpreted);
			if (!lookup.result) {
				return new Response(JSON.stringify({ error: 'User does not exist' }), {
					status: 404,
					headers: {
						'X-Crafthead-Profile-Cache-Hit': lookup.source,
					},
				});
			}
			return new Response(JSON.stringify(lookup.result), {
				headers: {
					'X-Crafthead-Profile-Cache-Hit': lookup.source,
				},
			});
		}
		case RequestedKind.Avatar:
		case RequestedKind.Helm:
		case RequestedKind.Cube:
		case RequestedKind.Body:
		case RequestedKind.Bust: {
			if (hasRenderAvatar(service)) {
				return service.renderAvatar(request, interpreted);
			}
			const skin = await service.retrieveSkin(request, interpreted);
			return renderImage(skin, interpreted);
		}
		case RequestedKind.Skin: {
			return service.retrieveSkin(request, interpreted);
		}
		case RequestedKind.Cape: {
			const cape = await service.retrieveCape(request, interpreted);
			if (cape.status === 404) {
				return new Response(EMPTY, {
					status: 404,
					headers: {
						'X-Crafthead-Profile-Cache-Hit': cape.headers.get('X-Crafthead-Profile-Cache-Hit') || 'invalid-profile',
					},
				});
			}
			return renderImage(cape, interpreted);
		}
		default: {
			return new Response('must request an avatar, helm, body, profile, or a skin', { status: 400 });
		}
	}
}

/**
 * Upload a single asset file to R2
 */
async function uploadSingleAssetToR2(filePath: string, fileContent: ArrayBuffer, contentType: string, env: Cloudflare.Env): Promise<{ success: boolean; error?: string; }> {
	if (!env.HYTALE_ASSETS) {
		return { success: false, error: 'R2 bucket HYTALE_ASSETS is not available' };
	}

	try {
		// Upload to R2
		await env.HYTALE_ASSETS.put(filePath, fileContent, {
			httpMetadata: {
				contentType,
			},
		});

		return { success: true };
	} catch (err) {
		return { success: false, error: (err as Error).message };
	}
}

async function handleRequest(request: Request, env: Cloudflare.Env, _ctx: ExecutionContext) {
	const startTime = new Date();

	// Handle temporary upload endpoints
	const url = new URL(request.url);
	if (url.pathname === '/upload-assets' && request.method === 'POST') {
		return uploadAssetsToR2(env);
	}
	if (url.pathname === '/upload-asset' && request.method === 'POST') {
		const requestContentType = request.headers.get('Content-Type') || '';

		if (requestContentType.includes('application/json')) {
			// Legacy format - just filePath (not used anymore but kept for compatibility)
			const body = await request.json();
			const { filePath } = body as { filePath?: string; };
			if (!filePath) {
				return new Response(JSON.stringify({ success: false, error: 'filePath is required' }), {
					status: 400,
					headers: { 'Content-Type': 'application/json' },
				});
			}
			return new Response(JSON.stringify({ success: false, error: 'File content must be provided in request body' }), {
				status: 400,
				headers: { 'Content-Type': 'application/json' },
			});
		}

		// New format: multipart/form-data with file content
		const formData = await request.formData();
		const filePath = formData.get('filePath') as string | null;
		const file = formData.get('file') as File | null;
		const fileContentType = formData.get('contentType') as string | null;

		if (!filePath || !file) {
			return new Response(JSON.stringify({ success: false, error: 'filePath and file are required' }), {
				status: 400,
				headers: { 'Content-Type': 'application/json' },
			});
		}

		const fileContent = await file.arrayBuffer();
		const contentType = fileContentType || getContentType(filePath);
		const result = await uploadSingleAssetToR2(filePath, fileContent, contentType, env);

		return new Response(JSON.stringify(result), {
			status: result.success ? 200 : 500,
			headers: { 'Content-Type': 'application/json' },
		});
	}

	const interpreted = interpretRequest(request);
	if (!interpreted) {
		// We don't understand this request.
		try {
			const asset = await env.ASSETS.fetch(request);
			writeDataPoint(env.CRAFTHEAD_ANALYTICS, request, {
				startTime,
				kind: '_asset',
				responseCode: 200,
			});
			return asset;
		} catch (err) {
			try {
				writeDataPoint(env.CRAFTHEAD_ANALYTICS, request, {
					startTime,
					kind: '_asset',
					responseCode: 500,
				});
				const probableError = err as Error;
				return new Response(probableError?.message || probableError.toString(), { status: 500 });
			} catch {
				writeDataPoint(env.CRAFTHEAD_ANALYTICS, request, {
					startTime,
					kind: '_asset',
					responseCode: 404,
				});
				return new Response('Not found', { status: 404 });
			}
		}
	}

	//console.log('Request interpreted as ', interpreted);

	try {
		// const cacheKey = getCacheKey(interpreted);
		// let response = await caches.default.match(new Request(cacheKey));
		let response: Response | undefined;
		const hitCache = Boolean(response);
		if (!response) {
			// The item is not in the Cloudflare datacenter's cache. We need to process the request further.
			//console.log('Request not satisfied from cache.');

			response = await processRequest(request, interpreted);
			if (response.ok) {
				const cacheResponse = response.clone();
				cacheResponse.headers.set('Content-Type', interpreted.requested === RequestedKind.Profile ? 'application/json' : 'image/png');
				cacheResponse.headers.set('Cache-Control', 'max-age=14400');
				// ctx.waitUntil(caches.default.put(new Request(cacheKey), cacheResponse));
			}
		}
		const headers = decorateHeaders(interpreted, response.headers, hitCache);
		writeDataPoint(env.CRAFTHEAD_ANALYTICS, request, {
			startTime,
			kind: interpreted.requestedKindString,
			identityType: interpreted.identityType,
			responseCode: response.status,
			cached: hitCache,
			game: interpreted.game,
		});
		return new Response(response.body, { status: response.status, headers });
	} catch (err) {
		if ((err as Error).name === 'TimeoutError') {
			writeDataPoint(env.CRAFTHEAD_ANALYTICS, request, {
				startTime,
				kind: interpreted.requestedKindString,
				identityType: interpreted.identityType,
				responseCode: 504,
				game: interpreted.game,
			});
			return new Response('Upstream request timed out', { status: 504 });
		}
		writeDataPoint(env.CRAFTHEAD_ANALYTICS, request, {
			startTime,
			kind: interpreted.requestedKindString,
			identityType: interpreted.identityType,
			responseCode: 500,
			game: interpreted.game,
		});
		return new Response((err as Error).toString(), { status: 500 });
	}
}

export default {
	fetch(request, env, ctx): Promise<Response> {
		return handleRequest(request, env, ctx);
	},
} satisfies ExportedHandler<Cloudflare.Env>;
