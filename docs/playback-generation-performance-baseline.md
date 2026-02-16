# Playback Generation Throughput Baseline

This document captures the US-008 baseline and post-refactor throughput for
`sync-metadata` playback generation.

## Benchmark Command

Run from `gsnake-levels/` after building the binary:

```bash
cargo build --bin gsnake-levels
./target/debug/gsnake-levels sync-metadata --difficulty easy >/dev/null
```

Representative set:
- `levels/easy/*.json` (2 levels at capture time)
- `max_depth` comes from `sync_metadata` default (`500`)

## Snapshot

- Captured: `2026-02-16 21:51 CET`
- Host: `Linux (WSL2)`

### Before (per-level `cargo run --bin solve_level` spawn)

- Single run wall time: `0.49 s`

### After (in-process solver call)

- 10-run average wall time: `9.997 ms`
- 10-run min/max wall time: `9.597 ms` / `10.937 ms`

### Improvement

- Approximate speedup: `49x` (`0.49 s` to `~0.01 s`) on the same easy-level
  benchmark command.

## Re-Run Guidance

- Keep the command and difficulty unchanged when comparing against this
  baseline.
- Run after an initial warm build to avoid compile time skew.
