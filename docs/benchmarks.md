# Benchmarks (v0.1.6)

> `crates/bg-bench` — latency + throughput micro-benchmarks for
> `bg-driver-rs`.

## What it measures

| Bench | What |
|-------|------|
| `bench_screenshot(driver, n)` | screenshot round-trip latency, n samples |
| `bench_key_tap(driver, key, n)` | key-tap RTT |
| `bench_mouse_click(driver, x, y, n)` | mouse-click latency at fixed point |
| `bench_throughput(label, window_ms, op)` | sustained ops/sec over a window |

## Statistics returned

Each `BenchResult` contains:

- `mean_ns` — arithmetic mean
- `p50_ns`, `p95_ns`, `p99_ns` — percentile latencies
- `total_ns` — wall-clock total
- `iterations` — count

## Usage

```rust
use bg_bench::bench_screenshot;

let driver = bg_driver::CdpBackend { endpoint: "http://localhost:9222".into() };
let r = bench_screenshot(&driver, 100).await;
println!("{}", serde_json::to_string_pretty(&r)?);
```

## Output format

```json
{
  "results": [
    { "label": "screenshot", "iterations": 100, "total_ns": 1234567,
      "mean_ns": 12345, "p50_ns": 12000, "p95_ns": 18000, "p99_ns": 21000 }
  ]
}
```

`BenchReport::to_json()` prints the whole batch.

## Edge cases

- `iterations = 0` → `summarize` returns zeros
- `op` returns `Err` → timing recorded as `Duration::ZERO`, counted as failure
- `throughput` with `window_ms = 0` → loop runs zero times, returns 0 iterations

## Not in v0.1.6

- Warmup phase (first N iterations discarded)
- Statistical outlier rejection (e.g. drop top 1%)
- Multi-driver comparison report
- JUnit / TAP output for CI integration
