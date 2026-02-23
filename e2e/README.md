# E2E Test Configuration

## Required Environment Variables

For tests that require a confirmed user (profile, grammar, kanji, words), set:

```bash
export E2E_TEST_EMAIL="your-confirmed-test@email.com"
export E2E_TEST_PASSWORD="YourTestPassword123!"
```

## Creating a Confirmed Test User

1. Go to Supabase Dashboard: <https://supabase.com/dashboard>
2. Navigate to your project → Authentication → Users
3. Click "Add user" → "Create new user"
4. Enter email and password (e.g., `e2e-test@origa.app` / `TestPass123!`)
5. **Important:** Check "Auto Confirm User" to skip email verification
6. Click "Create user"

## Running Tests

```bash
npx playwright test
```

## Test Accounts

### For Development/CI

- Email: `e2e-test@origa.app`
- Password: `TestPass123!`

This account should be created with "Auto Confirm User" enabled in Supabase.
