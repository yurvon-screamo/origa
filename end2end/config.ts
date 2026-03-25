// Test users are created dynamically via fixtures (see auth.fixture.ts)
// Each test gets a unique user that is automatically cleaned up
export const trailBaseUrl =
	process.env.TRAILBASE_URL || "https://origa.uwuwu.net";
