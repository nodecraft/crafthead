/// <reference path="../pkg/crafthead.d.ts">

import { getAssetFromKV } from "@cloudflare/kv-asset-handler"
import {interpretRequest, CraftheadRequest, RequestedKind} from './request';
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
        let response = /*await caches.default.match(new Request(getCacheKey(interpreted)))*/ null;
        if (!response) {
            // The item is not in the Cloudflare datacenter's cache. We need to process the request further.
            console.log("Request not satisfied from cache.");

            const gatherer = new PromiseGatherer();
            response = await processRequest(skinService, interpreted, gatherer);
            if (response.ok) {
                //gatherer.push(caches.default.put(getCacheKey(interpreted), response.clone()));
            }
            event.waitUntil(gatherer.all());
        }
        const headers = decorateHeaders(interpreted, response.headers);
        return new Response(response.body, { status: response.status, headers });
    } catch (e) {
        return new Response(e.toString(), { status: 500 })
    }
}

function decorateHeaders(interpreted: CraftheadRequest, headers: Headers): Headers {
    const copiedHeaders = new Headers(headers);

    // Set a liberal CORS policy - there's no harm you can do by making requests to this site...
    copiedHeaders.set('Access-Control-Allow-Origin', '*');
    copiedHeaders.set('Content-Type', interpreted.requested === RequestedKind.Profile ? 'application/json' : 'image/png');
    return copiedHeaders
}

async function processRequest(skinService: MojangRequestService, interpreted: CraftheadRequest, gatherer: PromiseGatherer): Promise<Response> {
    switch (interpreted.requested) {
        case RequestedKind.Profile: {
            const lookup = await skinService.fetchProfile(interpreted, gatherer);
            if (lookup.result === null) {
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
        case RequestedKind.Cube: {
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

async function renderImage(skin: Response, size: number, requested: RequestedKind.Avatar | RequestedKind.Helm | RequestedKind.Cube): Promise<Response> {
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
        case RequestedKind.Cube:
            which = "cube";
            break;
        default:
            throw new Error("Unknown requested kind");
    }

    return new Response(renderer.get_minecraft_head(skinBuf, size, which), {
        headers: destinationHeaders
    });
}

function getCacheKey(interpreted: CraftheadRequest): string {
    return `https://crafthead.net/${interpreted.requested}/${interpreted.identity.toLocaleLowerCase('en-US')}/${interpreted.size}`
}