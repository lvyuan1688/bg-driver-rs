//! bg-sandbox: process + filesystem isolation for `bg-driver-rs`.
//! The skeleton exposes a `Sandbox` trait with a no-op `Passthrough` impl.

use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Root directory the sandbox considers "home".
    fn root(&self) -> &PathBuf;
    /// Spawn a command inside the sandbox. Returns its exit code.
    async fn spawn(&self, cmd: &str, args: &[&str]) -> Result<i32>;
    /// Snapshot the sandbox state to a directory.
    async fn snapshot(&self, dst: &PathBuf) -> Result<()>;
}

pub struct Passthrough {
    pub root: PathBuf,
}

#[async_trait]
impl Sandbox for Passthrough {
    fn root(&self) -> &PathBuf { &self.root }

    async fn spawn(&self, cmd: &str, args: &[&str]) -> Result<i32> {
        let status = tokio::process::Command::new(cmd).args(args).status().await?;
        Ok(status.code().unwrap_or(-1))
    }

    async fn snapshot(&self, _dst: &PathBuf) -> Result<()> {
        // no-op in the skeleton
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_root() {
        let s = Passthrough { root: PathBuf::from("/tmp") };
        assert_eq!(s.root(), &PathBuf::from("/tmp"));
    }
}
