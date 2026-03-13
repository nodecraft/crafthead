import { createExecutionContext, waitOnExecutionContext } from 'cloudflare:test';
import { env } from 'cloudflare:workers';
import { describe, expect, it } from 'vitest';

import worker from '../src/worker/index';

const IncomingRequest = Request<unknown, IncomingRequestCfProperties>;

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
