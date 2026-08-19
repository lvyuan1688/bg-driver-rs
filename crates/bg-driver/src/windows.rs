//! Windows backend skeleton. Uses the `windows` crate to enumerate windows
//! and capture screenshots via GDI. The skeleton returns a black 1280x720
//! screenshot so the agent loop can be exercised without a real desktop.

use anyhow::Result;
use async_trait::async_trait;

use crate::{Action, ActionResult, ComputerDriver, Screenshot};

pub struct WindowsDriver;

#[async_trait]
impl ComputerDriver for WindowsDriver {
    fn name(&self) -> &str { "windows" }

    async fn execute(&self, action: &Action) -> Result<ActionResult> {
        Ok(match action {
            Action::Screenshot => ActionResult::Screenshot(Screenshot {
                width: 1280,
                height: 720,
                rgba: vec![0u8; 1280 * 720 * 4],
            }),
            Action::MouseMove { x, y } => {
                ActionResult::Err(format!("windows mouse_move stub x={x} y={y}"))
            }
            Action::MouseClick { x, y, button } => ActionResult::Err(format!(
                "windows mouse_click stub x={x} y={y} button={button:?}"
            )),
            Action::MouseScroll { dx, dy } => {
                ActionResult::Err(format!("windows scroll stub dx={dx} dy={dy}"))
            }
            Action::KeyTap { key } => ActionResult::Err(format!("windows key_tap stub {key}")),
            Action::TypeText { text } => {
                ActionResult::Err(format!("windows type_text stub {text:?}"))
            }
            Action::FocusWindow { title } => {
                ActionResult::Err(format!("windows focus_window stub {title:?}"))
            }
        })
    }

    async fn screen_size(&self) -> Result<(u32, u32)> {
        Ok((1280, 720))
    }
}
