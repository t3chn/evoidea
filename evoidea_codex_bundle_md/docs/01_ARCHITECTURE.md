# 01_ARCHITECTURE — modules and extensibility

## 1. Diagram
[CLI] -> [Orchestrator] -> [Pipeline]
                      -> [Storage]
                      -> [LlmProvider]
                      -> [Scoring/Selection]

## 2. Key abstractions

### Phase (plugin)
Each phase implements:
- `name() -> &str`
- `run(state, ctx) -> state`

Pipeline is a list of phases from config.

### LlmProvider
Trait:
- `generate_json(task: LlmTask, schema_path: &Path) -> serde_json::Value`

Implementations:
- `MockLlmProvider` (fixtures)
- `CodexExecProvider` (spawn `codex exec ... --output-schema <schema.json>`)
- `CommandProvider` (generic external CLI, if needed)

### Storage
Trait:
- `init_run(config) -> run_id`
- `load_state / save_state`
- `append_event`
- `save_final`

Implementations:
- `FileStorage` (default)
- `SqliteStorage` (feature flag)

## 3. Extending \"without rewrites\"
Add a phase = new module + register in the pipeline config.
Add a provider = implement LlmProvider.
Add storage = implement Storage.

## 4. Context and compression
Send a compact context to the LLM:
- only top-K ideas + new candidates
- only necessary fields
- (optional) TOON for uniform arrays

Canonical data is stored in JSON.

## 5. Logs and history
- `history.ndjson` as an append-only per-iteration event log
- `tracing` spans keyed by run_id and iteration
