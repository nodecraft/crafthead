import PromiseGatherer from "../promise_gather";

export interface CacheComputeResult<T> {
    result: T;
    source: string;
}

declare const CRAFTHEAD_PROFILE_CACHE: KVNamespace;

export async function computeObject<T>(key: string, source: () => Promise<T | null>, gatherer: PromiseGatherer | null): Promise<CacheComputeResult<T | null>> {
    const localCacheUrl = `https://crafthead.net/cache/${key}`;
    const response = await caches.default.match(localCacheUrl);
    if (response) {
        return {
            result: await response.json(),
            source: 'cloudflare-local'
        };
    }

    const kvResponse: string | null = await CRAFTHEAD_PROFILE_CACHE.get(key, "text");
    if (kvResponse !== null) {
        if (gatherer) {
            gatherer.push(caches.default.put(localCacheUrl, new Response(JSON.stringify(kvResponse))));
        }
        return {
            result: JSON.parse(kvResponse),
            source: 'cloudflare-kv'
        };
    }

    const remote = await source();
    if (gatherer) {
        const serialized = JSON.stringify(remote);
        gatherer.push(caches.default.put(localCacheUrl, new Response(serialized, {
            headers: {
                'Cache-Control': 'max-age: 86400'
            }
        })));
        gatherer.push(CRAFTHEAD_PROFILE_CACHE.put(key, serialized, {
            expirationTtl: 86400
        }));
    }

    return {
        result: remote,
        source: 'miss'
    };
}