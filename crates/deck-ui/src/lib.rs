//! Skeleton for the deck.
//!
//! Markup, motion, and color live in `ui/`. This crate holds the payload
//! types, the mock feed, and the two hooks the page needs once you write
//! them: [`gesture::frame`] / [`gesture::release`], and [`session::decide`].

pub mod contract;
pub mod gesture;
pub mod session;

use wasm_bindgen::prelude::*;

/// Called from the loader described in `ui/index.html` after `scripts/build-ui.sh`.
///
/// Nothing here touches the document yet. The wiring belongs to you:
/// clone `#card-template` from [`contract::bundled_feed`], fill `[data-field]`,
/// and write a [`gesture::GestureFrame`] onto the top card as `--x`, `--y`,
/// `--rot`, `--lean-keep`, and `--lean-pass`. The stylesheet already consumes those.
#[wasm_bindgen]
pub fn start() {}
