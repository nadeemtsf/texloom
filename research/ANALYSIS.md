# Development annotation analysis

`tools/summarize_annotations.py` implements the draft correctness denominators. It does not run experiments, establish interaction provenance or independently adjudicate a candidate. It requires explicit `--development`; integration with a reviewed frozen evaluation remains planned.

Supply a JSON list of annotated correctness cases. The unfilled object in `evaluation/outcome-schema.json` is a specification example, not input data. Cases are separated by study arm, contract family, comparison variant, engine, language and evidence origin. A duplicate case within a stratum is rejected, so timing repetitions cannot inflate the correctness sample. Document counts remain distinct from case counts.

An empty dataset has no results. A group containing an unrun case gets no success rates. Zero accepted candidates means the accepted-error fraction is not applicable; a deny-all system still has zero useful valid acceptance. Accepted cases with unresolved correctness get error bounds and an unresolved safety gate, rather than a zero-error claim. Compile failures, timeouts and crashes retain their own categories and do not become valid refusals. Conflicting oracle/preservation labels are rejected.

Five unit checks exercise these distinctions using explicitly synthetic annotations. No actual app observations or final-study rows have been summarized. Together with the separate nine PDF/replay and five source-contract diagnostics, 19 research-tool checks pass. These counts are not an accuracy sample.

```text
python research/tools/summarize_annotations.py ANNOTATED-ROWS.json NEW-OUTPUT.json --development
```

The output never overwrites earlier results. No strata are pooled into a headline rate, and this prototype computes no end-to-end latency or statistical confidence intervals. Those require separately recorded timing repetitions and a reviewed clustered analysis. Final evidence admission, sealed manifest checks and independent label review remain prerequisites for a final report.
