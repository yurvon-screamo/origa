import { testUser, trailBaseUrl } from '../config';

const FETCH_TIMEOUT_MS = 30000;

interface AuthResponse {
  auth_token: string;
  refresh_token?: string;
}

interface UserCreateRequest {
  email: string;
  password: string;
  verified?: boolean;
  admin?: boolean;
}

interface UserDeleteRequest {
  id: string;
}

async function fetchWithTimeout(
  url: string,
  options: RequestInit,
  timeout: number = FETCH_TIMEOUT_MS
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

export async function getAdminToken(): Promise<string> {
  const adminEmail = process.env.ORIGA_ADMIN_EMAIL || 'admin@localhost';
  const adminPassword = process.env.ORIGA_ADMIN_PASSWORD;

  if (!adminPassword) {
    throw new Error('ORIGA_ADMIN_PASSWORD is required for user management. Set it in .env file.');
  }

  const response = await fetchWithTimeout(
    `${trailBaseUrl}/api/auth/v1/login`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        email: adminEmail,
        password: adminPassword,
      }),
    }
  );

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Admin login failed: ${response.status} ${text}`);
  }

  const data = (await response.json()) as AuthResponse;
  return data.auth_token;
}

export async function createTestUser(
  token: string,
  email: string = testUser.email,
  password: string = testUser.password
): Promise<void> {
  const response = await fetchWithTimeout(
    `${trailBaseUrl}/api/_/user/create`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({
        email,
        password,
        verified: true,
        admin: false,
      } as UserCreateRequest),
    }
  );

  if (!response.ok) {
    const text = await response.text();
    if (response.status === 409 || text.includes('already exists')) {
      // User already exists, which is fine
      return;
    }
    throw new Error(`Failed to create user: ${response.status} ${text}`);
  }

  console.log('[admin] Created test user');
}

export async function deleteTestUser(
  token: string,
  emailOrId: string = testUser.email
): Promise<void> {
  const response = await fetchWithTimeout(
    `${trailBaseUrl}/api/_/user`,
    {
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ id: emailOrId } as UserDeleteRequest),
    }
  );

  if (!response.ok) {
    if (response.status === 404) {
      // User not found, which is fine
      return;
    }
    const text = await response.text();
    throw new Error(`Failed to delete user: ${response.status} ${text}`);
  }

  console.log('[admin] Deleted test user');
}

export async function loginTestUser(
  email: string = testUser.email,
  password: string = testUser.password
): Promise<string> {
  const response = await fetchWithTimeout(
    `${trailBaseUrl}/api/auth/v1/login`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email, password }),
    }
  );

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Test user login failed: ${response.status} ${text}`);
  }

  const data = (await response.json()) as AuthResponse;
  return data.auth_token;
}