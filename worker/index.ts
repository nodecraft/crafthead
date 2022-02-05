/// <reference path="../pkg/crafthead.d.ts">

import { getAssetFromKV } from "@cloudflare/kv-asset-handler"
import {interpretRequest, CraftheadRequest, RequestedKind} from './request';
import MojangRequestService from './services/mojang/service';
import {getRenderer} from './wasm';
import PromiseGatherer from "./promise_gather";
import {CachedMojangApiService, DirectMojangApiService} from "./services/mojang/api";
import { default as CACHE_BUST } from './util/cache-bust';
import { EMPTY } from "./data";

self.addEventListener('fetch', (event: FetchEvent) => {
    event.respondWith(handleRequest(event));
})

const skinService = new MojangRequestService(new CachedMojangApiService(new DirectMojangApiService()));

async function handleRequest(event: FetchEvent) {
    const request = event.request;

    const interpreted = interpretRequest(request);
    if (!interpreted) {
        // We don't understand this request.
        try {
            return await getAssetFromKV(event);
        } catch (e) {
            try {
                const notFoundResponse = await getAssetFromKV(event, {
                    mapRequestToAsset: req => new Request(`${new URL(req.url).origin}/404.html`, req),
                });
                return new Response(notFoundResponse.body, { ...notFoundResponse, status: 404 });
            } catch (e) {
                return new Response("Not found", { status: 404 });
            }
        }
    }

    console.log("Request interpreted as ", interpreted);

    try {
        const cacheKey = getCacheKey(interpreted);
        let response = await caches.default.match(new Request(cacheKey));
        const hitCache = !!response;
        if (!response) {
            // The item is not in the Cloudflare datacenter's cache. We need to process the request further.
            console.log("Request not satisfied from cache.");

            const gatherer = new PromiseGatherer();
            response = await processRequest(skinService, interpreted, gatherer);
            if (response.ok) {
                const cacheResponse = response.clone();
                cacheResponse.headers.set('Content-Type', interpreted.requested === RequestedKind.Profile ? 'application/json' : 'image/png');
                cacheResponse.headers.set('Cache-Control', 'max-age=14400');
                gatherer.push(caches.default.put(new Request(cacheKey), cacheResponse));
            }
            await gatherer.all();
        }
        const headers = decorateHeaders(interpreted, response.headers, hitCache);
        return new Response(response.body, { status: response.status, headers });
    } catch (e) {
        return new Response(e.toString(), { status: 500 })
    }
}

function decorateHeaders(interpreted: CraftheadRequest, headers: Headers, hitCache: boolean): Headers {
    const copiedHeaders = new Headers(headers);

    // Set a liberal CORS policy - there's no harm you can do by making requests to this site...
    copiedHeaders.set('Access-Control-Allow-Origin', '*');
    copiedHeaders.set('Cache-Control', 'max-age=14400');
    copiedHeaders.set('X-Crafthead-Request-Cache-Hit', hitCache ? 'yes' : 'no');
    if (!copiedHeaders.has('Content-Type')) {
        copiedHeaders.set('Content-Type', interpreted.requested === RequestedKind.Profile ? 'application/json' : 'image/png');
    } else {
        console.log(`Content-Type header already on response: ${copiedHeaders.get('Content-Type')}, not overriding.`);
    }

    return copiedHeaders
}

async function processRequest(skinService: MojangRequestService, interpreted: CraftheadRequest, gatherer: PromiseGatherer): Promise<Response> {
    switch (interpreted.requested) {
        case RequestedKind.Profile: {
            const lookup = await skinService.fetchProfile(interpreted, gatherer);
            if (!lookup.result) {
                return new Response(JSON.stringify({ error: "User does not exist"}), {
                    status: 404,
                    headers: {
                        'X-Crafthead-Profile-Cache-Hit': lookup.source
                    }
                });
            }
            return new Response(JSON.stringify(lookup.result), {
                headers: {
                    'X-Crafthead-Profile-Cache-Hit': lookup.source
                }
            });
        }
        case RequestedKind.Avatar:
        case RequestedKind.Helm:
        case RequestedKind.Cube:
        case RequestedKind.Body: {
            const skin = await skinService.retrieveSkin(interpreted, gatherer);
            return renderImage(skin, interpreted);
        }
        case RequestedKind.Skin: {
            return await skinService.retrieveSkin(interpreted, gatherer);
        }
        case RequestedKind.Cape: {
            const cape = await skinService.retrieveCape(interpreted, gatherer);
            if (cape.status === 404) {
                return new Response(EMPTY, {
                    status: 404,
                    headers: {
                        'X-Crafthead-Profile-Cache-Hit': 'invalid-profile'
                    }
                });
            }
            return renderImage(cape, interpreted);
        }
        default:
            return new Response('must request an avatar, helm, body, profile, or a skin', { status: 400 });
    }
}

async function renderImage(skin: Response, request: CraftheadRequest): Promise<Response> {
    const {size, requested, armored} = request;
    const destinationHeaders = new Headers(skin.headers);
    const slim = destinationHeaders.get('X-Crafthead-Skin-Model') === 'slim';
    const [renderer, skinArrayBuffer] = await Promise.all([getRenderer(), skin.arrayBuffer()]);
    const skinBuf = new Uint8Array(skinArrayBuffer);

    let which: string
    switch (requested) {
        case RequestedKind.Avatar:
            which = "avatar";
            break;
        case RequestedKind.Helm:
            which = "helm";
            break;
        case RequestedKind.Cube:
            which = "cube";
            break;
        case RequestedKind.Body:
            which = "body";
            break;
        case RequestedKind.Cape:
            which = "cape";
            break;
        default:
            throw new Error("Unknown requested kind");
    }

    return new Response(renderer.get_rendered_image(skinBuf, size, which, armored, slim), {
        headers: destinationHeaders
    });
}

function getCacheKey(interpreted: CraftheadRequest): string {
    return `https://crafthead.net/__public${CACHE_BUST}/${interpreted.requested}/${interpreted.armored}/${interpreted.identity.toLocaleLowerCase('en-US')}/${interpreted.size}`
}