// PromiseGatherer is intended to collect promises we don't need to await on right away,
// and defer them to be run after the request ends.
export default class PromiseGatherer {
    promises: Promise<any>[];

    constructor() {
        this.promises = [];
    }

    push(p: Promise<any>) {
        this.promises.push(p);
    }

    all(): Promise<any> {
        return Promise.all(this.promises);
    }
}