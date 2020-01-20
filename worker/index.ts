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

    if (interpreted.identityType === IdentityKind.Username) {
        // Usernames are not yet supported.
        return new Response('username fetching not yet supported', { status: 400 });
    }

    // Catch top-level requests 
    const cacheUrl = getCacheUrl(request, interpreted);
    const cacheRequest = new Request(cacheUrl);
    const cachedResponse = await caches.default.match(cacheRequest);
    if (cachedResponse) {
        console.log("Request satisified from cache");
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
    return `${urlJs.protocol}//${urlJs.host}/${interpreted.requested}/${interpreted.identity}/${interpreted.size}`;
}