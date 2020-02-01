import LRU from 'lru-cache';

// MemoryCacheService is an implementation of a cache service that keeps its
// contents in memory. This is great for fetching recently-retrieved items
// that happen to land on the same worker. By default, the most recently retrieved
// 1024 items are stored (assuming a generous 4KiB upper bound on responses, this
// is sufficient room for 4MiB worth of responses).
export default class MemoryCacheService<T> implements CacheService<T> {
    private cache: LRU<string, T>;

    constructor() {
        this.cache = new LRU({
            max: 1024
        });
    }

    async find(key: string): Promise<T | undefined> {
        return this.cache.get(key);
    }

    async put(key: string, value: T): Promise<any> {
        this.cache.set(key, value);
    }
}