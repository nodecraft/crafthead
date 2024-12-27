export async function v3(contents: string): Promise<ArrayBuffer> {
	const textEncoder = new TextEncoder();
	const encoded = textEncoder.encode(contents);

	const result = await crypto.subtle.digest('md5', encoded);

	const uint8View = new Uint8Array(result);
	uint8View[6] &= 0x0F;
	uint8View[6] |= 0x30;
	uint8View[8] &= 0x3F;
	uint8View[8] |= 0x80;
	return result;
}

export function uuidVersion(uuid: ArrayBuffer): number {
	const dv = new DataView(uuid);
	const msb = dv.getBigInt64(0);
	return Number((msb >> BigInt(12)) & BigInt(0x0F));
}

export function toHex(uuid: ArrayBuffer): string {
	return [...new Uint8Array(uuid)].map(bin => bin.toString(16).padStart(2, '0')).join('');
}

export function fromHex(uuid: string): ArrayBuffer {
	const match = uuid.match(/[\da-f]{2}/gi);
	if (!match) {
		throw new TypeError('UUID not provided');
	}
	return new Uint8Array(match.map(hex => Number.parseInt(hex, 16))).buffer;
}

export async function offlinePlayerUuid(username: string): Promise<ArrayBuffer> {
	return v3('OfflinePlayer:' + username);
}

export function javaHashCode(uuid: ArrayBuffer): number {
	const dv = new DataView(uuid);
	const msb = dv.getBigInt64(0);
	const lsb = dv.getBigInt64(8);

	const hilo = msb ^ lsb;
	// eslint-disable-next-line @stylistic/no-mixed-operators
	return Number(BigInt.asIntN(32, hilo >> BigInt(32) ^ hilo));
}
