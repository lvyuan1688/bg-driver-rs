//! Linux backend skeleton. Real impl would use X11 (or XTest for synthetic
//! input). The skeleton returns a black 1280x720 screenshot.

use anyhow::Result;
use async_trait::async_trait;

use crate::{Action, ActionResult, ComputerDriver, Screenshot};

pub struct LinuxDriver;

#[async_trait]
impl ComputerDriver for LinuxDriver {
    fn name(&self) -> &str { "linux" }

    async fn execute(&self, action: &Action) -> Result<ActionResult> {
        Ok(match action {
            Action::Screenshot => ActionResult::Screenshot(Screenshot {
                width: 1280,
                height: 720,
                rgba: vec![0u8; 1280 * 720 * 4],
            }),
            Action::MouseMove { x, y } => {
                ActionResult::Err(format!("linux mouse_move stub x={x} y={y}"))
            }
            Action::MouseClick { x, y, button } => ActionResult::Err(format!(
                "linux mouse_click stub x={x} y={y} button={button:?}"
            )),
            Action::MouseScroll { dx, dy } => {
                ActionResult::Err(format!("linux scroll stub dx={dx} dy={dy}"))
            }
            Action::KeyTap { key } => ActionResult::Err(format!("linux key_tap stub {key}")),
            Action::TypeText { text } => {
                ActionResult::Err(format!("linux type_text stub {text:?}"))
            }
            Action::FocusWindow { title } => {
                ActionResult::Err(format!("linux focus_window stub {title:?}"))
            }
        })
    }

    async fn screen_size(&self) -> Result<(u32, u32)> {
        Ok((1280, 720))
    }
}
