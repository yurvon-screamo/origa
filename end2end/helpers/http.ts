const DEFAULT_FETCH_TIMEOUT_MS = 30000;

export async function fetchWithTimeout(
	url: string,
	options: RequestInit,
	timeout: number = DEFAULT_FETCH_TIMEOUT_MS,
): Promise<Response> {
	const controller = new AbortController();
	const timeoutId = setTimeout(() => controller.abort(), timeout);

	try {
		const response = await fetch(url, {
			...options,
			signal: controller.signal,
		});
		return response;
	} finally {
		clearTimeout(timeoutId);
	}
}
