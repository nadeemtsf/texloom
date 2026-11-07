# Declared artifact-reference checking - development implementation

Prepared 5 November 2026 during the earlier test pause. Updated 7 November: `tests/test_study_records_v1.py` exercises these helpers with synthetic records/files. Its 13 passing checks include three reference-admission and two fingerprint checks; these overlap the existing 40-helper suite. They establish bounded regression behavior, not actual app execution, reviewed labels, sealed final inputs or a final admission result. The full coverage reconciliation is recorded in `docs/evidence/research-infrastructure-coverage-2026-11-07.json`.

## Purpose and limits

The v1 record format initially checks the shape of supplied hashes and relative evidence paths. It does not establish that referenced files exist or contain those bytes. The new reference checker is intended to add that narrower data-integrity step before development analysis. A complete reference check would establish agreement with the declared files at inspection time, not that an app ran, a source produced a binary, a gold expresses the intended edit, or an annotation was independently reviewed.

It always reports `final_evaluation_admitted: false`. It does not modify `evaluation/freeze.json`, save a project, invoke a compiler/renderer, run a candidate, extract an archive or follow commands in a manifest. The final study still requires reviewed labels/oracles, producer/build/environment evidence, pre-run sealing and actual application/native gates.

## Candidate manifest

`evaluation/artifact-manifest-template-v1.json` is an unfilled specification, not an admissible run. A filled manifest names one run and one variant, has kind `development_candidate_not_frozen`, declares the five v1 bindings, and explicitly lists files. The runner, protocol, fixture manifest and oracle manifest each have one binding role whose declared digest agrees with the binding field. Mixed runner variants need separate manifests; shared intentions/documents can be compared later without pooling them as independent samples.

Each file entry has exactly:

```text
path: relative POSIX-style name within the caller's explicit evidence root
bytes: declared byte length
sha256: lowercase SHA-256
role: runner | protocol | fixture_manifest | oracle_manifest |
      fixture_input | oracle_input | raw_evidence | source_snapshot
```

The fixture and oracle manifests have `schema_version: 1`, kind `synthetic_fixture_inputs` or `independent_oracle_inputs`, and a nonempty `files` list of `{path, bytes, sha256}` entries. `evaluation/input-manifest-template-v1.json` is the empty fixture-side example; use the oracle kind for a separately prepared oracle manifest. The lists must agree with the candidate manifest's corresponding input-role entries. This establishes closure of the **declared** lists, not completeness of all inputs actually read by a TeX engine or operating system.

Records must share the manifest's run, variant and bindings. Every raw-evidence reference must identify a listed file with the same path spelling and digest. Shared v1 record validation rejects duplicate/relabelled correctness cases without computing accuracy. Unrun records remain unrun even if their predeclared files can already be fingerprinted; no observation is inferred from file presence.

The implementation commit is a declared identifier. Neither a forty-character string nor a matching runner digest independently authenticates source-to-binary provenance. Existing publisher attestations likewise do not establish Texloom's complete build relation. Keep the relevant producer receipts/source snapshots and review this relation separately.

## Controlled file access

The checker reads only declared paths beneath one explicit root. It refuses absolute/traversal/ADS/device names, case-colliding declarations, linked/reparse-point components and changed/nonregular files. File identities are compared before/after hashing. No arbitrary directory scan or manifest command execution is used. The implementation budgets are 10,000 declared files, 128 MiB per general file, 50 MiB per JSON input and 2 GiB declared total.

The later source revision also uses each declared byte length as that file's read ceiling. A larger actual file remains unresolved rather than consuming a maximum-sized read for every understated declaration. Smaller changed files can still yield an observed mismatching digest. The aggregate declared budget therefore bounds successful input reads. Synthetic regression coverage does not establish exhaustive handling of every budget, race or filesystem condition.

These are accidental-misrouting/change checks for a controlled research directory. They are not an OS security boundary against hostile concurrent path replacement, a proof of input immutability for the original execution, or protection obtained merely by calling a directory synthetic. Actual documents remain outside the experimental corpus. Source bytes are only read; `.text` files must never be compiled.

Strict JSON reading rejects duplicate object keys and nonfinite numeric constants. Exclusive output creation preserves earlier reports. The CLI is designed to write a rejected/unresolved reference report for malformed or changed inputs rather than fabricate success. Raw observations distinguish declared from observed hashes and do not call a metadata mismatch an application refusal or a new correctness trial.

## Deferred interface

When tests are authorized, review/validate the implementation before relying on it. The intended invocation is:

```text
python research/tools/admit_records.py ROWS.json MANIFEST.json NEW-REPORT.json \
  --root EXPLICIT-CONTROLLED-RUN-DIRECTORY --development
```

Use continuation syntax appropriate to the shell; this illustrative command has not been run. A complete nonempty reference check would still not calculate accuracy or admit a final evaluation. The file report, analysis code versions, supplied labels and actual experiment evidence all retain separate identities. No new result belongs in the report's pending final-evaluation cells.
