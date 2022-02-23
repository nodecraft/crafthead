import { BloomFilter } from './bloom-filter';

declare const KV_CACHE_NAMESPACE: KVNamespace;

export enum KVExpiration {
    PERIODIC,
    TIMED,
    NONE
}

export class KVManager {
    static async get(key: string): Promise<string | null> {
        return KVDirect.get(key);
    }

    static async put(key: string, value: string): Promise<void> {
        const seen_before = await BloomFilter.has(key);
        if (seen_before && await KVDirect.get(key) === null) {
            return KVDirect.put(key, value, KVExpiration.PERIODIC);
        }

        return BloomFilter.add(key);
    }
}

export class KVDirect {
    private static namespace = KV_CACHE_NAMESPACE;

    static async get(key: string): Promise<string|null> {
        return this.namespace.get(key);
    }

    static async put(key: string, value: string, expiration = KVExpiration.NONE): Promise<void> {
        switch (expiration) {
            case KVExpiration.PERIODIC:
                return this.namespace.put(key, value, { expirationTtl: 86400 });
            case KVExpiration.TIMED:
                const expirationTime = new Date();
                expirationTime.setUTCDate(new Date().getUTCDate() + 1);
                expirationTime.setHours(0, 0, 0);
                const expirationEpoch = Math.floor(expirationTime.getTime() / 1000);
                return this.namespace.put(key, value, { expiration: expirationEpoch });
            case KVExpiration.NONE:
            default:
                return this.namespace.put(key, value);
        }
    }
}
