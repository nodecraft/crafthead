import { identityTypeEnumToString } from '../request';

import type { IdentityKind } from '../request';

type AnalyticsData = {
	startTime?: Date;
	kind?: string;
	identityType?: IdentityKind;
	cached?: boolean;
	responseCode?: number;
	game?: string;
};

export function writeDataPoint(
	analytics: AnalyticsEngineDataset | undefined,
	request: Request,
	data?: AnalyticsData,
) {
	if (!analytics) { return; }

	const endTime = new Date();
	const totalTime = data?.startTime ? endTime.getTime() - data?.startTime?.getTime() : 0;

	let identityKindString = 'unknown';
	if (data?.identityType !== undefined) {
		identityKindString = identityTypeEnumToString(data.identityType);
	}
	/* ORDER HERE IS VERY IMPORTANT. IF ANYTHING CHANGES, MUST BE APPENDED. MAX 20 */
	const reportData: AnalyticsEngineDataPoint = {
		blobs: [
			// raw req data
			request.url,
			request.headers.get('user-agent'),
			request.headers.get('referer'),
			(request.cf?.httpProtocol as string) || 'unknown',
			(request.cf?.city as string) || 'unknown',
			(request.cf?.colo as string) || 'unknown',
			(request.cf?.country as string) || 'unknown',
			(request.cf?.tlsVersion as string) || 'unknown',

			// crafthead data
			data?.kind || 'unknown',
			identityKindString,
			data?.game || 'unknown',
		],
		doubles: [
			(request.cf?.asn as number) || 0,
			data?.responseCode ?? 0,
			data?.cached ? 1 : 0,
			totalTime,
		],
	};
	try {
		analytics?.writeDataPoint?.(reportData);
	} catch (err) {
		console.error(err);
	}
}
