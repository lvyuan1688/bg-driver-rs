//! bg-driver-rs CLI entry point.
//! Skeleton: build the host OS driver, run a screenshot, print dimensions.

use anyhow::Result;
use bg_driver::{Action, ActionResult, ComputerDriver};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "bg-driver-rs", version, about = "Background computer-use driver")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Capture a single screenshot and report its dimensions.
    Shot,
    /// Print the resolved driver name.
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    // Pick the host OS driver. We construct a stub here because the cfg-gated
    // backends require platform-specific deps; the real `src/main.rs` selects
    // the right driver based on `cfg!(target_os = ...)`.
    let driver = StubDriver;
    match cli.cmd.unwrap_or(Command::Info) {
        Command::Shot => match driver.execute(&Action::Screenshot).await? {
            ActionResult::Screenshot(s) => {
                println!("captured {}x{}", s.width, s.height);
            }
            _ => println!("unexpected result"),
        },
        Command::Info => {
            println!("driver = {}", driver.name());
        }
    }
    Ok(())
}

/// Stub driver used by the skeleton CLI. The real `src/main.rs` selects
/// `WindowsDriver`, `MacosDriver`, or `LinuxDriver` based on `cfg`.
struct StubDriver;

#[async_trait::async_trait]
impl ComputerDriver for StubDriver {
    fn name(&self) -> &str { "stub" }
    async fn execute(&self, _: &Action) -> Result<ActionResult> {
        Ok(ActionResult::Screenshot(bg_driver::Screenshot {
            width: 1280,
            height: 720,
            rgba: vec![0u8; 1280 * 720 * 4],
        }))
    }
    async fn screen_size(&self) -> Result<(u32, u32)> { Ok((1280, 720)) }
}
