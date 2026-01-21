declare namespace Cloudflare {
	interface Env {
		ASSETS: Fetcher;
		CRAFTHEAD_ANALYTICS?: AnalyticsEngineDataset;
		PLAYERDB?: Fetcher;
		HYTALE_ASSETS?: R2Bucket;
		HYTALE_RENDERS_CACHE?: R2Bucket;
		WORKER_ENV: string;
	}
}
