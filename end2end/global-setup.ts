import type { FullConfig } from '@playwright/test';
import { testUser } from './config';

interface GlobalSetupConfig {
  trailBaseUrl: string;
  adminEmail: string;
  adminPassword: string | undefined;
}

function validateEnvVars(): void {
  const adminPassword = process.env.ORIGA_ADMIN_PASSWORD;

  if (!adminPassword) {
    console.warn(
      '[global-setup] WARNING: ORIGA_ADMIN_PASSWORD not set. ' +
        'User management operations will fail.'
    );
  }
}

function getConfig(): GlobalSetupConfig {
  return {
    trailBaseUrl: process.env.TRAILBASE_URL || 'https://origa.uwuwu.net',
    adminEmail: process.env.ORIGA_ADMIN_EMAIL || 'admin@localhost',
    adminPassword: process.env.ORIGA_ADMIN_PASSWORD,
  };
}

export default async function globalSetup(config: FullConfig): Promise<void> {
  console.log('[global-setup] Starting E2E test setup...');

  validateEnvVars();

  const configData = getConfig();

  console.log('[global-setup] Configuration:');
  console.log(`  - TRAILBASE_URL: ${configData.trailBaseUrl}`);
  console.log(`  - ORIGA_ADMIN_EMAIL: ${configData.adminEmail}`);
  console.log(`  - ORIGA_ADMIN_PASSWORD: ${configData.adminPassword ? '*****' : '(not set)'}`);
  console.log(`  - TEST_USER_EMAIL: ${testUser.email}`);

  process.env.TRAILBASE_URL = configData.trailBaseUrl;
  process.env.ORIGA_ADMIN_EMAIL = configData.adminEmail;
  if (configData.adminPassword) {
    process.env.ORIGA_ADMIN_PASSWORD = configData.adminPassword;
  }

  console.log('[global-setup] Setup complete.');
}