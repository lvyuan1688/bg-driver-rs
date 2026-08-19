//! bg-driver: the `ComputerDriver` trait + per-OS backend skeletons.
//! The trait exposes the small set of primitives an agent needs to drive a
//! headless desktop: screenshot, mouse move/click, keyboard text, key tap,
//! scroll, and window focus. Each OS backend lives in its own file and is
//! selected at compile time via `cfg`.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsDriver;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacosDriver;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxDriver;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A captured screen image.
#[derive(Debug, Clone)]
pub struct Screenshot {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Button {
    Left,
    Right,
    Middle,
}

/// Key codes are platform-agnostic strings: "enter", "tab", "esc",
/// "shift", "ctrl", "alt", "cmd", or a single character like "a".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Screenshot,
    MouseMove { x: i32, y: i32 },
    MouseClick { x: i32, y: i32, button: Button },
    MouseScroll { dx: i32, dy: i32 },
    KeyTap { key: String },
    TypeText { text: String },
    FocusWindow { title: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionResult {
    Screenshot(Screenshot),
    Ok,
    Err(String),
}

/// The trait every OS backend implements.
#[async_trait]
pub trait ComputerDriver: Send + Sync {
    /// Backend name, e.g. `"windows"`, `"macos"`, `"linux"`.
    fn name(&self) -> &str;
    /// Execute an action.
    async fn execute(&self, action: &Action) -> Result<ActionResult>;
    /// Return the current screen size in pixels.
    async fn screen_size(&self) -> Result<(u32, u32)>;
}
