# User Journey: Onboarding

## Цель

Новый пользователь регистрируется и настраивает профиль для начала обучения.

---

## Сценарий

### Step 1: Регистрация нового пользователя

**Given:** Пользователь впервые открывает приложение

**When:** Создаётся новый пользователь с email и native_language

**Then:**
- Пользователь создан с уникальным ID
- Email сохранён
- Username = часть email до @
- Native language = переданное значение
- JLPT level = N5 (по умолчанию)
- Knowledge set пуст
- Reminders включены

---

### Step 2: Получение профиля

**Given:** Пользователь существует

**When:** Запрашивается информация о профиле

**Then:**
- Возвращается профиль с ID, username, native_language, current_level, jlpt_progress, lesson_history

---

### Step 3: Обновление профиля

**Given:** Пользователь хочет изменить настройки

**When:** Обновляются native_language, telegram_user_id, reminders_enabled

**Then:**
- Все поля обновлены
- Изменения сохранены

---

## Edge Cases

### UserNotFound при GetUserInfo
- **When:** Запрашивается профиль несуществующего пользователя
- **Then:** Ошибка UserNotFound

### UserNotFound при UpdateUserProfile
- **When:** Обновляется профиль несуществующего пользователя
- **Then:** Ошибка UserNotFound

---

## Тестовые данные

| Параметр | Значения |
|----------|----------|
| Email | `test@example.com` |
| NativeLanguage | Russian, English |
| Telegram ID | None, 123456789 |
