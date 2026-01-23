# non_md — файлы, которые нужно материализовать

Этот пакет по запросу содержит только `.md` файлы.  
Но проекту понадобятся настоящие `.json/.sh/.csv/.mjs`.

## Что сделать
1) Создай в целевом репозитории директории:
- `schemas/`
- `.codex/skills/evo-ideator/scripts/`
- `evals/`

2) Материализуй файлы:

### Schemas
См. `non_md/schemas/SCHEMAS.json.md` — там несколько блоков с `Target path:`.

### Скрипт skill
Создай файл:
- `.codex/skills/evo-ideator/scripts/run_evoidea.sh`
с содержимым из `non_md/codex_skill_scripts/run_evoidea.sh.md`

Не забудь:
- `chmod +x .codex/skills/evo-ideator/scripts/run_evoidea.sh`

### Evals (опционально)
Создай:
- `evals/evo-ideator.prompts.csv` из `non_md/evals/evo-ideator.prompts.csv.md`
- `evals/run-evals.mjs` из `non_md/evals/run-evals.mjs.md`
