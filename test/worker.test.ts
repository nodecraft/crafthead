import { createExecutionContext, env, waitOnExecutionContext } from 'cloudflare:test';
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
	});

	it('responds with image for avatar on ID with dashes', async () => {
		const request = new IncomingRequest('http://crafthead.net/avatar/ef613480-5b62-44e4-a446-7fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for avatar on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/avatar/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for cape on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/cape/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for cape on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/cape/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for helm on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/helm/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for helm on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/helm/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for cube on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/cube/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for cube on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/cube/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for body on Id', async () => {
		const request = new IncomingRequest('http://crafthead.net/body/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for body on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/body/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for bust on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/bust/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for bust on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/bust/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for skin on ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/skin/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for skin on username', async () => {
		const request = new IncomingRequest('http://crafthead.net/skin/CherryJimbo');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
	});

	it('responds with image for avatar on texture ID', async () => {
		const request = new IncomingRequest('http://crafthead.net/avatar/9d2e80355eed693e3f0485893ef04ff6a507f3aab33f2bedb48cef56e30f67d0');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		expect(await response.headers.get('content-type')).toContain('image/png');
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
