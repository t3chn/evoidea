# Target path: `evals/run-evals.mjs`

```js
import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync, readFileSync, existsSync } from "node:fs";
import path from "node:path";

function runCodex(prompt, outJsonlPath) {
  const res = spawnSync(
    "codex",
    ["exec", "--json", "--full-auto", prompt],
    { encoding: "utf-8" }
  );
  writeFileSync(outJsonlPath, res.stdout ?? "", "utf-8");
  return { code: res.status ?? 1, stderr: res.stderr ?? "" };
}

function main() {
  const csvPath = path.join("evals", "evo-ideator.prompts.csv");
  const lines = readFileSync(csvPath, "utf-8").trim().split("\n");
  const rows = lines.slice(1).map(l => {
    const [id, should_trigger, ...rest] = l.split(",");
    const prompt = rest.join(",").replace(/^"|"$/g, "");
    return { id, should_trigger: should_trigger === "true", prompt };
  });

  mkdirSync(path.join("evals", "runs"), { recursive: true });

  let pass = 0;
  for (const r of rows) {
    const out = path.join("evals", "runs", `${r.id}.jsonl`);
    const { code, stderr } = runCodex(r.prompt, out);

    const jsonl = existsSync(out) ? readFileSync(out, "utf-8") : "";
    const ok = r.should_trigger ? (code === 0 && jsonl.length > 0) : (code === 0);

    console.log(`${r.id}: ${ok ? "PASS" : "FAIL"}`);
    if (!ok) console.log(stderr);
    if (ok) pass++;
  }

  console.log(`\n${pass}/${rows.length} passed`);
  process.exit(pass === rows.length ? 0 : 1);
}

main();

```
