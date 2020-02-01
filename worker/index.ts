/// <reference path="../pkg/crafthead.d.ts">

import CloudflareWorkerGlobalScope from 'types-cloudflare-worker';
import { MineheadRequest, RequestedKind, interpretRequest } from './request';
import MojangRequestService from './services/mojang';
import { getRenderer } from './wasm';
import { CloudflareCacheService, ArrayBufferCloudflareResponseMapper } from './services/cache/cloudflare';
import MemoryCacheService from './services/cache/memory';
import ResponseCacheService from './services/cache/response_helper';

declare var self: CloudflareWorkerGlobalScope;

self.addEventListener('fetch', event => {
    event.respondWith(handleRequest(event));
})

const l1Cache = new ResponseCacheService(
    new MemoryCacheService(),
    new CloudflareCacheService('general-cache', new ArrayBufferCloudflareResponseMapper())
);
const skinService = new MojangRequestService();

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

    try {
        // If the result is cached, we don't need to do aything else
        const l1CacheResponse = await l1Cache.find(getCacheKey(interpreted))
        if (l1CacheResponse) {
            return l1CacheResponse;
        }

        // We failed to be lazy, so we'll have to actually fetch the skin.
        console.log("Request not satisified from cache.");
        const skinResponse = await processRequest(skinService, interpreted);
        if (skinResponse.ok) {
            event.waitUntil(l1Cache.put(getCacheKey(interpreted), skinResponse.clone()));
        }
        return skinResponse;
    } catch (e) {
        return new Response(e.toString(), { status: 500 })
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
    const destinationHeaders = new Headers(skin.headers);
    const skinCacheHit = destinationHeaders.get('X-Minehead-Cache-Hit')
    if (skinCacheHit) {
        destinationHeaders.set('X-Minehead-Skin-Cache-Hit', skinCacheHit)
        destinationHeaders.delete('X-Minehead-Cache-Hit')
    }

    const [renderer, skinArrayBuffer] = await Promise.all([getRenderer(), skin.arrayBuffer()]);
    const skinBuf = new Uint8Array(skinArrayBuffer);
    return new Response(renderer.get_minecraft_head(skinBuf, size), {
        headers: destinationHeaders
    });
}

function getCacheKey(interpreted: MineheadRequest): string {
    return `${interpreted.requested}/${interpreted.identity.toLocaleLowerCase('en-US')}/${interpreted.size}`
}