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
