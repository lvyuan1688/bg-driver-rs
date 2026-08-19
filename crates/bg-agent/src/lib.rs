//! bg-agent: agent loop on top of `bg-driver::ComputerDriver`.
//! Think → Act → Observe state machine.

use anyhow::Result;
use async_trait::async_trait;
use bg_driver::{Action, ActionResult, ComputerDriver};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Think,
    Act,
    Observe,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub phase: Phase,
    pub action: Option<Action>,
    pub result: Option<ActionResult>,
}

pub async fn run_loop<B, F, Fut>(driver: &B, mut decide: F) -> Result<Vec<Step>>
where
    B: ComputerDriver,
    F: FnMut(&[Step]) -> Fut,
    Fut: std::future::Future<Output = Result<Phase>>,
{
    let mut history = Vec::new();
    for _ in 0..100 {
        let phase = decide(&history).await?;
        if phase == Phase::Done {
            history.push(Step { phase, action: None, result: None });
            break;
        }
        let action = Action::Screenshot;
        let result = driver.execute(&action).await?;
        history.push(Step { phase, action: Some(action), result: Some(result) });
    }
    Ok(history)
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
    async fn loop_terminates() {
        let d = Stub;
        let r = run_loop(&d, |_: &[Step]| async move { Ok(Phase::Done) }).await.unwrap();
        assert_eq!(r.len(), 1);
    }
}
