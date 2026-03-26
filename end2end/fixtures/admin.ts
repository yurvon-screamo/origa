import { trailBaseUrl } from "../config";

const FETCH_TIMEOUT_MS = 30000;

interface AuthResponse {
	auth_token: string;
	refresh_token?: string;
	csrf_token?: string;
}

interface UserCreateRequest {
	email: string;
	password: string;
	verified?: boolean;
	admin?: boolean;
}

interface CreateUserResponse {
	id: string;
	email: string;
}

interface UserDeleteRequest {
	id: string;
}

export interface AdminTokens {
	token: string;
	csrfToken: string;
}

export interface LoginResponse {
	token: string;
	csrfToken?: string;
}

async function fetchWithTimeout(
	url: string,
	options: RequestInit,
	timeout: number = FETCH_TIMEOUT_MS,
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

export async function getAdminToken(): Promise<AdminTokens> {
	const adminEmail = process.env.ORIGA_ADMIN_EMAIL || "admin@localhost";
	const adminPassword = process.env.ORIGA_ADMIN_PASSWORD;

	if (!adminPassword) {
		throw new Error(
			"ORIGA_ADMIN_PASSWORD is required for user management. Set it in .env file.",
		);
	}

	const response = await fetchWithTimeout(`${trailBaseUrl}/api/auth/v1/login`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			email: adminEmail,
			password: adminPassword,
		}),
	});

	if (!response.ok) {
		const text = await response.text();
		throw new Error(`Admin login failed: ${response.status} ${text}`);
	}

	const data = (await response.json()) as AuthResponse;
	const csrfToken = data.csrf_token;

	if (!csrfToken) {
		throw new Error("Admin login response missing csrf_token");
	}

	return {
		token: data.auth_token,
		csrfToken,
	};
}

export async function createTestUser(
	token: string,
	csrfToken: string,
	email: string,
	password: string,
): Promise<string> {
	const response = await fetchWithTimeout(`${trailBaseUrl}/api/_admin/user`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
			Authorization: `Bearer ${token}`,
			"csrf-token": csrfToken,
		},
		body: JSON.stringify({
			email,
			password,
			verified: true,
			admin: false,
		} as UserCreateRequest),
	});

	if (!response.ok) {
		const text = await response.text();
		if (response.status === 409 || text.includes("already exists")) {
			throw new Error(`User already exists: ${email}. Cannot retrieve UUID without creation.`);
		}
		throw new Error(`Failed to create user: ${response.status} ${text}`);
	}

	const data = (await response.json()) as CreateUserResponse;
	console.log(`[admin] Created test user: ${email} (id: ${data.id})`);
	return data.id;
}

export async function deleteTestUser(
	token: string,
	csrfToken: string,
	userId: string,
): Promise<void> {
	const response = await fetchWithTimeout(`${trailBaseUrl}/api/_admin/user`, {
		method: "DELETE",
		headers: {
			"Content-Type": "application/json",
			Authorization: `Bearer ${token}`,
			"csrf-token": csrfToken,
		},
		body: JSON.stringify({ id: userId } as UserDeleteRequest),
	});

	if (!response.ok) {
		if (response.status === 404) {
			return;
		}
		const text = await response.text();
		throw new Error(`Failed to delete user: ${response.status} ${text}`);
	}

	console.log(`[admin] Deleted test user: ${userId}`);
}

export async function loginTestUser(
	email: string,
	password: string,
): Promise<LoginResponse> {
	const response = await fetchWithTimeout(`${trailBaseUrl}/api/auth/v1/login`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({ email, password }),
	});

	if (!response.ok) {
		const text = await response.text();
		throw new Error(`Test user login failed: ${response.status} ${text}`);
	}

	const data = (await response.json()) as AuthResponse;
	return {
		token: data.auth_token,
		csrfToken: data.csrf_token,
	};
}