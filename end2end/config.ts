// Этот пользователь создается автоматически на время запуска теста, после автоматически удаляется.
export const testUser = {
    email: "e2e-runner@origa.local",
    password: "e2e-test-password-123",
};

export const trailBaseUrl =
    process.env.TRAILBASE_URL || "https://origa.uwuwu.net";
