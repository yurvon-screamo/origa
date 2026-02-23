# Supabase Changelog

## Настройка RLS и отказ от ANON доступа

### Цель

Отказаться от анонимного доступа (ANON_KEY) и реализовать модель безопасности: любой аутентифицированный пользователь может **создавать** данные (своего профиля), но **изменять и удалять** может только свои данные.

### Добавить колонку `auth_user_id` в таблицу `user`

```sql
ALTER TABLE public."user" ADD COLUMN auth_user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE;
CREATE INDEX idx_user_auth_user_id ON public."user"(auth_user_id);
ALTER TABLE public."user" ADD CONSTRAINT unique_auth_user_id UNIQUE (auth_user_id);
```

#### 2. Включить Row Level Security (RLS)

```sql
ALTER TABLE public."user" ENABLE ROW LEVEL SECURITY;
```

### Создать RLS политики

```sql
-- Политика: Пользователь может читать только свои данные
CREATE POLICY "Users can read own data" 
ON public."user" 
FOR SELECT 
USING (auth_user_id = auth.uid());

-- Политика: Любой аутентифицированный пользователь может создать свой профиль
CREATE POLICY "Authenticated users can insert own profile" 
ON public."user" 
FOR INSERT 
WITH CHECK (auth_user_id = auth.uid());

-- Политика: Пользователь может обновлять только свои данные
CREATE POLICY "Users can update own data" 
ON public."user" 
FOR UPDATE 
USING (auth_user_id = auth.uid())
WITH CHECK (auth_user_id = auth.uid());

-- Политика: Пользователь может удалять только свои данные
CREATE POLICY "Users can delete own data" 
ON public."user" 
FOR DELETE 
USING (auth_user_id = auth.uid());
```
