/* eslint-disable no-restricted-syntax */

// Game determines which game the request is for.
export enum Game {
	Minecraft = 'minecraft',
	Hytale = 'hytale',
}

// RequestedKind determines the kind of request the user is making.
export enum RequestedKind {
	Skin,
	Avatar,
	Helm,
	Cube,
	Body,
	Bust,
	Cape,
	Profile,
}

// IdentityKind determines if the request is for a UUID or a username.
export enum IdentityKind {
	Uuid,
	Username,
	TextureID,
}

export enum TextureKind {
	SKIN,
	CAPE,
}

/* eslint-enable no-restricted-syntax */

export interface CraftheadRequest {
	game: Game;
	requested: RequestedKind;
	requestedKindString: string;
	identity: string;
	identityType: IdentityKind;
	size: number;
	armored: boolean;
	model: string | null;
}

function stringKindToRequestedKind(kind: string): RequestedKind | null {
	switch (kind) {
		case 'skin': {
			return RequestedKind.Skin;
		}
		case 'avatar': {
			return RequestedKind.Avatar;
		}
		case 'cube': {
			return RequestedKind.Cube;
		}
		case 'helm': {
			return RequestedKind.Helm;
		}
		case 'body': {
			return RequestedKind.Body;
		}
		case 'bust': {
			return RequestedKind.Bust;
		}
		case 'cape': {
			return RequestedKind.Cape;
		}
		case 'profile': {
			return RequestedKind.Profile;
		}
		default: {
			return null;
		}
	}
}

export function identityTypeEnumToString(kind: IdentityKind): string {
	switch (kind) {
		case IdentityKind.Uuid: {
			return 'uuid';
		}
		case IdentityKind.Username: {
			return 'username';
		}
		case IdentityKind.TextureID: {
			return 'textureID';
		}
		default: {
			return 'unknown';
		}
	}
}

export function interpretRequest(request: Request): CraftheadRequest | null {
	const url = new URL(request.url);
	let pathname = url.pathname;
	if (pathname.endsWith('.png')) {
		pathname = pathname.slice(0, -4);
	}

	let model = url.searchParams.get('model');
	if (model && !['slim', 'default'].includes(model)) {
		model = null;
	}

	// Parse game prefix from URL
	let game = Game.Minecraft; // Default to Minecraft for legacy URLs
	if (pathname.startsWith('/minecraft/')) {
		game = Game.Minecraft;
		pathname = pathname.slice('/minecraft'.length); // Keep the leading /
	} else if (pathname.startsWith('/hytale/')) {
		game = Game.Hytale;
		pathname = pathname.slice('/hytale'.length); // Keep the leading /
	}

	let armored = false;
	let sliceAmt = 1;

	if (pathname.includes('/armor/cube/') || pathname.includes('/armor/body/') || pathname.includes('/armor/bust/')) {
		armored = true;
		sliceAmt = 2;
	}

	// eslint-disable-next-line prefer-const
	let [requestedKindString, identity, sizeString] = pathname.split('/').slice(sliceAmt);
	if (!identity) {
		return null;
	}

	let size = Number.parseInt(sizeString, 10);
	if (!size) {
		size = 180; // default, same as Minotar
	} else if (size < 8) {
		size = 8; // minimum size
	} else if (size > 300) {
		// In order to limit abuse, don't scale above 300px.
		size = 300;
	}

	const requested = stringKindToRequestedKind(requestedKindString);
	if (requested === null) {
		return null;
	}

	let identityType: IdentityKind;
	if (identity.length <= 16) {
		identityType = IdentityKind.Username;
	} else if (identity.length === 32) {
		identityType = IdentityKind.Uuid;
	} else if (identity.length === 36) {
		identity = identity.replaceAll('-', '');
		identityType = IdentityKind.Uuid;
	} else if (identity.length > 16 && identity.length <= 90) {
		// Validate hex characters first
		if (!/^[\da-f]+$/i.test(identity)) {
			console.error(`Invalid texture ID format: contains non-hexadecimal characters. Identity: ${identity.slice(0, 20)}...`);
			return null;
		}

		// Handle texture IDs with any number of leading zeros
		// Mojang stores these hashes with leading zeros already stripped, so we need to
		// normalize user input (which may have 0-N leading zeros) to match the stored format
		// Accept range of 17-90 chars (SHA-256 is 64 chars + up to 26 leading zeros)
		identity = identity.replace(/^0+/, '') || '0';

		// Validate that the normalized hash is at most 64 chars (SHA-256 hash length)
		if (identity.length > 64) {
			console.error(`Invalid texture ID: normalized length ${identity.length} exceeds 64 characters`);
			return null;
		}

		identityType = IdentityKind.TextureID;
	} else {
		return null;
	}

	return { game, requested, requestedKindString, identityType, identity, size, armored, model };
}
