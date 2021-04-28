/// <reference path="../pkg/crafthead.d.ts">

import {interpretRequest, MineheadRequest, RequestedKind} from './request';
import MojangRequestService from './services/mojang/service';
import {getRenderer} from './wasm';
import PromiseGatherer from "./promise_gather";
import {CachedMojangApiService, DirectMojangApiService} from "./services/mojang/api";

self.addEventListener('fetch', (event: FetchEvent) => {
    event.respondWith(handleRequest(event));
})

const skinService = new MojangRequestService(new CachedMojangApiService(new DirectMojangApiService()));

async function handleRequest(event: FetchEvent) {
    const request = event.request;

    const interpreted = interpretRequest(request);
    if (!interpreted) {
        // We don't understand this request. Pass it straight to the origin.
        return fetch(request);
    }

    console.log("Request interpreted as ", interpreted);

    try {
        let response = await caches.default.match(new Request(getCacheKey(interpreted)))
        if (!response) {
            // The item is not in the Cloudflare datacenter's cache. We need to process the request further.
            console.log("Request not satisfied from cache.");

            const gatherer = new PromiseGatherer();
            response = await processRequest(skinService, interpreted, gatherer);
            if (response.ok) {
                gatherer.push(caches.default.put(getCacheKey(interpreted), response.clone()));
            }
            event.waitUntil(gatherer.all());
        }
        const headers = decorateHeaders(interpreted, response.headers);
        return new Response(response.body, { status: response.status, headers });
    } catch (e) {
        return new Response(e.toString(), { status: 500 })
    }
}

function decorateHeaders(interpreted: MineheadRequest, headers: Headers): Headers {
    const copiedHeaders = new Headers(headers);

    // Set a liberal CORS policy - there's no harm you can do by making requests to this site...
    copiedHeaders.set('Access-Control-Allow-Origin', '*');
    copiedHeaders.set('Content-Type', interpreted.requested === RequestedKind.Profile ? 'application/json' : 'image/png');
    return copiedHeaders
}

async function processRequest(skinService: MojangRequestService, interpreted: MineheadRequest, gatherer: PromiseGatherer): Promise<Response> {
    switch (interpreted.requested) {
        case RequestedKind.Profile: {
            const lookup = await skinService.fetchProfile(interpreted, gatherer);
            if (lookup.result === null) {
                return new Response(JSON.stringify({ error: "User does not exist"}), {
                    status: 404,
                    headers: {
                        'X-Minehead-Profile-Cache-Hit': lookup.source
                    }
                });
            }
            return new Response(JSON.stringify(lookup.result), {
                headers: {
                    'X-Minehead-Profile-Cache-Hit': lookup.source
                }
            });
        }
        case RequestedKind.Avatar:
        case RequestedKind.Helm: {
            const skin = await skinService.retrieveSkin(interpreted, gatherer);
            return renderImage(skin, interpreted.size, interpreted.requested);
        }
        case RequestedKind.Skin: {
            return await skinService.retrieveSkin(interpreted, gatherer);
        }
        default:
            return new Response('must request an avatar, helm, profile, or a skin', { status: 400 });
    }
}

async function renderImage(skin: Response, size: number, requested: RequestedKind.Avatar | RequestedKind.Helm): Promise<Response> {
    const destinationHeaders = new Headers(skin.headers);
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
        default:
            throw new Error("Unknown requested kind");
    }

    return new Response(renderer.get_minecraft_head(skinBuf, size, which), {
        headers: destinationHeaders
    });
}

function getCacheKey(interpreted: MineheadRequest): string {
    return `https://crafthead.net/${interpreted.requested}/${interpreted.identity.toLocaleLowerCase('en-US')}/${interpreted.size}`
}