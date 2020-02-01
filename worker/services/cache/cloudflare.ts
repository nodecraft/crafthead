// CloudflareCacheService is an implementation of a cache service that
// uses the Cache API of the Cloudflare Worker.
const BASE_URL = `https://crafthead.net/`

interface CloudflareResponseMapper<T> {
    decode(response: Response): Promise<T>;
    encode(object: T): Response;
}

export class CloudflareCacheService<T> implements CacheService<T> {
    private key: string;
    private mapper: CloudflareResponseMapper<T>;

    constructor(key: string, mapper: CloudflareResponseMapper<T>) {
        this.key = key;
        this.mapper = mapper;
    }

    async find(key: string): Promise<T | undefined> {
        return caches.default.match(`${BASE_URL}/${this.key}/${key}`)
            .then(response => {
                if (response) {
                    return this.mapper.decode(response);
                }
            });
    }

    async put(key: string, value: T): Promise<any> {
        return caches.default.put(`${BASE_URL}/${this.key}/${key}`, this.mapper.encode(value));
    }
}

export class ResponseCloudflareResponseMapper implements CloudflareResponseMapper<Response> {
    decode(response: Response): Promise<Response> {
        return Promise.resolve(response)
    }

    encode(object: Response): Response {
        return object
    }
}

export class ArrayBufferCloudflareResponseMapper implements CloudflareResponseMapper<ArrayBuffer> {
    decode(response: Response): Promise<ArrayBuffer> {
        return response.arrayBuffer();
    }

    encode(object: ArrayBuffer): Response {
        return new Response(object, {
            headers: {
                'Cache-Control': 'public, max-age=86400'
            }
        })
    }
}