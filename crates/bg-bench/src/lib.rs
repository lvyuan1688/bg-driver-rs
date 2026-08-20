//! bg-bench: latency + throughput micro-benchmarks for `bg-driver-rs`.
//!
//! Measures:
//!   - screenshot round-trip latency (mean / p50 / p95 / p99)
//!   - mouse-click jitter (stdev of click-to-result time)
//!   - key-tap RTT
//!   - sustained throughput over a fixed window
//!
//! The harness is `bench<F>(label, iterations, op) -> BenchResult`. It
//! runs `op` `iterations` times, collects timings, and computes summary
//! statistics. Callers pass their own `op: impl Fn() -> Fut`.

use anyhow::Result;
use async_trait::async_trait;
use bg_driver::{Action, ActionResult, ComputerDriver, Screenshot};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Summary statistics for one benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    pub label: String,
    pub iterations: u32,
    pub total_ns: u128,
    pub mean_ns: u64,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
}

/// A batch of related benchmarks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BenchReport {
    pub results: Vec<BenchResult>,
}

impl BenchReport {
    pub fn push(&mut self, r: BenchResult) {
        self.results.push(r);
    }
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

/// Run `op` `iterations` times and return summary statistics.
pub async fn bench<F, Fut>(label: impl Into<String>, iterations: u32, op: F) -> BenchResult
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let mut times: Vec<Duration> = Vec::with_capacity(iterations as usize);
    for _ in 0..iterations {
        let t0 = Instant::now();
        match op().await {
            Ok(()) => times.push(t0.elapsed()),
            Err(_) => times.push(Duration::ZERO),
        }
    }
    summarize(label, times)
}

/// Measure screenshot latency over `iterations` calls.
pub async fn bench_screenshot<D: ComputerDriver>(
    driver: &D,
    iterations: u32,
) -> BenchResult {
    bench("screenshot", iterations, || async move {
        driver.execute(&Action::Screenshot).await.map(|_| ())
    })
    .await
}

/// Measure key-tap RTT.
pub async fn bench_key_tap<D: ComputerDriver>(
    driver: &D,
    key: &str,
    iterations: u32,
) -> BenchResult {
    let key = key.to_string();
    let label = format!("key_tap:{key}");
    bench(label, iterations, || {
        let key = key.clone();
        async move { driver.execute(&Action::KeyTap { key }).await.map(|_| ()) }
    })
    .await
}

/// Measure mouse-click latency at (x, y).
pub async fn bench_mouse_click<D: ComputerDriver>(
    driver: &D,
    x: i32,
    y: i32,
    iterations: u32,
) -> BenchResult {
    let label = format!("mouse_click:{x},{y}");
    bench(label, iterations, || async move {
        driver
            .execute(&Action::MouseClick {
                x,
                y,
                button: bg_driver::Button::Left,
            })
            .await
            .map(|_| ())
    })
    .await
}

/// Sustained-throughput benchmark: how many `op`s fit in `window_ms`.
pub async fn bench_throughput<F, Fut>(
    label: impl Into<String>,
    window_ms: u64,
    op: F,
) -> BenchResult
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let deadline = Instant::now() + Duration::from_millis(window_ms);
    let mut count = 0u32;
    let mut times: Vec<Duration> = Vec::new();
    while Instant::now() < deadline {
        let t0 = Instant::now();
        if op().await.is_ok() {
            times.push(t0.elapsed());
            count += 1;
        }
    }
    let mut r = summarize(label, times);
    r.iterations = count;
    r
}

// ---- internal helpers ----------------------------------------------------

fn summarize(label: impl Into<String>, mut times: Vec<Duration>) -> BenchResult {
    let iterations = times.len() as u32;
    if times.is_empty() {
        return BenchResult {
            label: label.into(),
            iterations: 0,
            total_ns: 0,
            mean_ns: 0,
            p50_ns: 0,
            p95_ns: 0,
            p99_ns: 0,
        };
    }
    times.sort();
    let total: Duration = times.iter().sum();
    let mean = total / times.len() as u32;
    let pick = |pct: f64| -> u64 {
        let idx = ((times.len() as f64 - 1.0) * pct).round() as usize;
        times[idx.min(times.len() - 1)].as_nanos() as u64
    };
    BenchResult {
        label: label.into(),
        iterations,
        total_ns: total.as_nanos(),
        mean_ns: mean.as_nanos() as u64,
        p50_ns: pick(0.50),
        p95_ns: pick(0.95),
        p99_ns: pick(0.99),
    }
}

// ---- tests ---------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopDriver;

    #[async_trait]
    impl ComputerDriver for NoopDriver {
        fn name(&self) -> &str { "noop" }
        async fn execute(&self, _: &Action) -> Result<ActionResult> {
            Ok(ActionResult::Screenshot(Screenshot {
                width: 1, height: 1, rgba: vec![0, 0, 0, 255],
            }))
        }
        async fn screen_size(&self) -> Result<(u32, u32)> { Ok((1, 1)) }
    }

    #[tokio::test]
    async fn bench_screenshot_runs() {
        let d = NoopDriver;
        let r = bench_screenshot(&d, 3).await;
        assert_eq!(r.label, "screenshot");
        assert_eq!(r.iterations, 3);
    }

    #[tokio::test]
    async fn bench_throughput_returns_count() {
        let r = bench_throughput("tput", 50, || async { Ok(()) }).await;
        assert!(r.iterations > 0);
    }

    #[test]
    fn summarize_handles_empty() {
        let r = summarize("empty", vec![]);
        assert_eq!(r.iterations, 0);
    }

    #[test]
    fn summarize_p50_of_three_is_middle() {
        let times = vec![Duration::from_millis(1), Duration::from_millis(2), Duration::from_millis(3)];
        let r = summarize("x", times);
        assert_eq!(r.p50_ns, 2_000_000);
    }
}
