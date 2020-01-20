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