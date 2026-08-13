//! bg-driver-rs - background computer-use driver (macOS/Windows/Linux)
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bg-driver-rs", version, about = "Background computer-use driver")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Start the background driver daemon
    Daemon { #[arg(default_value = "/tmp/bg-driver.sock")] socket: String },
    /// Take a screenshot of the current desktop
    Screenshot { out: String },
    /// Move the cursor and click at (x,y)
    Click { x: u32, y: u32 },
    /// Type a string of keys
    Type { text: String },
}

fn main() {
    match Cli::parse().cmd.unwrap_or(Cmd::Daemon { socket: "/tmp/bg-driver.sock".into() }) {
        Cmd::Daemon { socket } => println!("[daemon] listening on {socket} (stub)"),
        Cmd::Screenshot { out } => println!("[screenshot] -> {out} (stub)"),
        Cmd::Click { x, y } => println!("[click] ({x},{y}) (stub)"),
        Cmd::Type { text } => println!("[type] {text} (stub)"),
    }
}
