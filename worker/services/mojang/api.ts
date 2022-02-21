import PromiseGatherer from "../../promise_gather";
import { CacheComputeResult } from "../../util/cache-helper";
import { default as CACHE_BUST } from '../../util/cache-bust';
import {KVExpiration, KVManager} from '../../util/kv-manager';

export interface MojangProfile {
    id: string;
    name: string;
    properties: MojangProfileProperty[];
}

export interface MojangUsernameLookupResult {
    id: string;
    name: string;
}

export interface MojangProfileProperty {
    name: string;
    value: string;
}

export interface MojangApiService {
    lookupUsername(usernames: string, gatherer: PromiseGatherer | null): Promise<MojangUsernameLookupResult | null>;

    fetchProfile(id: string, gatherer: PromiseGatherer | null): Promise<CacheComputeResult<MojangProfile | null>>;
}

// Implements MojangApiService by caching results and forwarding to Mojang when needed.
export class CachedMojangApiService implements MojangApiService {
    private delegate: MojangApiService;

    public constructor(delegate: MojangApiService) {
        this.delegate = delegate;
    }

    async lookupUsername(username: string, gatherer: PromiseGatherer | null): Promise<MojangUsernameLookupResult | null> {
        const lowercased = username.toLocaleLowerCase('en-US');
        const localCacheKey = 'https://crafthead.net/__internal' + CACHE_BUST + '/username-lookup/' + lowercased;

        const localCacheResult = await caches.default.match(new Request(localCacheKey));
        if (localCacheResult && localCacheResult.ok) {
            return localCacheResult.json();
        }

        const kvResult = await KVManager.get('username-lookup:' + lowercased);
        if (kvResult) {
            gatherer?.push(caches.default.put(new Request(localCacheKey), new Response(
                kvResult, { headers: { 'Cache-Control': 'max-age=3600', 'Content-Type': 'application/json' }}
            )));
            return JSON.parse(kvResult);
        }

        const lookup = await this.delegate.lookupUsername(lowercased, gatherer);
        if (lookup) {
            gatherer?.push(
                KVManager.put(
                    'username-lookup:' + lowercased,
                    JSON.stringify(lookup)
                )
            );
            gatherer?.push(caches.default.put(new Request(localCacheKey), new Response(
                JSON.stringify(lookup), { headers: { 'Cache-Control': 'max-age=3600', 'Content-Type': 'application/json' }}
            )));
        }

        return lookup;
    }

    async fetchProfile(id: string, gatherer: PromiseGatherer | null): Promise<CacheComputeResult<MojangProfile | null>> {
        const kvResult = await KVManager.get('profile-lookup:' + id);
        if (kvResult !== null) {
            return {
                result: JSON.parse(kvResult),
                source: 'cf-kv'
            };
        }

        const lookup = await this.delegate.fetchProfile(id, gatherer);
        if (lookup) {
            gatherer?.push(
                KVManager.put(
                    'profile-lookup:' + id,
                    JSON.stringify(lookup.result)
                )
            );
        }

        return lookup;
    }
}

// Implements MojangApiService by contacting the Mojang API endpoints directly.
export class DirectMojangApiService implements MojangApiService {
    async lookupUsername(username: string, gatherer: PromiseGatherer | null): Promise<MojangUsernameLookupResult | null> {
        const lookupResponse = await fetch(`https://api.mojang.com/users/profiles/minecraft/${username}`, {
            headers: {
                'Content-Type': 'application/json',
                'User-Agent': 'Crafthead (+https://crafthead.net)'
            }
        });

        if (lookupResponse.status === 204) {
            return null;
        } else if (!lookupResponse.ok) {
            throw new Error('Unable to lookup UUID for username, http status ' + lookupResponse.status);
        } else {
            const contents: MojangUsernameLookupResult | undefined = await lookupResponse.json();
            if (typeof contents === 'undefined') {
                return null;
            }
            return contents;
        }
    }

    async fetchProfile(id: string, gatherer: PromiseGatherer | null): Promise<CacheComputeResult<MojangProfile | null>> {
        const profileResponse = await fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${id}?unsigned=false`, {
            headers: {
                'User-Agent': 'Crafthead (+https://crafthead.net)'
            }
        });

        if (profileResponse.status === 200) {
            return {
                result: await profileResponse.json(),
                source: 'miss'
            };
        } else if (profileResponse.status === 206 || profileResponse.status === 204) {
            return {
                result: null,
                source: 'miss'
            };
        } else {
            throw new Error(`Unable to retrieve profile from Mojang, http status ${profileResponse.status}`);
        }
    }
}