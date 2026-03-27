# Origa E2E Tests

Playwright end-to-end тесты для приложения Origa.

## Установка

```bash
cd e2e
npm install
npx playwright install
```

## Запуск тестов

```bash
# Все тесты
npm test

# С UI
npm run test:ui

# В headed режиме
npm run test:headed

# Debug режим
npm run test:debug

# Только smoke тесты
npm run test:smoke

# Только critical тесты
npm run test:critical

# Просмотр отчёта
npm run report
```

## Структура

```
e2e/
├── fixtures/          # Custom fixtures
├── pages/             # Page Objects
├── tests/             # Тесты
│   ├── auth/          # Авторизация
│   ├── home.spec.ts   # Главная страница
│   ├── lesson.spec.ts # Уроки
│   ├── kanji.spec.ts  # Кандзи
│   ├── words.spec.ts  # Словарь
│   ├── grammar.spec.ts# Грамматика
│   └── sets.spec.ts   # Наборы
└── playwright/
    └── .auth/         # Auth state (gitignored)
```

## Тегирование тестов

- `@smoke` - базовые smoke тесты
- `@critical` - критические сценарии
- `@slow` - медленные тесты

## Требования

- Запущенный trunk serve на http://localhost:8080
- Тестовый пользователь: e2e@sample.com / 12345678
