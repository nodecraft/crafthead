import { env } from 'cloudflare:workers';

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

const PlayerDBHeaders = {
	'Content-Type': 'application/json',
	'User-Agent': 'Crafthead (+https://crafthead.net)',
};

export async function lookupUsername(
	request: Request,
	username: string,
): Promise<MojangUsernameLookupResult | null> {
	let lookupResponse: Response;
	if (env.PLAYERDB) {
		const playerDbRequest = new Request(`https://playerdb.co/api/player/minecraft/${username}`, {
			headers: PlayerDBHeaders,
			cf: request?.cf,
			signal: AbortSignal.timeout(5000),
		});
		lookupResponse = await env.PLAYERDB.fetch(playerDbRequest);
	} else {
		lookupResponse = await fetch(`https://playerdb.co/api/player/minecraft/${username}`, {
			headers: PlayerDBHeaders,
			cf: request?.cf,
			signal: AbortSignal.timeout(5000),
		});
	}

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

export async function fetchProfile(
	request: Request,
	id: string,
): Promise<CacheComputeResult<MojangProfile | null>> {
	let profileResponse: Response;
	if (env.PLAYERDB) {
		const playerDbRequest = new Request(`https://playerdb.co/api/player/minecraft/${id}`, {
			headers: PlayerDBHeaders,
			cf: request?.cf,
			signal: AbortSignal.timeout(5000),
		});
		profileResponse = await env.PLAYERDB.fetch(playerDbRequest);
	} else {
		profileResponse = await fetch(`https://playerdb.co/api/player/minecraft/${id}`, {
			headers: PlayerDBHeaders,
			cf: request?.cf,
			signal: AbortSignal.timeout(5000),
		});
	}
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
			source: returnedProfile?.meta?.cached_at ? 'hit' : 'miss',
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
