name: Задача фабрике (work)
description: Разработка через origa-factory — пайп с ревью, ПР и скринами
labels: ["work"]
body:
  - type: textarea
    id: context
    attributes:
      label: Контекст
      description: Что делаем и зачем. Ссылки на связанные места в репо.
    validations:
      required: true
  - type: textarea
    id: acceptance
    attributes:
      label: Критерии приёмки
      description: Список проверяемых условий «сделано, если…». По ним агент и ревьюер проверяют результат.
      placeholder: |
        - ...
        - ...
    validations:
      required: true
  - type: checkboxes
    id: ui
    attributes:
      label: Затрагивает UI/верстку
      description: Если да — к ПР будут приложены скрины desktop (1280×800) и mobile (390×844)
    options:
      - label: Да, затрагивает UI
        required: false
  - type: textarea
    id: notes
    attributes:
      label: Заметки
      description: Ограничения, ссылки на референсы, что НЕ трогать.
    validations:
      required: false
