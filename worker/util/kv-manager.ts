import PromiseGatherer from '../promise_gather';
import xxhash from 'xxhash-wasm';

declare const KV_CACHE_NAMESPACE: KVNamespace;

export enum KVExpiration {
    PERIODIC,
    TIMED,
    NONE
}

const ALLOCATED_ROWS = 10;
const K_HASHES = 3;

class BloomFilter {
    private static seed = 8149214274; // Randomly chosen, nothing special
    private static m = ALLOCATED_ROWS;
    private static k = K_HASHES;

    public static async add(element: string): Promise<void> {
        if (await KVManager.get('bloom:0') == null) {
            await this.allocate();
        }

        const gatherer = new PromiseGatherer();

        const indexes = this.getIndexes(element);
        for (const index in indexes) {
            gatherer.push(KVManager.put_direct('bloom:' + index, '1', KVExpiration.TIMED));
        }

        return gatherer.all();
    }

    public static async has(element: string): Promise<boolean> {
        if (await KVManager.get('bloom:0') == null) {
            await this.allocate();
            return false;
        }

        const indexes = this.getIndexes(element);
        for (const index in indexes) {
            if (await KVManager.get('bloom:' + index) == '0') {
                return false;
            }
        }
        return true;
    }

    // See: https://willwhim.wpengine.com/2011/09/03/producing-n-hash-functions-by-hashing-only-once/
    private static async getIndexes(element: string): Promise<number[]> {
        const [a, b] = await this.doubleHash(element);
        let indexes = [];
        for (let i = 0; i < this.k; i++) {
            indexes.push((a + b * i) % this.m);
        }
        return indexes;
    }

    private static async doubleHash(value: string): Promise<number[]> {
        const { h32 } = await xxhash();
        return [ h32(value, this.seed), h32(value, this.seed + 1) ];
    }

    private static async allocate(): Promise<void> {
        const gatherer = new PromiseGatherer();

        for (let i = 0; i < this.m; i++) {
            gatherer.push(
                KVManager.put_direct('bloom:' + i, '0', KVExpiration.TIMED)
            );
        }

        return gatherer.all();
    }
}

export class KVManager {
    private static namespace = KV_CACHE_NAMESPACE;

    public static async get(key: string): Promise<string|null> {
        const seen_before = await BloomFilter.has(key);
        if (!seen_before) {
            return null;
        }

        const { value, metadata } = await this.namespace.getWithMetadata(key);
        console.log(metadata);

        if (value !== null) {
            await BloomFilter.add(key);
            return value;
        }

        return null;
    }

    public static async put(key: string, value: string): Promise<void> {
        const seen_before = await BloomFilter.has(key);
        if (!seen_before) {
            return;
        }

        return this.put_direct(key, value, KVExpiration.PERIODIC);
    }

    public static async put_direct(key: string, value: string, expiration = KVExpiration.NONE): Promise<void> {
        switch (expiration) {
            case KVExpiration.PERIODIC:
                return this.namespace.put(key, value, { expirationTtl: 86400 });
            case KVExpiration.TIMED:
                const expirationEpoch = Math.floor(new Date().getTime() / 1000) + 86400;
                return this.namespace.put(key, value, { expiration: expirationEpoch });
            case KVExpiration.NONE:
            default:
                return this.namespace.put(key, value);
        }
    }
}
