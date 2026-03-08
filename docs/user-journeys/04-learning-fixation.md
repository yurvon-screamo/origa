# User Journey: Learning - Fixation

## Цель

Пользователь закрепляет пройденный материал: система отбирает карточки с high difficulty для повторения.

---

## Сценарий

### Step 1: Выбор карточек для закрепления

**Given:** У пользователя есть карточки с high difficulty

**When:** Запрашиваются карточки для закрепления

**Then:**
- Возвращается map карточек с is_high_difficulty = true
- Лимит: 15 карточек
- Отсортированы по next_review_date (давние первыми)

---

### Step 2: Оценка карточки (Rating::Again)

**Given:** Карточка показана пользователю

**When:** Пользователь забыл (Again)

**Then:**
- Memory state обновлён: stability сброшена/уменьшена
- Difficulty увеличена
- Карточка останется в high difficulty

---

### Step 3: Оценка карточки (Rating::Good)

**Given:** Карточка показана пользователю

**When:** Пользователь вспомнил (Good)

**Then:**
- Memory state обновлён: stability увеличена
- Difficulty уменьшена
- Карточка может выйти из high difficulty (если difficulty < порог)

---

### Step 4: Полный цикл закрепления

**Given:** Пользователь начинает закрепление

**When:** Проходит через все отобранные карточки

**Then:**
- Все карточки оценены
- Memory states обновлены
- Lesson history обновлён

---

## Различия Lesson vs Fixation

| Aspect | Lesson | Fixation |
|--------|--------|----------|
| **Цель** | Изучение новых + повторение due | Повторение сложных карточек |
| **Source** | Все карточки по приоритету | Только high_difficulty |
| **Лимит** | 7 новых + due | 15 карточек |
| **RateMode** | StandardLesson | Fixation |
| **FSRS поведение** | Стандартный алгоритм | Акцент на сложных карточках |

---

## High Difficulty Logic

Карточка считается high_difficulty когда:
- Difficulty > пороговое значение
- ИЛИ была оценена как Again/Hard недавно

---

## Edge Cases

### EmptyFixation - нет сложных карточек
- **Given:** Нет карточек с high_difficulty
- **When:** Запрашиваются карточки для закрепления
- **Then:** Возвращается пустой map

### EmptyFixation - карточки не due
- **Given:** Все карточки имеют low difficulty
- **When:** Запрашиваются карточки для закрепления
- **Then:** Возвращается пустой map

---

## Тестовые данные

| Параметр | Значения |
|----------|----------|
| Количество карточек | 1, 5, 10 |
| Ratings | Again, Good |
| RateMode | Fixation |
| Difficulty states | high, low |
