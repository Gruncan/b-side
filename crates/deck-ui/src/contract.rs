//! Provider-neutral payload the deck renders.
//!
//! The backend owns catalog lookups, audio analysis, and ranking. It hands the
//! page a [`DeckFeed`] and accepts a [`Decision`] back. Nothing in here is
//! Spotify-shaped on purpose: a trait meter is a normalized `value` plus an
//! optional display string, and a preview is just a URL that might be missing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct DeckFeed {
    pub session_id: String,
    pub provider: Provider,
    pub copy: Copy,
    pub actions: Vec<Action>,
    pub cards: Vec<Card>,
}

impl Default for DeckFeed {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            provider: Provider::default(),
            copy: Copy::default(),
            actions: Vec::new(),
            cards: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Provider {
    pub id: String,
    pub label: String,
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            id: "catalog".into(),
            label: "Catalog".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Copy {
    pub title: String,
    pub empty_title: String,
    pub empty_body: String,
    pub pass_label: String,
    pub keep_label: String,
    pub later_label: String,
}

impl Default for Copy {
    fn default() -> Self {
        Self {
            title: "Side A".into(),
            empty_title: "That's the stack".into(),
            empty_body: "Everything you kept is in the playlist.".into(),
            pass_label: "Pass".into(),
            keep_label: "Keep".into(),
            later_label: "Later".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub method: String,
    pub href: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Card {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artists: Vec<String>,
    #[serde(default)]
    pub artwork: Artwork,
    #[serde(default)]
    pub preview: Option<Preview>,
    #[serde(default)]
    pub traits: Vec<TraitMeter>,
    #[serde(default)]
    pub badges: Vec<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct Artwork {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub colors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Preview {
    pub url: String,
    #[serde(default)]
    pub duration_ms: Option<u32>,
}

/// One bar on the card.
///
/// `value` is always 0 to 1 so the stylesheet can draw the meter without
/// knowing what a BPM or a loudness is. `display` is the text beside it
/// ("128", "High"). Omit a trait the analyzer could not measure; do not send
/// a zero and pretend it was measured.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct TraitMeter {
    pub id: String,
    pub label: String,
    pub value: f64,
    #[serde(default)]
    pub display: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Link {
    pub label: String,
    pub href: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Choice {
    Keep,
    Pass,
    Later,
    Undo,
}

/// What the page knows about a swipe. The ranker turns this into a weight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Decision {
    pub session_id: String,
    pub card_id: String,
    pub choice: Choice,
    /// How far into the preview playback got, in milliseconds.
    pub heard_ms: u32,
    /// Wall-clock time the card was on top before the choice.
    pub elapsed_ms: u32,
    pub play_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportRequest<'a> {
    pub session_id: &'a str,
    pub card_ids: Vec<&'a str>,
}

pub fn bundled_feed() -> Result<DeckFeed, serde_json::Error> {
    serde_json::from_str(include_str!("../../../ui/fixtures/deck.json"))
}

/// Artwork and preview URLs that the page is willing to assign to `src`.
pub fn is_safe_asset_url(url: &str) -> bool {
    let url = url.trim();
    if url.is_empty() || url.contains("..") || url.contains('\\') {
        return false;
    }
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("javascript:") || lower.starts_with("data:") || lower.starts_with("//") {
        return false;
    }
    lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("fixtures/")
        || lower.starts_with("./fixtures/")
}

pub fn is_safe_color(color: &str) -> bool {
    let bytes = color.as_bytes();
    if bytes.first() != Some(&b'#') {
        return false;
    }
    let digits = &bytes[1..];
    matches!(digits.len(), 3 | 4 | 6 | 8) && digits.iter().all(|b| b.is_ascii_hexdigit())
}

/// POST targets the page is willing to call: a site path, or an http(s) URL.
pub fn is_safe_endpoint(url: &str) -> bool {
    let url = url.trim();
    if url.starts_with('/') && !url.starts_with("//") && !url.contains("..") {
        return true;
    }
    is_safe_asset_url(url)
}

pub fn unit_interval(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_matches_the_contract() {
        let feed = bundled_feed().expect("fixture parses");
        assert_eq!(feed.provider.id, "catalog");
        assert!(feed.cards.len() >= 4);
        assert!(feed.cards.iter().any(|card| card.preview.is_none()));
        let ui = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ui");
        for card in &feed.cards {
            let url = card
                .artwork
                .url
                .as_deref()
                .unwrap_or_else(|| panic!("{} has no cover", card.id));
            assert!(is_safe_asset_url(url), "{url}");
            assert!(ui.join(url).is_file(), "missing cover {url}");
        }
        assert!(feed.cards.iter().all(|card| {
            card.traits
                .iter()
                .all(|meter| meter.value >= 0.0 && meter.value <= 1.0)
        }));
    }

    #[test]
    fn asset_urls_reject_scriptable_schemes() {
        assert!(is_safe_asset_url("fixtures/tones/glasshouse.wav"));
        assert!(is_safe_asset_url("https://cdn.example/preview.mp3"));
        assert!(!is_safe_asset_url("javascript:alert(1)"));
        assert!(!is_safe_asset_url("data:text/html,hi"));
        assert!(!is_safe_asset_url("//evil.example/a.mp3"));
        assert!(!is_safe_asset_url("fixtures/../secret"));
    }

    #[test]
    fn colors_are_hex_only() {
        assert!(is_safe_color("#abc"));
        assert!(is_safe_color("#e7a07a"));
        assert!(!is_safe_color("red"));
        assert!(!is_safe_color("#gg0000"));
        assert!(!is_safe_color("expression(alert(1))"));
    }

    #[test]
    fn endpoints_allow_app_paths_and_reject_protocol_relative() {
        assert!(is_safe_endpoint("/api/deck/decisions"));
        assert!(is_safe_endpoint("https://api.example/deck"));
        assert!(!is_safe_endpoint("//evil.example/post"));
        assert!(!is_safe_endpoint("javascript:alert(1)"));
    }

    #[test]
    fn mock_decision_matches_the_post_body() {
        let decision = Decision {
            session_id: "local-demo".into(),
            card_id: "catalog:track:glasshouse-hour".into(),
            choice: Choice::Keep,
            heard_ms: 8_200,
            elapsed_ms: 10_440,
            play_count: 1,
        };
        let encoded = serde_json::to_value(&decision).expect("encode");
        let expected: serde_json::Value =
            serde_json::from_str(include_str!("../../../ui/fixtures/decision.json"))
                .expect("fixture");
        assert_eq!(encoded, expected);
    }
}
