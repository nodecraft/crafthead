export default class ResponseCacheService implements CacheService<Response> {
    private memoryCache: CacheService<ArrayBuffer>;
    private externalCache: CacheService<ArrayBuffer>;

    constructor(memoryCache: CacheService<ArrayBuffer>, externalCache: CacheService<ArrayBuffer>) {
        this.memoryCache = memoryCache;
        this.externalCache = externalCache;
    }

    async find(key: string): Promise<Response | undefined> {
        let memoryResponse = await this.memoryCache.find(key);
        if (memoryResponse) {
            return this.appendHitSource(new Response(memoryResponse), 'memory')
        }

        const externalResponse = await this.externalCache.find(key);
        if (externalResponse) {
            return this.appendHitSource(new Response(externalResponse), 'cloudflare')
        } else {
            return undefined;
        }
    }

    private appendHitSource(response: Response, source: string): Response {
        const headersCopy = new Headers(response.headers);
        headersCopy.set('X-Minehead-Cache-Hit', source);
        return new Response(response.body, {
            status: response.status,
            headers: headersCopy
        });
    }

    async put(key: string, value: Response): Promise<any> {
        const buffer = await value.arrayBuffer();
        return Promise.all([
            this.memoryCache.put(key, buffer),
            this.externalCache.put(key, buffer)
        ]);
    }

    static cleanResponseForCache(response: Response): Response {
        const cleanedHeaders = new Headers(response.headers)
        for (const [key] of response.headers.entries()) {
            if (key !== 'content-type' && key !== 'content-length') {
                if (key === 'x-amz-cf-id') {
                    cleanedHeaders.set('X-Minehead-Cache-Hit', 'miss');
                }
                cleanedHeaders.delete(key);
            }
        }
        return new Response(response.body, {
            status: response.status,
            headers: cleanedHeaders
        });
    }
}