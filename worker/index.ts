/// <reference path="../pkg/crafthead.d.ts">

import CloudflareWorkerGlobalScope from 'types-cloudflare-worker';
import { MineheadRequest, IdentityKind, RequestedKind, interpretRequest } from './request';
import MojangRequestService from './mojang';

declare var self: CloudflareWorkerGlobalScope;
const skinService = new MojangRequestService();

self.addEventListener('fetch', event => {
    event.respondWith(handleRequest(event));
})

async function handleRequest(event: FetchEvent) {
    const request = event.request;
    const interpreted = interpretRequest(request);
    if (!interpreted) {
        // We don't understand this request. Pass it straight to the origin (Amazon S3).
        return await fetch(request);
    }

    console.log("Request interpreted as ", interpreted);

    // We use a two-layer cache. Our first layer is relatively dumb and acts on the raw request itself
    // with some mild rewriting. The second layer runs after all remapping is done.

    // Catch most repeated requests
    const l1CacheUrl = getCacheUrl(request, interpreted);
    const l1CacheResponse = await caches.default.match(l1CacheUrl);
    if (l1CacheResponse) {
        console.log("Request satisified from level 1 cache");
        return l1CacheResponse;
    }

    // Remap usernames.
    if (interpreted.identityType === IdentityKind.Username) {
        const uuidIdentity = await skinService.mapNameToUuid(interpreted.identity);
        if (uuidIdentity) {
            console.log(`Identified ${interpreted.identity} as ${uuidIdentity}`)
            interpreted.identity = uuidIdentity
            interpreted.identityType = IdentityKind.Uuid
        } else {
            // username is not valid
            console.log(`${interpreted.identity} isn't a valid username, bailing out now!`)
            return skinService.getSteveSkin()
        }
    }

    // Level 2 caching catches remapped usernames for which we have the UUIDs.
    const cacheUrl = getCacheUrl(request, interpreted);
    const cacheRequest = new Request(cacheUrl);
    const cachedResponse = await caches.default.match(cacheUrl);
    if (cachedResponse) {
        console.log("Request satisified from level 2 cache");
        return cachedResponse;
    }

    // We failed to be lazy, so we'll have to actually fetch the skin.
    console.log("Request not satisified from cache. Going to do the computationally expensive stuff.");
    const skinResponse = await processRequest(interpreted);
    if (skinResponse.ok) {
        event.waitUntil(caches.default.put(cacheRequest, skinResponse.clone()));
    }
    return skinResponse;
}

async function processRequest(interpreted: MineheadRequest): Promise<Response> {
    switch (interpreted.requested) {
        case RequestedKind.Avatar:
            return generateHead(interpreted.identity, interpreted.size);
        case RequestedKind.Skin:
            return retrieveSkinAsResponse(interpreted.identity);
        default:
            return new Response('must request an avatar or a skin', { status: 400 });
    }
}

async function retrieveSkinAsResponse(uuid: string): Promise<Response> {
    return await skinService.retrieveSkin(uuid);
}

// This is a hack in order to get webpack to play nice with the included WebAssembly module.
// See https://github.com/rustwasm/wasm-bindgen/issues/700 for more details.
async function getRenderer(): Promise<{
    get_minecraft_head(skin_image: any, size: number): any;
}> {
    return new Promise((resolve, reject) => {
        require.ensure([], function () {
            const renderer = require("../pkg/crafthead")
            return resolve(renderer);
        })
    })
}

async function generateHead(uuid: string, size: number): Promise<Response> {
    const [skinResponse, renderer] = await Promise.all([skinService.retrieveSkin(uuid), getRenderer()]);
    const skinBuf = new Uint8Array(await skinResponse.arrayBuffer());

    return new Response(renderer.get_minecraft_head(skinBuf, size));
}

function getCacheUrl(request: Request, interpreted: MineheadRequest): string {
    const urlJs = new URL(request.url);

    // Use a full URL, plus the identity in lowercase (to handle the case-insensitivity of MC usernames)
    return `${urlJs.protocol}//${urlJs.host}/${interpreted.requested}/${interpreted.identity.toLocaleLowerCase('en-US')}/${interpreted.size}`;
}