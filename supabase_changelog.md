# Supabase Changelog

## [2025-03-04] Настройка RLS и отказ от ANON доступа

### Цель

Отказаться от анонимного доступа (ANON_KEY) и реализовать модель безопасности: 
любой аутентифицированный пользователь может **создавать** данные (своего профиля), 
но **изменять и удалять** может только свои данные.

```sql
-- Добавить колонку auth_user_id с внешним ключом на auth.users
ALTER TABLE public."user" 
ADD COLUMN auth_user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE;

-- Индексы для поиска
CREATE INDEX idx_user_auth_user_id ON public."user"(auth_user_id);
CREATE INDEX idx_user_email ON public."user"(email);

-- Ограничения уникальности
ALTER TABLE public."user" ADD CONSTRAINT unique_auth_user_id UNIQUE (auth_user_id);
ALTER TABLE public."user" ADD CONSTRAINT unique_email UNIQUE (email);

-- Включить Row Level Security
ALTER TABLE public."user" ENABLE ROW LEVEL SECURITY;

-- RLS политики
CREATE POLICY "Users can read own data" 
ON public."user" FOR SELECT 
USING (auth_user_id = auth.uid());

CREATE POLICY "Authenticated users can insert own profile" 
ON public."user" FOR INSERT 
WITH CHECK (auth_user_id = auth.uid());

CREATE POLICY "Users can update own data" 
ON public."user" FOR UPDATE 
USING (auth_user_id = auth.uid())
WITH CHECK (auth_user_id = auth.uid());

CREATE POLICY "Users can delete own data" 
ON public."user" FOR DELETE 
USING (auth_user_id = auth.uid());
```

### Примечания

- `auth_user_id` связывает профиль с таблицей `auth.users` (каскадное удаление)
- `email` должен быть уникальным для предотвращения дубликатов
- Все операции ограничены владельцем данных через RLS
