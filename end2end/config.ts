import { config } from "dotenv";
import { resolve } from "path";

// Override system env vars with local .env values
const { parsed } = config({ path: resolve(__dirname, ".env"), override: true });

export function getTrailBaseUrl(): string {
    const url = process.env.TRAILBASE_URL;
    if (!url) {
        throw new Error("TRAILBASE_URL is not set. Configure it in .env file.");
    }
    return url;
}
