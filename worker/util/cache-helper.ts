export interface CacheComputeResult<T> {
    result: T;
    source: string;
}

declare const CRAFTHEAD_PROFILE_CACHE: KVNamespace;