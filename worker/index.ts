/// <reference path="../pkg/crafthead.d.ts">

import CloudflareWorkerGlobalScope from 'types-cloudflare-worker';
import {interpretRequest, MineheadRequest, RequestedKind} from './request';
import MojangRequestService from './services/mojang/service';
import {getRenderer} from './wasm';
import {ArrayBufferCloudflareResponseMapper, CloudflareCacheService} from './services/cache/cloudflare';
import ResponseCacheService from './services/cache/response_helper';
import PromiseGatherer from "./promise_gather";
import {DirectMojangApiService} from "./services/mojang/api";
import NoopCacheService from "./services/cache/noop";

declare var self: CloudflareWorkerGlobalScope;

self.addEventListener('fetch', event => {
    event.respondWith(handleRequest(event));
})

const l1Cache = new ResponseCacheService(
    new NoopCacheService(),
    new CloudflareCacheService('general-cache', new ArrayBufferCloudflareResponseMapper())
);
const skinService = new MojangRequestService(new DirectMojangApiService());

async function handleRequest(event: FetchEvent) {
    const request = event.request;

    const interpreted = interpretRequest(request);
    if (!interpreted) {
        // We don't understand this request. Pass it straight to the origin (Amazon S3).
        return fetch(request);
    }

    console.log("Request interpreted as ", interpreted);

    try {
        // If the result is cached, we don't need to do anything else
        let response = await l1Cache.find(getCacheKey(interpreted))
        if (!response) {
            // We failed to be lazy, so we'll have to actually fetch the skin.
            console.log("Request not satisfied from cache.");

            const gatherer = new PromiseGatherer();
            response = await processRequest(skinService, interpreted, gatherer);
            if (response.ok) {
                gatherer.push(l1Cache.put(getCacheKey(interpreted), response.clone()));
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
            const profile = await skinService.fetchProfile(interpreted, gatherer);
            if (profile === null) {
                return new Response(JSON.stringify({ error: "Unable to fetch the profile"}), { status: 500 });
            }
            return new Response(JSON.stringify(profile));
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
    const skinCacheHit = destinationHeaders.get('X-Minehead-Cache-Hit')
    if (skinCacheHit) {
        destinationHeaders.set('X-Minehead-Skin-Cache-Hit', skinCacheHit)
        destinationHeaders.delete('X-Minehead-Cache-Hit')
    }

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
    return `${interpreted.requested}/${interpreted.identity.toLocaleLowerCase('en-US')}/${interpreted.size}`
}