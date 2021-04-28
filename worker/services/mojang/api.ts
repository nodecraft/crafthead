import PromiseGatherer from "../../promise_gather";
import { CacheComputeResult, computeObject } from "../../util/cache-helper";

declare const CRAFTHEAD_PROFILE_CACHE: KVNamespace;

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
        const cacheKey = `username-lookup:${username.toLocaleLowerCase('en-US')}`
        const usernameLookupResult = await computeObject(cacheKey, () => this.delegate.lookupUsername(username, gatherer), gatherer);
        return usernameLookupResult.result;
    }

    async fetchProfile(id: string, gatherer: PromiseGatherer | null): Promise<CacheComputeResult<MojangProfile | null>> {
        const cacheKey = `profile-lookup:${id.toLocaleLowerCase('en-US')}`
        return await computeObject(cacheKey, async () => {
            const result = await this.delegate.fetchProfile(id, gatherer);
            return result.result;
        }, gatherer);
    }
}

// Implements MojangApiService by contacting the Mojang API endpoints directly.
export class DirectMojangApiService implements MojangApiService {
    async lookupUsername(username: string, gatherer: PromiseGatherer | null): Promise<MojangUsernameLookupResult | null> {
        const lookupResponse = await fetch('https://api.mojang.com/profiles/minecraft', {
            method: 'POST',
            body: JSON.stringify([ username ]),
            headers: {
                'Content-Type': 'application/json'
            }
        })

        if (!lookupResponse.ok) {
            if (lookupResponse.status === 400) {
                return null;
            }
            throw new Error(`Unable to lookup UUID from Mojang, http status ${lookupResponse.status}`);
        }

        const contents: MojangUsernameLookupResult[] | undefined = await lookupResponse.json();
        if (typeof contents === 'undefined' || contents.length === 0) {
            return null;
        }
        return contents[0];
    }

    async fetchProfile(id: string, gatherer: PromiseGatherer | null): Promise<CacheComputeResult<MojangProfile | null>> {
        const profileResponse = await fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${id}?unsigned=false`, {
            cf: {
                cacheEverything: true,
                cacheTtl: MOJANG_API_TTL
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