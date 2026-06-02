import "dotenv/config";

export function getTrailBaseUrl(): string {
    const url = process.env.TRAILBASE_URL;
    if (!url) {
        throw new Error("TRAILBASE_URL is not set. Configure it in .env file.");
    }
    return url;
}
