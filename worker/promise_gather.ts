// PromiseGatherer is intended to collect promises we don't need to await on right away,
// and defer them to be run after the request ends.
export default class PromiseGatherer {
	promises: Promise<any>[];

	constructor() {
		this.promises = [];
	}

	push(promise: Promise<any>) {
		this.promises.push(promise);
	}

	all(): Promise<any> {
		return Promise.all(this.promises);
	}
}
