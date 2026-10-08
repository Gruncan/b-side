//! Pointer geometry. No DOM.
//!
//! Both functions are stubs. The thresholds and the CSS custom properties
//! they should drive are documented on the functions and in `DESIGN.md`.

/// Horizontal distance, in CSS pixels, that commits a keep or a pass.
pub const COMMIT_X: f64 = 108.0;
/// Downward distance that commits "later".
pub const COMMIT_Y: f64 = 132.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerDelta {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Commit {
    Keep,
    Pass,
    Later,
}

/// What the stylesheet needs while a finger is down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GestureFrame {
    pub x: f64,
    pub y: f64,
    pub rot_deg: f64,
    pub lean_keep: f64,
    pub lean_pass: f64,
}

impl GestureFrame {
    pub const fn rest() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rot_deg: 0.0,
            lean_keep: 0.0,
            lean_pass: 0.0,
        }
    }
}

/// Position of the card during a drag.
///
/// Resist the axis that is losing so a sideways swipe stays mostly horizontal.
/// A rotation around `x / 18` degrees, clamped near ±16, matches the stamps.
/// `lean_keep` is `x / COMMIT_X` clamped to 0..=1, and `lean_pass` is the
/// mirror of that. The stub returns [`GestureFrame::rest`].
pub fn frame(_delta: PointerDelta) -> GestureFrame {
    GestureFrame::rest()
}

/// Choice when the pointer is released.
///
/// Horizontal distance wins over vertical. At or past [`COMMIT_X`] to the
/// right is keep, to the left is pass. A mostly downward drag at or past
/// [`COMMIT_Y`] is later. Anything smaller is `None`, and the card springs
/// back. The stub always returns `None`.
pub fn release(_delta: PointerDelta) -> Option<Commit> {
    None
}
