# MentisDB 0.10.8.53 (technical debt)

**Date:** September 16, 2026

## Fixes

### Vector sidecar WAL brick (issue #20)

A chain could brick itself with no crash and no concurrency. Compaction rewrote the JSON
snapshot with the full-corpus digest but never rebased the live in-memory sidecar, so the
first append after crossing the 32-record WAL threshold wrote a record whose
`prev_digest` no longer matched the snapshot. The next chain open failed with:

```
InvalidData("vector sidecar WAL digest chain mismatch")
```

Because the chain could not be opened, every write and most reads failed until the `.wal`
was removed by hand. Fixes:

- `compact_to_path` rebases the in-memory WAL chain head onto the persisted snapshot
  digest.
- The managed append path rebases the live cached sidecar when it schedules a compaction.
- A full snapshot write clears any sibling `.wal` before the atomic rename, so a crash can
  only leave a stale (rebuildable) snapshot without a WAL.
- An unreadable sidecar now rebuilds from the canonical thought log instead of aborting
  chain open.

### CLI `mentisdb add` missing `thought_type` (issue #17)

`mentisdb add` documented `--type` as defaulting to `fact-learned` but omitted the field
when `--type` was absent, so a plain `mentisdb add "text"` failed with HTTP 422
`missing field thought_type`. `add` now always sends `thought_type`, defaulting to
`fact-learned`, and validates an explicit `--type` locally with the list of valid types.

### One canonical `ThoughtType` parser

Removed three hand-maintained copies of the type table in `server.rs`, `llm.rs`, and
`lib.rs`. REST/MCP now accept `Goal` and `LLMExtracted`, which the server previously
rejected.

## Upgrade

```bash
cargo install mentisdb --locked --force
```

Restart the daemon.

## Benchmarks

Not re-run: technical debt release with no retrieval-scoring change. The local quality
gate (fmt, clippy `--all-targets --all-features -D warnings`, full test suite) is green.
