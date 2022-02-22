import PromiseGatherer from '../promise_gather';
import { getWASMModule } from '../wasm';

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
        if (await KVManager.getDirect('bloom:0') === null) {
            await this.allocate();
        }

        const gatherer = new PromiseGatherer();

        const indexes = await this.getIndexes(element);
        for (const index of indexes) {
            gatherer.push(KVManager.putDirect('bloom:' + index, '1', KVExpiration.TIMED));
        }

        return gatherer.all();
    }

    public static async has(element: string): Promise<boolean> {
        if (await KVManager.getDirect('bloom:0') === null) {
            await this.allocate();
            return false;
        }

        const indexes = await this.getIndexes(element);
        for (const index of indexes) {
            if (await KVManager.getDirect('bloom:' + index) === '0') {
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
        const wasm = await getWASMModule();
        return [ Number(wasm.xxhash(value, this.seed)), Number(wasm.xxhash(value, this.seed + 1)) ];
    }

    private static async allocate(): Promise<void> {
        const gatherer = new PromiseGatherer();

        for (let i = 0; i < this.m; i++) {
            gatherer.push(
                KVManager.putDirect('bloom:' + i, '0', KVExpiration.TIMED)
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

        return this.getDirect(key);
    }

    public static async put(key: string, value: string): Promise<void> {
        const seen_before = await BloomFilter.has(key);
        if (seen_before && await this.getDirect(key) === null) {
            return this.putDirect(key, value, KVExpiration.PERIODIC);
        }

        return BloomFilter.add(key);
    }

    public static async getDirect(key: string): Promise<string|null> {
        return this.namespace.get(key);
    }

    public static async putDirect(key: string, value: string, expiration = KVExpiration.NONE): Promise<void> {
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
