//! bg-input-chain — input action chain recording and replay for bg-driver-rs.
//!
//! An `InputAction` is a single low-level input event (mouse move, key
//! down, scroll, etc.). A `Chain` is an ordered sequence with optional
//! inter-action delays. `Chain::replay` yields actions in order, sleeping
//! the recorded gap between each — used for deterministic UI automation
//! regression tests.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A button on the mouse or keyboard.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Button {
    Left,
    Right,
    Middle,
    Key(u32),
}

/// A single low-level input action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputAction {
    /// Action variant.
    pub kind: ActionKind,
    /// Monotonic timestamp (ms since chain start).
    pub ts_ms: u64,
}

/// Variants of input action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionKind {
    /// Mouse move to absolute pixel coords.
    MouseMove { x: i32, y: i32 },
    /// Button press.
    Down(Button),
    /// Button release.
    Up(Button),
    /// Vertical scroll by `delta` lines (negative = up).
    Scroll { delta: i32 },
    /// Text input — typed UTF-8 string.
    TypeText(String),
}

impl InputAction {
    pub fn new(kind: ActionKind, ts_ms: u64) -> Self {
        Self { kind, ts_ms }
    }

    /// Milliseconds gap between this action and the previous one.
    pub fn gap_after_prev(&self, prev_ts: u64) -> u64 {
        self.ts_ms.saturating_sub(prev_ts)
    }
}

/// A recorded chain of input actions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Chain {
    /// Actions ordered by `ts_ms` ascending.
    pub actions: Vec<InputAction>,
}

impl Chain {
    pub fn new() -> Self {
        Self { actions: Vec::new() }
    }

    /// Record an action at the given timestamp.
    pub fn record(&mut self, action: InputAction) {
        self.actions.push(action);
        // Keep sorted by ts_ms.
        let n = self.actions.len();
        if n > 1 && self.actions[n - 1].ts_ms < self.actions[n - 2].ts_ms {
            self.actions.sort_by_key(|a| a.ts_ms);
        }
    }

    /// Number of recorded actions.
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// True if no actions are recorded.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Total duration of the chain (ms from first to last action).
    pub fn duration_ms(&self) -> u64 {
        match (self.actions.first(), self.actions.last()) {
            (Some(a), Some(b)) => b.ts_ms.saturating_sub(a.ts_ms),
            _ => 0,
        }
    }

    /// Compress the chain: drop `MouseMove` actions that are within
    /// `threshold_ms` of the previous one (keep only the last in a burst).
    pub fn compress_mouse_moves(&mut self, threshold_ms: u64) {
        let mut kept: Vec<InputAction> = Vec::with_capacity(self.actions.len());
        for action in self.actions.iter().cloned() {
            let is_move = matches!(action.kind, ActionKind::MouseMove { .. });
            if is_move {
                // Look-ahead: is the next action also a move within threshold?
                let next = kept.last().filter(|p| matches!(p.kind, ActionKind::MouseMove { .. }));
                if let Some(prev) = next {
                    if action.gap_after_prev(prev.ts_ms) <= threshold_ms {
                        // Replace previous move with this one (coalesce).
                        let last = kept.last_mut().unwrap();
                        last.ts_ms = action.ts_ms;
                        last.kind = action.kind;
                        continue;
                    }
                }
            }
            kept.push(action);
        }
        self.actions = kept;
    }

    /// Replay the chain, yielding actions with the original gaps.
    /// Caller is responsible for actually sleeping between yields.
    pub fn replay(&self) -> impl Iterator<Item = (u64, &InputAction)> {
        let mut prev_ts: Option<u64> = None;
        self.actions.iter().map(move |a| {
            let gap = prev_ts.map(|p| a.gap_after_prev(p)).unwrap_or(0);
            prev_ts = Some(a.ts_ms);
            (gap, a)
        })
    }

    /// Filter actions by predicate (returns a new chain).
    pub fn filter<F: Fn(&InputAction) -> bool>(&self, pred: F) -> Self {
        Self {
            actions: self.actions.iter().filter(|a| pred(a)).cloned().collect(),
        }
    }

