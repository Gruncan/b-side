//! In-memory queue for one sitting.
//!
//! [`Session::from_feed`] loads the mock (or a future backend payload).
//! [`Session::decide`] and [`Session::undo`] are stubs for you to fill.
//! Playlist order is swipe order; an energy arc or harmonic sort belongs
//! on the server, which should return items already ordered.

use crate::contract::{Card, Choice, Decision, DeckFeed};

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub feed: DeckFeed,
    pub queue: Vec<Card>,
    pub playlist: Vec<Card>,
}

impl Session {
    pub fn from_feed(feed: DeckFeed) -> Self {
        let session_id = feed.session_id.clone();
        let queue = feed.cards.clone();
        Self {
            session_id,
            feed,
            queue,
            playlist: Vec::new(),
        }
    }

    pub fn current(&self) -> Option<&Card> {
        self.queue.first()
    }

    /// Record a swipe and return the body to `POST`.
    ///
    /// Keep appends the card to [`Session::playlist`]. Pass drops it. Later
    /// moves it to the back of the queue once; a second later on the same id
    /// drops it, so one card cannot loop forever. `heard_ms` is how far the
    /// preview played, `elapsed_ms` is how long the card was on top, and
    /// `play_count` is how many times playback started.
    ///
    /// The stub returns `None` and does not change the queue.
    pub fn decide(
        &mut self,
        choice: Choice,
        heard_ms: u32,
        elapsed_ms: u32,
        play_count: u32,
    ) -> Option<Decision> {
        let _ = (choice, heard_ms, elapsed_ms, play_count);
        None
    }

    /// Drop a kept card and return an [`Choice::Undo`] decision.
    /// Do not put the card back in the queue. The stub returns `None`.
    pub fn undo(&mut self, card_id: &str) -> Option<Decision> {
        let _ = card_id;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::bundled_feed;

    #[test]
    fn mock_feed_loads_as_an_empty_playlist() {
        let session = Session::from_feed(bundled_feed().expect("mock feed"));
        assert_eq!(session.session_id, "local-demo");
        assert_eq!(session.queue.len(), session.feed.cards.len());
        assert!(session.playlist.is_empty());
        assert_eq!(
            session.current().map(|card| card.id.as_str()),
            Some("catalog:track:glasshouse-hour")
        );
        assert!(session.queue.iter().any(|card| card.preview.is_none()));
    }
}
