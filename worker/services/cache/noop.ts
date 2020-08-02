// NoopCacheService is an implementation of a cache service that loses everything :)
export default class NoopCacheService<T> implements CacheService<T> {
    async find(key: string): Promise<T | undefined> {
        return undefined
    }

    async put(key: string, value: T): Promise<any> {
    }
}