// PromiseGatherer is intended to collect promises we don't need to await on right away,
// and defer them to be run after the request ends.
export default class PromiseGatherer {
	promises: Promise<unknown>[];

	constructor() {
		this.promises = [];
	}

	push<T>(promise: Promise<T>) {
		this.promises.push(promise);
	}

	all(): Promise<unknown> {
		return Promise.all(this.promises);
	}
}
