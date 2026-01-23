# 08_EVALS — e2e проверки через `codex exec --json` (опционально)

## Идея
Проверяем не “красоту идеи”, а:
- что skill триггерится
- что создаются артефакты
- что соблюдается структура output
- что цикл останавливается по правилам

## Датасет
`evals/evo-ideator.prompts.csv` (см. `non_md/evals/*.md`)

## Раннер
`evals/run-evals.mjs` (см. `non_md/evals/*.md`)

## Замечание
`codex exec --json` печатает newline-delimited JSON events.
