import CloudflareWorkerGlobalScope from 'types-cloudflare-worker';
import { MineheadRequest, IdentityKind, RequestedKind, interpretRequest } from './request';
declare var self: CloudflareWorkerGlobalScope;

self.addEventListener('fetch', event => {
    event.respondWith(handleRequest(event))
})

async function handleRequest(event: FetchEvent) {
    const request = event.request
    const interpreted = interpretRequest(request)
    if (!interpreted) {
        // We don't understand this request. Pass it straight to the origin (Amazon S3).
        return await fetch(request)
    }

    console.log("Request interpreted as ", interpreted)

    if (interpreted.identityType === IdentityKind.Username) {
        // Usernames are not yet supported.
        return new Response('username fetching not yet supported', { status: 400 })
    }

    // Catch top-level requests 
    const cacheUrl = getCacheUrl(request, interpreted)
    const cacheRequest = new Request(cacheUrl)
    const cachedResponse = await caches.default.match(cacheRequest)
    if (cachedResponse) {
        console.log("Request satisified from cache")
        return cachedResponse
    }

    // We failed to be lazy, so we'll have to actually fetch the skin.
    console.log("Request not satisified from cache. Going to do the computationally expensive stuff.")
    const skinResponse = await processRequest(interpreted)
    if (skinResponse.ok) {
        event.waitUntil(caches.default.put(cacheRequest, skinResponse.clone()))
    }
    return skinResponse
}

async function processRequest(interpreted: MineheadRequest) {
    switch (interpreted.requested) {
        case RequestedKind.Avatar:
            return generateHead(interpreted.identity, interpreted.size)
        case RequestedKind.Skin:
            return retrieveSkinAsResponse(interpreted.identity)
        default:
            return new Response('must request an avatar or a skin', { status: 400 })
    }
}

function getCacheUrl(request: Request, interpreted: MineheadRequest): string {
    const urlJs = new URL(request.url)
    return `${urlJs.protocol}//${urlJs.host}/${interpreted.requested}/${interpreted.identity}/${interpreted.size}`
}