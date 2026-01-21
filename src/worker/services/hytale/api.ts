import { env } from 'cloudflare:workers';

import type { CacheComputeResult } from '../../util/cache-helper';

export interface HytaleProfile {
	id: string;
	name: string;
	properties: HytaleProfileProperty[];
	skin?: HytaleSkin | null;
}

export interface HytaleProfileProperty {
	name: string;
	value: string;
}

export interface HytaleSkin {
	bodyCharacteristic: string;
	underwear: string;
	face: string;
	ears: string;
	mouth: string;
	haircut: string;
	facialHair: string | null;
	eyebrows: string;
	eyes: string;
	pants: string | null;
	overpants: string | null;
	undertop: string | null;
	overtop: string | null;
	shoes: string | null;
	headAccessory: string | null;
	faceAccessory: string | null;
	earAccessory: string | null;
	skinFeature: string | null;
	gloves: string | null;
	cape: string | null;
}

export interface PlayerDBHytaleProfileData {
	player: {
		meta: {
			cached_at: number;
		};
		username: string;
		id: string;
		raw_id: string;
		avatar: string;
		skin: HytaleSkin | null;
		properties: HytaleProfileProperty[];
	};
}

export interface PlayerDBHytaleProfile {
	code: string;
	message: string;
	success: boolean;
	data: PlayerDBHytaleProfileData;
}

const PlayerDBHeaders = {
	'Content-Type': 'application/json',
	'User-Agent': 'Crafthead (+https://crafthead.net)',
};

/**
 * Looks up a username and returns the full profile (including properties/textures).
 * This avoids needing a separate fetchProfile call since PlayerDB returns everything.
 */
export async function lookupUsername(
	request: Request,
	username: string,
): Promise<HytaleProfile | null> {
	let lookupResponse: Response;
	if (env.PLAYERDB) {
		const playerDbRequest = new Request(`https://playerdb.co/api/player/hytale/${username}`, {
			headers: PlayerDBHeaders,
			cf: request?.cf,
			signal: AbortSignal.timeout(5000),
		});
		lookupResponse = await env.PLAYERDB.fetch(playerDbRequest);
	} else {
		lookupResponse = await fetch(`https://playerdb.co/api/player/hytale/${username}`, {
			headers: PlayerDBHeaders,
			cf: request?.cf,
			signal: AbortSignal.timeout(5000),
		});
	}

	let jsonData: PlayerDBHytaleProfile | null = null;
	try {
		jsonData = await lookupResponse.json();
	} catch {
		// ignore
	}
	if (lookupResponse.status === 204 || jsonData?.code === 'hytale.invalid_username') {
		return null;
	} else if (lookupResponse.status === 400 && jsonData?.code === 'hytale.not_found') {
		return null;
	} else if (!lookupResponse.ok || !jsonData) {
		throw new Error('Unable to lookup UUID for username, http status ' + lookupResponse.status);
	} else {
		const returnedProfile = jsonData.data?.player;

		return {
			id: returnedProfile.raw_id,
			name: returnedProfile.username,
			properties: returnedProfile.properties,
			skin: returnedProfile.skin,
		};
	}
}

export async function fetchProfile(
	request: Request,
	id: string,
): Promise<CacheComputeResult<HytaleProfile | null>> {
	let profileResponse: Response;
	if (env.PLAYERDB) {
		const playerDbRequest = new Request(`https://playerdb.co/api/player/hytale/${id}`, {
			headers: PlayerDBHeaders,
			cf: request?.cf,
			signal: AbortSignal.timeout(5000),
		});
		profileResponse = await env.PLAYERDB.fetch(playerDbRequest);
	} else {
		profileResponse = await fetch(`https://playerdb.co/api/player/hytale/${id}`, {
			headers: PlayerDBHeaders,
			cf: request?.cf,
			signal: AbortSignal.timeout(5000),
		});
	}
	let jsonData: PlayerDBHytaleProfile | null = null;
	try {
		jsonData = await profileResponse.json();
	} catch {
		// ignore
	}
	if (jsonData && profileResponse.status === 200) {
		const returnedProfile = jsonData.data?.player;

		const data = {
			id: returnedProfile.raw_id,
			name: returnedProfile.username,
			properties: returnedProfile.properties,
			skin: returnedProfile.skin,
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
	} else if (jsonData && jsonData.code === 'hytale.invalid_username') {
		return {
			result: null,
			source: 'miss',
		};
	}
	throw new Error(`Unable to retrieve profile from Hytale API, http status ${profileResponse.status}`);
}
