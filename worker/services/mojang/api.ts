import { default as CACHE_BUST } from '../../util/cache-bust';

import type PromiseGatherer from '../../promise_gather';
import type { Env } from '../../types';
import type { CacheComputeResult } from '../../util/cache-helper';

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

export interface PlayerDBProfileData {
	player: {
		meta: {
			cached_at: number;
		};
		username: string;
		id: string;
		raw_id: string;
		avatar: string;
		name_history: string[];
		skin_texture: string;
		properties: MojangProfileProperty[];
	};
}

export interface PlayerDBProfile {
	code: string;
	message: string;
	success: boolean;
	data: PlayerDBProfileData;
}

export interface MojangApiService {
	lookupUsername(usernames: string, gatherer: PromiseGatherer | null): Promise<MojangUsernameLookupResult | null>;

	fetchProfile(id: string, gatherer: PromiseGatherer | null): Promise<CacheComputeResult<MojangProfile | null>>;
}

// Implements MojangApiService by caching results and forwarding to Mojang when needed.
export class CachedMojangApiService implements MojangApiService {
	private delegate: MojangApiService;
	private env: Env;

	public constructor(delegate: MojangApiService, env: Env) {
		this.delegate = delegate;
		this.env = env;
	}

	async lookupUsername(username: string, gatherer: PromiseGatherer | null): Promise<MojangUsernameLookupResult | null> {
		const lowercased = username.toLocaleLowerCase('en-US');
		const localCacheKey = 'https://crafthead.net/__internal' + CACHE_BUST + '/username-lookup/' + lowercased;

		const localCacheResult = await caches.default.match(new Request(localCacheKey));
		if (localCacheResult && localCacheResult.ok) {
			return localCacheResult.json();
		}

		const kvKey = CACHE_BUST + ':username-lookup:' + lowercased;

		const kvResult: MojangUsernameLookupResult | null = await this.env.CRAFTHEAD_PROFILE_CACHE?.get(kvKey, 'json') ?? null;
		if (kvResult) {
			gatherer?.push(caches.default.put(new Request(localCacheKey), new Response(
				JSON.stringify(kvResult), { headers: { 'Cache-Control': 'max-age=3600', 'Content-Type': 'application/json' } },
			)));
			return kvResult;
		}

		const lookup = await this.delegate.lookupUsername(lowercased, gatherer);
		if (lookup) {
			gatherer?.push(
				this.env.CRAFTHEAD_PROFILE_CACHE.put(
					kvKey,
					JSON.stringify(lookup),
					{ expirationTtl: 86400 },
				),
			);
			gatherer?.push(caches.default.put(new Request(localCacheKey), new Response(
				JSON.stringify(lookup), { headers: { 'Cache-Control': 'max-age=3600', 'Content-Type': 'application/json' } },
			)));
		}

		return lookup;
	}

	async fetchProfile(id: string, gatherer: PromiseGatherer | null): Promise<CacheComputeResult<MojangProfile | null>> {
		const kvKey = CACHE_BUST + ':profile-lookup:' + id;
		const kvResult: MojangProfile | null = await this.env?.CRAFTHEAD_PROFILE_CACHE?.get(kvKey, 'json') ?? null;
		if (kvResult !== null) {
			return {
				result: kvResult,
				source: 'cf-kv',
			};
		}

		const lookup = await this.delegate.fetchProfile(id, gatherer);
		if (lookup) {
			gatherer?.push(
				this.env.CRAFTHEAD_PROFILE_CACHE.put(
					kvKey,
					JSON.stringify(lookup.result),
					{ expirationTtl: 86400 },
				),
			);
		}

		return lookup;
	}
}

// Implements MojangApiService by contacting the Mojang API endpoints directly.
export class DirectMojangApiService implements MojangApiService {
	async lookupUsername(username: string, gatherer: PromiseGatherer | null): Promise<MojangUsernameLookupResult | null> {
		const lookupResponse = await fetch(`https://playerdb.co/api/player/minecraft/${username}`, {
			headers: {
				'Content-Type': 'application/json',
				'User-Agent': 'Crafthead (+https://crafthead.net)',
			},
		});

		if (lookupResponse.status === 204) {
			return null;
		} else if (!lookupResponse.ok) {
			throw new Error('Unable to lookup UUID for username, http status ' + lookupResponse.status);
		} else {
			const jsonData: PlayerDBProfile = await lookupResponse.json();
			const returnedProfile = jsonData.data?.player;

			// Now we need to mangle this data into the format we expect.
			const data = {
				id: returnedProfile.raw_id,
				name: returnedProfile.username,
			};
			if (data === undefined) {
				return null;
			}
			return data;
		}
	}

	async fetchProfile(id: string, gatherer: PromiseGatherer | null): Promise<CacheComputeResult<MojangProfile | null>> {
		const profileResponse = await fetch(`https://playerdb.co/api/player/minecraft/${id}`, {
			headers: {
				'User-Agent': 'Crafthead (+https://crafthead.net)',
			},
		});

		if (profileResponse.status === 200) {
			const jsonData: PlayerDBProfile = await profileResponse.json();
			const returnedProfile = jsonData.data?.player;

			// Now we need to mangle this data into the format we expect.
			const data = {
				id: returnedProfile.raw_id,
				name: returnedProfile.username,
				properties: returnedProfile.properties,
			};
			return {
				result: data,
				source: 'miss',
			};
		} else if (profileResponse.status === 206 || profileResponse.status === 204) {
			return {
				result: null,
				source: 'miss',
			};
		}
		throw new Error(`Unable to retrieve profile from Mojang, http status ${profileResponse.status}`);
	}
}
