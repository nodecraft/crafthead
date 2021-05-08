import PromiseGatherer from "../../promise_gather";
import { CacheComputeResult, computeObject } from "../../util/cache-helper";

const MOJANG_API_TTL = 86400

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
        const cacheKey = `username-lookup:${lowercased}`;
        const usernameLookupResult = await computeObject(cacheKey, () => this.delegate.lookupUsername(lowercased, gatherer), gatherer);
        return usernameLookupResult.result;
    }

    async fetchProfile(id: string, gatherer: PromiseGatherer | null): Promise<CacheComputeResult<MojangProfile | null>> {
        const lowercased = id.toLocaleLowerCase('en-US');
        const cacheKey = `profile-lookup:${lowercased}`;
        return await computeObject(cacheKey, async () => {
            const result = await this.delegate.fetchProfile(lowercased, gatherer);
            return result.result;
        }, gatherer);
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