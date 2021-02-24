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
    lookupUsername(usernames: string[]): Promise<MojangUsernameLookupResult | null>;

    fetchProfile(id: string): Promise<MojangProfile | null>;
}

// Implements MojangApiService by contacting the Mojang API endpoints directly. Only basic caching done by
// Cloudflare itself is implemented.
export class DirectMojangApiService implements MojangApiService {
    async lookupUsername(usernames: string[]): Promise<MojangUsernameLookupResult | null> {
        const lookupResponse = await fetch('https://api.mojang.com/profiles/minecraft', {
            method: 'POST',
            body: JSON.stringify(usernames),
            headers: {
                'Content-Type': 'application/json'
            },
            cf: {
                cacheEverything: true,
                cacheTtl: MOJANG_API_TTL
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

    async fetchProfile(id: string): Promise<MojangProfile | null> {
        const profileResponse = await fetch(`https://sessionserver.mojang.com/session/minecraft/profile/${id}?unsigned=false`, {
            cf: {
                cacheEverything: true,
                cacheTtl: MOJANG_API_TTL
            }
        });

        if (profileResponse.status === 200) {
            return await profileResponse.json();
        } else if (profileResponse.status === 206) {
            return null;
        } else {
            throw new Error(`Unable to retrieve profile from Mojang, http status ${profileResponse.status}`);
        }
    }
}