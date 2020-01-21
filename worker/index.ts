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
        // Catch most repeated requests
        const skinService = new MojangRequestService(gatherer);

        const l1CacheRequest = getCacheRequest(request, interpreted);
        const l1CacheResponse = await caches.default.match(l1CacheRequest);
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
        const l2CacheRequest = getCacheRequest(request, interpreted);
        const l2CacheResponse = await caches.default.match(l2CacheRequest);
        if (l2CacheResponse) {
            console.log("Request satisified from level 2 cache");
            gatherer.push(caches.default.put(l1CacheRequest, l2CacheResponse.clone()));
            return l2CacheResponse;
        }

        // We failed to be lazy, so we'll have to actually fetch the skin.
        console.log("Request not satisified from cache. Going to do the computationally expensive stuff.");
        const skinResponse = await processRequest(skinService, interpreted);
        if (skinResponse.ok) {
            if (l2CacheRequest.url !== l1CacheRequest.url) {
                gatherer.push(caches.default.put(l1CacheRequest, skinResponse.clone()))
            }
            gatherer.push(caches.default.put(l2CacheRequest, skinResponse.clone()));
        }
        return skinResponse;
    } finally {
        event.waitUntil(gatherer.all());
    }
}

async function processRequest(skinService: MojangRequestService, interpreted: MineheadRequest): Promise<Response> {
    switch (interpreted.requested) {
        case RequestedKind.Avatar:
            return generateHead(skinService, interpreted.identity, interpreted.size);
        case RequestedKind.Skin:
            return skinService.retrieveSkin(interpreted.identity);
        default:
            return new Response('must request an avatar or a skin', { status: 400 });
    }
}

async function generateHead(skinService: MojangRequestService, uuid: string, size: number): Promise<Response> {
    const [skinResponse, renderer] = await Promise.all([skinService.retrieveSkin(uuid), getRenderer()]);
    const skinBuf = new Uint8Array(await skinResponse.arrayBuffer());
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