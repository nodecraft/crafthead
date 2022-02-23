import PromiseGatherer from '../promise_gather';
import { getWASMModule } from '../wasm';
import { KVDirect, KVExpiration } from './kv-manager';

const ALLOCATED_ROWS = 10;
const K_HASHES = 3;

export class BloomFilter {
    private static seed = 8149214274; // Randomly chosen, nothing special
    private static m = ALLOCATED_ROWS;
    private static k = K_HASHES;

    static async add(element: string): Promise<void> {
        if (await KVDirect.get('bloom:0') === null) { // Check if Bloom filter exists in KV
            await this.allocate();
        }

        const gatherer = new PromiseGatherer();

        const indexes = await this.getIndexes(element);
        for (const index of indexes) {
            gatherer.push(KVDirect.put('bloom:' + index, '1', KVExpiration.TIMED));
        }

        return gatherer.all();
    }

    static async has(element: string): Promise<boolean> {
        if (await KVDirect.get('bloom:0') === null) { // Check if Bloom filter exists in KV
            await this.allocate();
            return false;
        }

        const indexes = await this.getIndexes(element);
        for (const index of indexes) {
            if (await KVDirect.get('bloom:' + index) === '0') {
                return false;
            }
        }
        return true;
    }

    // See https://willwhim.wpengine.com/2011/09/03/producing-n-hash-functions-by-hashing-only-once/
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
                KVDirect.put('bloom:' + i, '0', KVExpiration.TIMED)
            );
        }

        return gatherer.all();
    }
}
