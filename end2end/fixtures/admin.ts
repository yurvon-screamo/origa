import { trailBaseUrl } from "../config";
import { fetchWithTimeout } from "../helpers/http";

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

interface ListUsersResponse {
	total_row_count: number;
	users: ListedUser[];
}

interface ListedUser {
	id: string;
	email: string;
	verified: boolean;
	admin: boolean;
}

export interface AdminTokens {
	token: string;
	csrfToken: string;
}

export interface LoginResponse {
	token: string;
	csrfToken?: string;
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

export async function deleteTestUserWithRetry(
	token: string,
	csrfToken: string,
	userId: string,
	userEmail: string,
	retries: number = 1,
): Promise<void> {
	for (let attempt = 0; attempt <= retries; attempt++) {
		try {
			await deleteTestUser(token, csrfToken, userId);
			return;
		} catch (error) {
			const isRetryable = error instanceof Error && isRetryableError(error);
			if (attempt < retries && isRetryable) {
				console.error(`[admin] Retrying delete user ${userEmail} (${userId}), attempt ${attempt + 1}`);
				await new Promise((resolve) => setTimeout(resolve, 500));
				continue;
			}
			throw error;
		}
	}
}

export async function listAllTestUsers(
	token: string,
	csrfToken: string,
): Promise<{ id: string; email: string }[]> {
	const PAGE_SIZE = 500;
	const allE2eUsers: { id: string; email: string }[] = [];
	let offset = 0;
	let totalRowcount = Infinity;

	while (offset < totalRowcount) {
		const response = await fetchWithTimeout(
			`${trailBaseUrl}/api/_admin/user?limit=${PAGE_SIZE}&offset=${offset}`,
			{
				method: "GET",
				headers: {
					Authorization: `Bearer ${token}`,
					"csrf-token": csrfToken,
				},
			},
		);

		if (!response.ok) {
			const text = await response.text();
			throw new Error(`Failed to list users: ${response.status} ${text}`);
		}

		const data = (await response.json()) as ListUsersResponse;
		totalRowcount = data.total_row_count;

		const e2eUsers = data.users.filter(
			(u) => u.email.startsWith("e2e-") && u.email.endsWith("@origa.local"),
		);
		allE2eUsers.push(...e2eUsers.map((u) => ({ id: u.id, email: u.email })));

		offset += data.users.length;

		if (data.users.length === 0) break;
	}

	return allE2eUsers;
}

function isRetryableError(error: Error): boolean {
	const message = error.message.toLowerCase();
	return (
		message.includes("timeout") ||
		message.includes("abort") ||
		message.includes("network") ||
		message.includes("500") ||
		message.includes("502") ||
		message.includes("503") ||
		message.includes("504")
	);
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