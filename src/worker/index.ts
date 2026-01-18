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

async function handleRequest(request: Request, env: Cloudflare.Env, ctx: ExecutionContext) {
	const startTime = new Date();
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
		const cacheKey = getCacheKey(interpreted);
		let response = await caches.default.match(new Request(cacheKey));
		const hitCache = Boolean(response);
		if (!response) {
			// The item is not in the Cloudflare datacenter's cache. We need to process the request further.
			//console.log('Request not satisfied from cache.');

			response = await processRequest(request, interpreted);
			if (response.ok) {
				const cacheResponse = response.clone();
				cacheResponse.headers.set('Content-Type', interpreted.requested === RequestedKind.Profile ? 'application/json' : 'image/png');
				cacheResponse.headers.set('Cache-Control', 'max-age=14400');
				ctx.waitUntil(caches.default.put(new Request(cacheKey), cacheResponse));
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
