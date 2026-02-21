# Tasks

## Phase 1: Базовые формы глаголов (N5)
- [x] Task 1.1: Реализовать функции трансформации глаголов в `forms_verb.rs`
  - [x] `to_masu_form` - ます-форма
  - [x] `to_masen_form` - ません-форма
  - [x] `to_mashita_form` - ました-форма
  - [x] `to_masen_deshita_form` - ませんでした-форма
  - [x] `to_nai_form` - ない-форма
  - [x] `to_ta_form` - た-форма
  - [x] `to_dictionary_form` - словарная форма
  - [x] `to_tara_form` - たら-форма
- [x] Task 1.2: Добавить правила в `grammar.json` для базовых глагольных форм N5

## Phase 2: Базовые формы прилагательных (N5)
- [x] Task 2.1: Реализовать функции трансформации i-прилагательных в `forms_adjective.rs`
  - [x] `to_kunai_form` - くない (отрицание)
  - [x] `to_katta_form` - かった (прошедшее)
  - [x] `to_kunakatta_form` - くなかった (отрицание прошедшего)
  - [x] `to_kute_form` - くて (соединительная)
  - [x] `to_ku_form` - く (наречие)
- [x] Task 2.2: Реализовать функции трансформации na-прилагательных в `forms_adjective.rs`
  - [x] `to_na_form` - な (определительная)
  - [x] `to_de_form` - で (соединительная)
- [x] Task 2.3: Добавить правила в `grammar.json` для форм прилагательных N5

## Phase 3: Расширенные формы глаголов (N4)
- [x] Task 3.1: Реализовать условные формы
  - [x] `to_ba_form` - ば-форма
- [x] Task 3.2: Реализовать залоговые формы
  - [x] `to_potential_form` - 可能形
  - [x] `to_passive_form` - 受身
  - [x] `to_causative_form` - 使役
  - [x] `to_imperative_form` - 命令形
  - [x] `to_volitional_form` - 意向形
- [x] Task 3.3: Реализовать модальные формы
  - [x] `to_stem_form` - основа (для компаундов)
- [x] Task 3.4: Добавить правила в `grammar.json` для расширенных форм N4

## Phase 4: N3 формы
- [x] Task 4.1: Реализовать компаунды (через VerbToStem + AddPostfix)
  - [x] 〜だす, 〜はじめる, 〜おわる, 〜つづける, 〜わすれる, 〜あう, 〜かえる
- [x] Task 4.2: Реализовать сокращения
  - [x] 〜ちゃう, 〜とく, 〜てる
- [x] Task 4.3: Реализовать дополнительные формы
  - [x] `to_causative_passive_form` - 使役受身
  - [x] `to_sou_form` - そう-форма (вид)
  - [x] `to_zu_form` - ず-форма
  - [x] `to_garu_form` - がる-форма
  - [x] `to_kereba_form` - ければ (i-adj условная)
  - [x] `to_nara_form` - なら (na-adj условная)
  - [x] `to_nasasou_form` - なさそう (отрицание そう)
  - [x] `to_sugiru_form` - すぎる (слишком)
- [x] Task 4.4: Добавить правила в `grammar.json` для форм N3

## Phase 5: Вежливые формы (N4-N3)
- [x] Task 5.1: Реализовать вежливые формы
  - [x] `to_o_ni_narimasu_form` - お〜になります
  - [x] `to_o_kudasai_form` - お/ご〜ください
  - [x] `to_o_shimasu_form` - お/ご〜します
  - [ ] `to_o_masu_form` - お〜ますです (опционально)

## Phase 6: Правила без format_map
- [x] Task 6.1: Добавить все правила N5 с format_map: null в `grammar.json`
- [x] Task 6.2: Добавить все правила N4 с format_map: null в `grammar.json`
- [x] Task 6.3: Добавить все правила N3 с format_map: null в `grammar.json`

---

# Итоги

**Реализовано:**
- `forms_verb.rs`: 31 функция трансформации
- `forms_adjective.rs`: 14 функций трансформации
- `grammar.json`: 147 правил (с format_map и без)

**Компиляция:** ✅ Успешно
