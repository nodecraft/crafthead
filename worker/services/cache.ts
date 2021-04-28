interface CacheService<T> {
    find(key: string): Promise<T | undefined>;
    put(key: string, value: T): Promise<any>;
    name(): string;
}