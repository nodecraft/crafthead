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
// Implements MojangApiService by contacting the Mojang API endpoints directly.
export class DirectMojangApiService implements MojangApiService {
	async lookupUsername(username: string, gatherer: PromiseGatherer | null): Promise<MojangUsernameLookupResult | null> {
		const lookupResponse = await fetch(`https://playerdb.co/api/player/minecraft/${username}`, {
			headers: {
				'Content-Type': 'application/json',
				'User-Agent': 'Crafthead (+https://crafthead.net)',
			},
		});

		let jsonData: PlayerDBProfile | null = null;
		try {
			jsonData = await lookupResponse.json();
		} catch {
			// ignore
		}
		if (lookupResponse.status === 204 || jsonData?.code === 'minecraft.invalid_username') {
			return null;
		} else if (!lookupResponse.ok || !jsonData) {
			throw new Error('Unable to lookup UUID for username, http status ' + lookupResponse.status);
		} else {
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
		let jsonData: PlayerDBProfile | null = null;
		try {
			jsonData = await profileResponse.json();
		} catch {
			// ignore
		}
		if (jsonData && profileResponse.status === 200) {
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
		} else if (jsonData && jsonData.code === 'minecraft.invalid_username') {
			return {
				result: null,
				source: 'miss',
			};
		}
		throw new Error(`Unable to retrieve profile from Mojang, http status ${profileResponse.status}`);
	}
}
