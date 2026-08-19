//! bg-bench: micro-benchmark harness for `bg-driver-rs`.
//! Measures screenshot latency, mouse-move jitter, and key-tap RTT.

use anyhow::Result;
use async_trait::async_trait;
use bg_driver::{Action, ActionResult, ComputerDriver};
use std::time::{Duration, Instant};

pub struct BenchResult {
    pub label: String,
    pub iterations: u32,
    pub total: Duration,
    pub mean: Duration,
}

pub async fn bench_screenshot<D: ComputerDriver>(driver: &D, iterations: u32) -> Result<BenchResult> {
    let mut total = Duration::ZERO;
    for _ in 0..iterations {
        let t0 = Instant::now();
        let _ = driver.execute(&Action::Screenshot).await?;
        total += t0.elapsed();
    }
    let mean = total / iterations;
    Ok(BenchResult {
        label: "screenshot".into(),
        iterations,
        total,
        mean,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Stub;
    #[async_trait]
    impl ComputerDriver for Stub {
        fn name(&self) -> &str { "stub" }
        async fn execute(&self, _: &Action) -> Result<ActionResult> {
            Ok(ActionResult::Ok)
        }
        async fn screen_size(&self) -> Result<(u32, u32)> { Ok((1280, 720)) }
    }

    #[tokio::test]
    async fn bench_runs() {
        let r = bench_screenshot(&Stub, 3).await.unwrap();
        assert_eq!(r.iterations, 3);
    }
}
