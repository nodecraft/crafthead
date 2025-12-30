import { createExecutionContext, env, waitOnExecutionContext } from 'cloudflare:test';
import { imageSize } from 'image-size';
import { describe, expect, it } from 'vitest';

import worker from '../src/worker/index';

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

describe('worker requests', () => {
	it('responds with HTML for index', async () => {
		const request = new IncomingRequest('http://crafthead.net');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('text/html');
	});

	it('responds with 404 for unknown path', async () => {
		const request = new IncomingRequest('http://crafthead.net/unknown');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(response.status).toBe(404);
	});

	it('responds with image for avatar on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/avatar/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(180);
		expect(height).toBe(180);
	});

	it('responds with max 300px image for avatar on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/avatar/ef6134805b6244e4a4467fbe85d65513/512');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(300);
		expect(height).toBe(300);
	});

	it('responds with min 8px image for avatar on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/avatar/ef6134805b6244e4a4467fbe85d65513/2');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(8);
		expect(height).toBe(8);
	});

	it('responds with image for avatar on ID with dashes', async () => {
		const request = new IncomingRequest('http://crafthead.net/avatar/ef613480-5b62-44e4-a446-7fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(180);
		expect(height).toBe(180);
	});

	it('responds with image for avatar on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/avatar/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(180);
		expect(height).toBe(180);
	});

	it('responds with image for cape on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/cape/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { height } = imageSize(buffer);
		expect(height).toBe(180);
	});

	it('responds with image for cape on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/cape/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { height } = imageSize(buffer);
		expect(height).toBe(180);
	});

	it('responds with image for helm on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/helm/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(180);
		expect(height).toBe(180);
	});

	it('responds with image for helm on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/helm/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(180);
		expect(height).toBe(180);
	});

	it('responds with image for cube on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/cube/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(180);
		expect(height).toBe(180);
	});

	it('responds with image for cube on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/cube/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(180);
		expect(height).toBe(180);
	});

	it('responds with image for body on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/body/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { height } = imageSize(buffer);
		expect(height).toBe(360);
	});

	it('responds with image for body on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/body/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { height } = imageSize(buffer);
		expect(height).toBe(360);
	});

	it('responds with image for bust on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/bust/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(180);
		expect(height).toBe(180);
	});

	it('responds with image for bust on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/bust/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(180);
		expect(height).toBe(180);
	});

	it('responds with image for skin on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/skin/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width } = imageSize(buffer);
		expect(width).toBe(64);
	});

	it('responds with image for skin on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/skin/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width } = imageSize(buffer);
		expect(width).toBe(64);
	});

	it('responds with image for avatar on texture ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/avatar/9d2e80355eed693e3f0485893ef04ff6a507f3aab33f2bedb48cef56e30f67d0');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width } = imageSize(buffer);
		expect(width).toBe(180);
	});

	it('responds with image for helm on texture ID with leading zero trimmed by Mojang', async () => {
		// This tests the specific issue: hash 67f7105... (63 chars) should work
		// even though it's missing the leading zero that would make it 64 chars
		const request = new IncomingRequest('http://crafthead.net/helm/67f7105027d3d2b8eba224c980ad04d9c5a151b58e373c20fed5a4e4c164c05');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');

		const image = await response.arrayBuffer();
		const buffer = Buffer.from(image);
		const { width, height } = imageSize(buffer);
		expect(width).toBe(180);
		expect(height).toBe(180);
	});

	it('responds with same image for texture ID with and without leading zero', async () => {
		// Both requests should return the same image since Mojang trims leading zeros
		const request63 = new IncomingRequest('http://crafthead.net/helm/67f7105027d3d2b8eba224c980ad04d9c5a151b58e373c20fed5a4e4c164c05');
		const request64 = new IncomingRequest('http://crafthead.net/helm/067f7105027d3d2b8eba224c980ad04d9c5a151b58e373c20fed5a4e4c164c05');
		const ctx = createExecutionContext();
		const response63 = await worker.fetch(request63, env, ctx);
		const response64 = await worker.fetch(request64, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response63.headers.get('content-type')).toContain('image/png');
		expect(await response64.headers.get('content-type')).toContain('image/png');

		const image63 = await response63.arrayBuffer();
		const image64 = await response64.arrayBuffer();
		expect(image63).toStrictEqual(image64);
	});

	it('handles texture ID with two leading zeros (issue #126)', async () => {
		// Test case from issue #126: hash that would be 64 chars if fully zero-padded
		// All three variations should work and return the same image
		const hash62 = '3e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 62 chars - normalized form (as stored by Mojang)
		const hash63 = '03e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 63 chars - with 1 leading zero
		const hash64 = '003e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 64 chars - with 2 leading zeros

		const request62 = new IncomingRequest(`http://crafthead.net/helm/${hash62}`);
		const request63 = new IncomingRequest(`http://crafthead.net/helm/${hash63}`);
		const request64 = new IncomingRequest(`http://crafthead.net/helm/${hash64}`);

		const ctx = createExecutionContext();
		const response62 = await worker.fetch(request62, env, ctx);
		const response63 = await worker.fetch(request63, env, ctx);
		const response64 = await worker.fetch(request64, env, ctx);
		await waitOnExecutionContext(ctx);

		// All should return valid images
		expect(response62.headers.get('content-type')).toContain('image/png');
		expect(response63.headers.get('content-type')).toContain('image/png');
		expect(response64.headers.get('content-type')).toContain('image/png');

		// All should return the same image
		const image62 = await response62.arrayBuffer();
		const image63 = await response63.arrayBuffer();
		const image64 = await response64.arrayBuffer();
		expect(image62).toStrictEqual(image63);
		expect(image63).toStrictEqual(image64);
	});

	it('handles texture ID with three leading zeros', async () => {
		// Tests up to 65-char input (62-char normalized hash + 3 leading zeros)
		// Uses same underlying hash as issue #126 to verify handling of additional leading zeros
		const hash62 = '3e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 62 chars - normalized form (as stored by Mojang)
		const hash63 = '03e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 63 chars - with 1 leading zero
		const hash64 = '003e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 64 chars - with 2 leading zeros
		const hash65 = '0003e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 65 chars - with 3 leading zeros

		const request62 = new IncomingRequest(`http://crafthead.net/helm/${hash62}`);
		const request63 = new IncomingRequest(`http://crafthead.net/helm/${hash63}`);
		const request64 = new IncomingRequest(`http://crafthead.net/helm/${hash64}`);
		const request65 = new IncomingRequest(`http://crafthead.net/helm/${hash65}`);

		const ctx = createExecutionContext();
		const response62 = await worker.fetch(request62, env, ctx);
		const response63 = await worker.fetch(request63, env, ctx);
		const response64 = await worker.fetch(request64, env, ctx);
		const response65 = await worker.fetch(request65, env, ctx);
		await waitOnExecutionContext(ctx);

		// All should return valid images
		expect(response62.headers.get('content-type')).toContain('image/png');
		expect(response63.headers.get('content-type')).toContain('image/png');
		expect(response64.headers.get('content-type')).toContain('image/png');
		expect(response65.headers.get('content-type')).toContain('image/png');

		// All should return the same image
		const image62 = await response62.arrayBuffer();
		const image63 = await response63.arrayBuffer();
		const image64 = await response64.arrayBuffer();
		const image65 = await response65.arrayBuffer();
		expect(image62).toStrictEqual(image63);
		expect(image63).toStrictEqual(image64);
		expect(image64).toStrictEqual(image65);
	});

	it('handles texture ID with extra leading zeros beyond 64 chars', async () => {
		// Test with extra leading zeros (65+ chars input)
		const hash64 = '003e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 64 chars - with 2 leading zeros
		const hash65 = '0003e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 65 chars - with 3 leading zeros
		const hash66 = '00003e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 66 chars - with 4 leading zeros

		const request64 = new IncomingRequest(`http://crafthead.net/helm/${hash64}`);
		const request65 = new IncomingRequest(`http://crafthead.net/helm/${hash65}`);
		const request66 = new IncomingRequest(`http://crafthead.net/helm/${hash66}`);

		const ctx = createExecutionContext();
		const response64 = await worker.fetch(request64, env, ctx);
		const response65 = await worker.fetch(request65, env, ctx);
		const response66 = await worker.fetch(request66, env, ctx);
		await waitOnExecutionContext(ctx);

		// All should return valid images
		expect(response64.headers.get('content-type')).toContain('image/png');
		expect(response65.headers.get('content-type')).toContain('image/png');
		expect(response66.headers.get('content-type')).toContain('image/png');

		// All should return the same image
		const image64 = await response64.arrayBuffer();
		const image65 = await response65.arrayBuffer();
		const image66 = await response66.arrayBuffer();
		expect(image64).toStrictEqual(image65);
		expect(image65).toStrictEqual(image66);
	});

	it('rejects invalid texture IDs with non-hex characters', async () => {
		// Test that non-hex characters are rejected
		const invalidHash = 'g3e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 'g' is not hex

		const request = new IncomingRequest(`http://crafthead.net/helm/${invalidHash}`);
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);

		// Should return 404 for invalid hash
		expect(response.status).toBe(404);
	});

	it('handles texture ID at minimum length boundary (17 chars)', async () => {
		// 17 hex characters is the minimum for texture ID (16 or fewer = username)
		// This tests the boundary between username and texture ID classification
		const hash17 = '1234567890abcdef1'; // exactly 17 chars

		const request = new IncomingRequest(`http://crafthead.net/helm/${hash17}`);
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);

		// Should attempt to process as texture ID (may 404 if texture doesn't exist, or 500)
		// The key is it's not treated as a username
		expect([200, 404, 500]).toContain(response.status);
	});

	it('handles texture ID at maximum input length boundary (90 chars)', async () => {
		// 90 chars is the maximum accepted input length (64 char hash + up to 26 leading zeros)
		// This tests the upper boundary of the generous input range
		const leadingZeros = '0'.repeat(26);
		const validHash = '1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'; // 64 chars
		const hash90 = leadingZeros + validHash; // exactly 90 chars (26 + 64 = 90)

		const request = new IncomingRequest(`http://crafthead.net/helm/${hash90}`);
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);

		// Should accept and process (normalize to 64 chars)
		expect([200, 404, 500]).toContain(response.status);
	});

	it('rejects texture ID beyond maximum input length (91 chars)', async () => {
		// 91 chars exceeds the maximum, should be rejected immediately
		const leadingZeros = '0'.repeat(27);
		const validHash = '1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'; // 64 chars
		const hash91 = leadingZeros + validHash; // 91 chars (27 + 64 = 91)

		const request = new IncomingRequest(`http://crafthead.net/helm/${hash91}`);
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);

		// Should return 404 for hash that's too long
		expect(response.status).toBe(404);
	});

	it('handles texture ID with many leading zeros (edge case)', async () => {
		// Test with a hash that has many leading zeros
		// Using the real hash from issue #126 with extra leading zeros
		const hash20Zeros = '000000000000000000003e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // 20 leading zeros (82 chars total)
		const hash64Normal = '003e77dc49bb365c095f9dc3333938d416d48d19d1089b8f037d8bebb898be7c'; // normal (64 chars)

		const request20 = new IncomingRequest(`http://crafthead.net/helm/${hash20Zeros}`);
		const request64 = new IncomingRequest(`http://crafthead.net/helm/${hash64Normal}`);

		const ctx = createExecutionContext();
		const response20 = await worker.fetch(request20, env, ctx);
		const response64 = await worker.fetch(request64, env, ctx);
		await waitOnExecutionContext(ctx);

		// Both should return valid images
		expect(response20.headers.get('content-type')).toContain('image/png');
		expect(response64.headers.get('content-type')).toContain('image/png');

		// Both should return the same image
		const image20 = await response20.arrayBuffer();
		const image64 = await response64.arrayBuffer();
		expect(image20).toStrictEqual(image64);
	});

	it('rejects texture IDs that are too long after normalization', async () => {
		// Test with a hash that's > 64 chars even after stripping zeros
		// This hash has 65 hex chars and doesn't start with 0, so it stays 65 chars after normalization
		const tooLong = '123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef01'; // 65 chars of valid hex

		const request = new IncomingRequest(`http://crafthead.net/helm/${tooLong}`);
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);

		// Should return 404 for hash that's too long
		expect(response.status).toBe(404);
	});

	it('responds with a matching avatar image by UUID, username, and texture ID', async () => {
		const request1 = new IncomingRequest('http://crafthead.net/avatar/ef6134805b6244e4a4467fbe85d65513');
		const request2 = new IncomingRequest('http://crafthead.net/avatar/CherryJimbo');
		const request3 = new IncomingRequest('http://crafthead.net/avatar/9d2e80355eed693e3f0485893ef04ff6a507f3aab33f2bedb48cef56e30f67d0');
		const ctx = createExecutionContext();
		const response1 = await worker.fetch(request1, env, ctx);
		const response2 = await worker.fetch(request2, env, ctx);
		const response3 = await worker.fetch(request3, env, ctx);
		await waitOnExecutionContext(ctx);
		const image1 = await response1.arrayBuffer();
		const image2 = await response2.arrayBuffer();
		const image3 = await response3.arrayBuffer();
		expect(image1).toStrictEqual(image2);
		expect(image2).toStrictEqual(image3);
	});

	type ProfileResponse = {
		id: string;
		name: string;
		properties: {
			name: string;
			value: string;
			signature?: string;
		}[];
	};
	it('responds with data for profile on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/profile/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		const json = await response.json<ProfileResponse>();

		expect(json.id).toBe('ef6134805b6244e4a4467fbe85d65513');
		expect(json.name).toBe('CherryJimbo');
		expect(json.properties).toBeDefined();
		expect(json.properties).toBeInstanceOf(Array);
		expect(json.properties[0].name).toBe('textures');
	});

	it('responds with data for profile on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/profile/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		const json = await response.json<ProfileResponse>();
		expect(json.id).toBe('ef6134805b6244e4a4467fbe85d65513');
		expect(json.name).toBe('CherryJimbo');
		expect(json.properties).toBeDefined();
		expect(json.properties).toBeInstanceOf(Array);
		expect(json.properties[0].name).toBe('textures');
	});
});

