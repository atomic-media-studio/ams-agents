This folder stores local metrics/tracing artifacts for offline analysis.

Current tracing output:

- `timings.jsonl` (default)

Each line is one JSON object (`JSONL`) and may include:

- `inference_timing`: per Ollama call timings (`t_start`, `t_first_token`, `t_end`, `duration_ms`, `ttft_ms`)
- `turn_timing`: per dialogue turn pacing (`gap_ms` between turns)

Notes:

- This folder is intentionally ignored by git.
- Data is intended for research workflows and post-run analysis.