    /// All `TypeText` payloads concatenated.
    pub fn typed_text(&self) -> String {
        let mut s = String::new();
        for a in &self.actions {
            if let ActionKind::TypeText(t) = &a.kind {
                s.push_str(t);
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mv(x: i32, y: i32, ts: u64) -> InputAction {
        InputAction::new(ActionKind::MouseMove { x, y }, ts)
    }

    fn down(b: Button, ts: u64) -> InputAction {
        InputAction::new(ActionKind::Down(b), ts)
    }

    #[test]
    fn record_sorts_by_ts() {
        let mut c = Chain::new();
        c.record(mv(10, 10, 100));
        c.record(mv(20, 20, 50)); // out of order
        assert_eq!(c.actions[0].ts_ms, 50);
        assert_eq!(c.actions[1].ts_ms, 100);
    }

    #[test]
    fn duration_ms_handles_empty() {
        assert_eq!(Chain::new().duration_ms(), 0);
    }

    #[test]
    fn duration_ms_first_to_last() {
        let mut c = Chain::new();
        c.record(mv(0, 0, 100));
        c.record(mv(10, 10, 500));
        assert_eq!(c.duration_ms(), 400);
    }

    #[test]
    fn replay_yields_gaps() {
        let mut c = Chain::new();
        c.record(mv(0, 0, 0));
        c.record(mv(1, 1, 50));
        c.record(mv(2, 2, 200));
        let gaps: Vec<u64> = c.replay().map(|(g, _)| g).collect();
        assert_eq!(gaps, vec![0, 50, 150]);
    }

    #[test]
    fn compress_coalesces_bursts() {
        let mut c = Chain::new();
        // Three moves within 10ms of each other, then a fourth 100ms later.
        c.record(mv(0, 0, 0));
        c.record(mv(5, 5, 5));
        c.record(mv(10, 10, 8));
        c.record(mv(20, 20, 108));
        c.compress_mouse_moves(10);
        // Expected: 2 moves (coalesced first burst → last, then 108ms one).
        let moves: Vec<_> = c.actions.iter().filter(|a| matches!(a.kind, ActionKind::MouseMove { .. })).collect();
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0].ts_ms, 8); // kept last of first burst
        assert_eq!(moves[1].ts_ms, 108);
    }

    #[test]
    fn compress_preserves_non_moves() {
        let mut c = Chain::new();
        c.record(mv(0, 0, 0));
        c.record(down(Button::Left, 5));
        c.record(mv(10, 10, 10));
        c.compress_mouse_moves(100);
        assert_eq!(c.len(), 3);
        // Middle action still a Down.
        assert!(matches!(c.actions[1].kind, ActionKind::Down(_)));
    }

    #[test]
    fn typed_text_concatenates() {
        let mut c = Chain::new();
        c.record(InputAction::new(ActionKind::TypeText("Hello, ".into()), 0));
        c.record(InputAction::new(ActionKind::TypeText("World!".into()), 100));
        assert_eq!(c.typed_text(), "Hello, World!");
    }

    #[test]
    fn filter_returns_subset() {
        let mut c = Chain::new();
        c.record(mv(0, 0, 0));
        c.record(down(Button::Left, 10));
        let downs = c.filter(|a| matches!(a.kind, ActionKind::Down(_)));
        assert_eq!(downs.len(), 1);
    }

    #[test]
    fn serde_roundtrip() {
        let mut c = Chain::new();
        c.record(mv(0, 0, 0));
        c.record(down(Button::Left, 10));
        let json = serde_json::to_string(&c).unwrap();
        let back: Chain = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn gap_after_prev_saturates() {
        let a = InputAction::new(ActionKind::Scroll { delta: 1 }, 100);
        assert_eq!(a.gap_after_prev(200), 0);
        assert_eq!(a.gap_after_prev(50), 50);
    }
}