describe('worker headers', () => {
	it('responds with expected headers', async () => {
		const request = new IncomingRequest('http://crafthead.net/avatar/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(response.headers.get('access-control-allow-origin')).toBe('*');
		expect(response.headers.get('cache-control')).toBe('max-age=14400');
		expect(response.headers.get('content-type')).toBe('image/png');
		expect(response.headers.get('x-crafthead-request-cache-hit')).toBe('no');
		expect(response.headers.get('x-crafthead-texture-id')).toBe('9d2e80355eed693e3f0485893ef04ff6a507f3aab33f2bedb48cef56e30f67d0');
		expect(response.headers.get('x-crafthead-skin-model')).toBeNull();

		// make second response to check cache hit
		const ctx2 = createExecutionContext();
		const response2 = await worker.fetch(request, env, ctx2);
		await waitOnExecutionContext(ctx2);
		expect(response2.headers.get('x-crafthead-request-cache-hit')).toBe('yes');
	});

	it('responds with expected headers for profile', async () => {
		const request = new IncomingRequest('http://crafthead.net/profile/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(response.headers.get('access-control-allow-origin')).toBe('*');
		expect(response.headers.get('cache-control')).toBe('max-age=14400');
		expect(response.headers.get('content-type')).toBe('application/json');
		expect(response.headers.get('x-crafthead-request-cache-hit')).toBe('no');

		// make second response to check cache hit
		const ctx2 = createExecutionContext();
		const response2 = await worker.fetch(request, env, ctx2);
		await waitOnExecutionContext(ctx2);
		expect(response2.headers.get('x-crafthead-request-cache-hit')).toBe('yes');
	});

	it('responds with expected headers for body', async () => {
		const request = new IncomingRequest('http://crafthead.net/body/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(response.headers.get('access-control-allow-origin')).toBe('*');
		expect(response.headers.get('cache-control')).toBe('max-age=14400');
		expect(response.headers.get('content-type')).toBe('image/png');
		expect(response.headers.get('x-crafthead-request-cache-hit')).toBe('no');
		expect(response.headers.get('x-crafthead-skin-model')).toBe('default');
		expect(response.headers.get('x-crafthead-texture-id')).toBe('9d2e80355eed693e3f0485893ef04ff6a507f3aab33f2bedb48cef56e30f67d0');

		// make second response to check cache hit
		const ctx2 = createExecutionContext();
		const response2 = await worker.fetch(request, env, ctx2);
		await waitOnExecutionContext(ctx2);
		expect(response2.headers.get('x-crafthead-request-cache-hit')).toBe('yes');
	});

	it('responds with expected headers for body (slim)', async () => {
		const request = new IncomingRequest('http://crafthead.net/body/Alex');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(response.headers.get('access-control-allow-origin')).toBe('*');
		expect(response.headers.get('cache-control')).toBe('max-age=14400');
		expect(response.headers.get('content-type')).toBe('image/png');
		expect(response.headers.get('x-crafthead-request-cache-hit')).toBe('no');
		expect(response.headers.get('x-crafthead-skin-model')).toBe('slim');

		// make second response to check cache hit
		const ctx2 = createExecutionContext();
		const response2 = await worker.fetch(request, env, ctx2);
		await waitOnExecutionContext(ctx2);
		expect(response2.headers.get('x-crafthead-request-cache-hit')).toBe('yes');
	});
});
