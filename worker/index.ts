/// <reference path="../pkg/crafthead.d.ts">

import CloudflareWorkerGlobalScope from 'types-cloudflare-worker';
import PromiseGatherer from './promise_gather';
import { MineheadRequest, IdentityKind, RequestedKind, interpretRequest } from './request';
import MojangRequestService from './mojang';
import { getRenderer } from './wasm';

declare var self: CloudflareWorkerGlobalScope;

self.addEventListener('fetch', event => {
    event.respondWith(handleRequest(event));
})

async function handleRequest(event: FetchEvent) {
    const request = event.request;

    // a debug endpoint to diagnose high startup times
    if (request.url.endsWith("/testing1234/ping")) {
        return new Response("ping")
    }

    const interpreted = interpretRequest(request);
    if (!interpreted) {
        // We don't understand this request. Pass it straight to the origin (Amazon S3).
        return fetch(request);
    }

    console.log("Request interpreted as ", interpreted);

    // We use a two-layer cache. Our first layer is relatively dumb and acts on the raw request itself
    // with some mild rewriting. The second layer runs after all remapping is done.
    const gatherer = new PromiseGatherer();

    try {
        // If the result is cached, we don't need to do aything else
        const l1CacheRequest = getCacheRequest(request, interpreted);
        const l1CacheResponse = await caches.default.match(l1CacheRequest);
        if (l1CacheResponse) {
            console.log("Request satisified from level 1 cache");
            return l1CacheResponse;
        }

        // We failed to be lazy, so we'll have to actually fetch the skin.
        const skinService = new MojangRequestService(gatherer);
        console.log("Request not satisified from cache. Going to do the computationally expensive stuff.");
        const skinResponse = await processRequest(skinService, interpreted);
        if (skinResponse.ok) {
            gatherer.push(caches.default.put(l1CacheRequest, skinResponse.clone()));
        }
        return skinResponse;
    } finally {
        event.waitUntil(gatherer.all());
    }
}

async function processRequest(skinService: MojangRequestService, interpreted: MineheadRequest): Promise<Response> {
    const skin = await skinService.retrieveSkin(interpreted.identity, interpreted.identityType);
    switch (interpreted.requested) {
        case RequestedKind.Avatar:
            return generateHead(skin, interpreted.size);
        case RequestedKind.Skin:
            return skin;
        default:
            return new Response('must request an avatar or a skin', { status: 400 });
    }
}

async function generateHead(skin: Response, size: number): Promise<Response> {
    const [renderer, skinArrayBuffer] = await Promise.all([getRenderer(), skin.arrayBuffer()]);
    const skinBuf = new Uint8Array(skinArrayBuffer);
    return new Response(renderer.get_minecraft_head(skinBuf, size), {
        headers: {
            'Cache-Control': 'public, max-age=21600'
        }
    });
}

function getCacheRequest(request: Request, interpreted: MineheadRequest): Request {
    const urlJs = new URL(request.url);

    // Use a full URL, plus the identity in lowercase (to handle the case-insensitivity of MC usernames)
    const cacheUrl = `${urlJs.protocol}//${urlJs.host}/${interpreted.requested}/${interpreted.identity.toLocaleLowerCase('en-US')}/${interpreted.size}`;
    return new Request(cacheUrl, {
        headers: request.headers
    })
}