# E2E Tests

## Test Structure

```
e2e/
├── fixtures/
│   └── test-helpers.ts      # Shared utilities for tests
├── journeys/                # User Journey tests
│   ├── full-learning-cycle.spec.ts
│   ├── content-creation.spec.ts
│   ├── lesson-ratings.spec.ts
│   ├── fixation-mode.spec.ts
│   ├── profile-management.spec.ts
│   ├── auth-validation.spec.ts
│   └── search-filters.spec.ts
├── pages/                   # Page Objects
│   ├── LoginPage.ts
│   ├── HomePage.ts
│   ├── KanjiPage.ts
│   ├── WordsPage.ts
│   ├── GrammarPage.ts
│   ├── LessonPage.ts
│   └── ProfilePage.ts
└── TEST_MODEL.md            # Test documentation
```

## Required Environment Variables

Set these in `e2e/.env`:

```
E2E_TEST_EMAIL=your-confirmed-test@email.com
E2E_TEST_PASSWORD=YourTestPassword123!
```

## Creating a Confirmed Test User

1. Go to Supabase Dashboard
2. Navigate to Authentication → Users
3. Click "Add user" → "Create new user"
4. Enter email and password
5. Check "Auto Confirm User"
6. Click "Create user"

## Running Tests

```bash
npx playwright test
```

Run specific test file:

```bash
npx playwright test journeys/full-learning-cycle.spec.ts
```

Run with UI:

```bash
npx playwright test --ui
```
