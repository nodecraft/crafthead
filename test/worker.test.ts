import { createExecutionContext, env, waitOnExecutionContext } from 'cloudflare:test';
import { describe, expect, it } from 'vitest';

import worker from '../src/worker/index';

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

describe('worker', () => {
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

	it('responds with data for profile', async () => {
		const request = new IncomingRequest('http://crafthead.net/profile/ef6134805b6244e4a4467fbe85d65513');
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		await waitOnExecutionContext(ctx);
		const json = await response.json<any>();
		expect(json.id).toBe('ef6134805b6244e4a4467fbe85d65513');
		expect(json.name).toBe('CherryJimbo');
		expect(json.properties).toBeDefined();
		expect(json.properties).toBeInstanceOf(Array);
		expect(json.properties[0].name).toBe('textures');
	});
});
