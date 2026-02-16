# Solver Performance Baseline

This document tracks a repeatable runtime baseline for `solve_level` so future
optimization work can be compared against a known starting point.

## Benchmark Command

Run from repository root:

```bash
cargo run --manifest-path gsnake-levels/Cargo.toml --bin profile_solver -- \
  --levels-root gsnake-levels/levels \
  --difficulties easy,medium \
  --iterations 3 \
  --max-depth 120
```

Run from `gsnake-levels/`:

```bash
cargo run --bin profile_solver -- \
  --levels-root levels \
  --difficulties easy,medium \
  --iterations 3 \
  --max-depth 120
```

## Baseline Snapshot

- Captured: `2026-02-16 21:06:44 CET`
- OS: `Linux 6.6.87.2-microsoft-standard-WSL2 x86_64`
- CPU: `11th Gen Intel(R) Core(TM) i5-1135G7 @ 2.40GHz` (8 logical CPUs)

### Summary

- Levels benchmarked: `5`
- Total solves: `15` (`5 levels x 3 iterations`)
- Wall time: `47.622 s`
- Mean solve time: `3174.814 ms`

### Per-Difficulty Cumulative Time

- `easy`: `0.013 s`
- `medium`: `47.609 s`

### Hotspot Summary (Top 3)

1. `levels/medium/level-1769975926647-nv3aqb.json`  
   total `47.563 s`, avg `15854.362 ms` (min `15708.517 ms`, max `16131.814 ms`)
2. `levels/medium/level-1769976243963-g777pk.json`  
   total `0.026 s`, avg `8.545 ms` (min `7.893 ms`, max `9.122 ms`)
3. `levels/medium/level-1769976545160-dvt1ot.json`  
   total `0.020 s`, avg `6.642 ms` (min `4.934 ms`, max `9.748 ms`)

## Re-Run Guidance

- Keep `--levels-root`, `--difficulties`, `--iterations`, and `--max-depth`
  unchanged when comparing against this baseline.
- Run benchmarks on an idle machine; CPU contention can skew wall-time metrics.
