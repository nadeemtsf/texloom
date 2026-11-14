# Prototype source-contract oracle

`tools/source_contract.py` checks an independently predeclared, sources-only snapshot contract. It imports no app planner and executes no LaTeX. Its output is `matches_contract`, `differs` or `unresolved`; none authorizes an app commit.

Schema 1 has a `files` mapping keyed by normalized relative paths. Every entry supplies `before_sha256` (null only for a declared new file) and exactly one of `after_sha256` or `after_utf8_template`. The baseline file set and hashes must match before candidate evaluation. The candidate must contain exactly the declared files; unchanged assets/support files can be bound by exact hashes. UTF-8 templates preserve explicit BOMs, CRLFs and all other bytes; they do not normalize text.

One optional `{{NEW_OBJECT_ID}}` slot accepts `text-` followed by eight lowercase hexadecimal digits. All occurrences bind the same value, which must be absent from every baseline buffer. The rest of the file is literal expected source. No user-supplied regular expression is executed. This deliberately narrow ID rule and exact templates can exclude other valid source representations; they are not a general TeX inverse or semantics oracle.

Five regression checks cover the intended second occurrence, unexpected line-ending/support/extra-file changes, a stale baseline, injected text in an ID slot and collision with an existing ID. They preserve English/Arabic/Hebrew bytes. Together with nine PDF/replay checks they form a 14-check subset of the research-tool diagnostics; these are not final edit trials.

CLI usage from the repository root:

```text
python research/tools/source_contract.py BASELINE-SOURCES CANDIDATE-SOURCES PREDECLARED-CONTRACT.json NEW-RESULT.json
```

Use dedicated sources-only directories, separate from compiler outputs. Snapshots reject links/reparse points and enforce file/byte budgets. Results never overwrite an existing output. A wrong baseline is unresolved, not a successful unsafe-edit refusal. Before a final evaluation, prepare/review contracts independently of runtime outputs, seal hashes and combine source checks with the independent rendered reference. Integration into the final evaluation runner and broader grammar/ambiguity cases remain planned.
