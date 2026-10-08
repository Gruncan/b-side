# Side A

A deck for previews. One card, a short listen, then keep, pass, or later. Keeps collect into a playlist the backend can save to whatever catalog it talks to. The wordmark is a placeholder. `copy.title` in the feed replaces it.

The static page is a picture of a session already two keeps in, so the playlist row is on screen. `ui/fixtures/deck.json` is the catalog the tests load. When you implement `start`, render that queue and start the playlist empty.

## CSS, not Sass

Style this with native CSS. Nesting, `@layer`, custom properties, `color-mix`, and OKLCH are baseline, so a Sass compile does not buy this page anything it needs. The split is:

| File | Role |
| --- | --- |
| `ui/styles/tokens.css` | Color, type, motion |
| `ui/styles/base.css` | Document, header, focus |
| `ui/styles/deck.css` | Card, drag, stamps, meters, actions |
| `ui/styles/playlist.css` | Rail on wide screens, sheet on narrow ones |
| `ui/styles/states.css` | Reduced motion, forced colors |

`@layer` order is tokens, base, components, states. Components do not fight each other with ever-higher specificity.

Custom properties are the runtime API. Sass variables disappear at compile time, so Rust cannot set them while a finger is down. `--x`, `--y`, `--rot`, `--lean-keep`, `--lean-pass`, `--c1` through `--c4`, `--value`, `--progress`, and `--glow` are the knobs. If you later want a build step for prefixes and minification, use Lightning CSS, not Sass.

## Rust in the browser

Do not start with Leptos, Dioxus, or Yew. Those frameworks own the markup (`view!`, `rsx!`, `html!`). This page is meant to stay HTML, with you writing the deck behavior.

`crates/deck-ui` is the skeleton:

- `contract` — serde types for the feed and the decision POST. This is real, and it is small, because the mock has to parse.
- `gesture::frame` and `gesture::release` — stubs. Fill these in. They must stay free of `web_sys` so `cargo test` can cover them without a browser.
- `session::from_feed` — loads a feed into a queue and an empty playlist. `decide` and `undo` are stubs.
- `start` — the `wasm-bindgen` export. It does not touch the document yet.

The only JavaScript you should add is the wasm-bindgen loader commented at the bottom of `ui/index.html`. That file is a shim the tool emits. Gesture math, playback, and `fetch` belong in Rust.

Leptos is the framework to pick up later, if the product grows past one deck into accounts and several routes. It is the closest of the Rust UI kits to a small WASM bundle and fine-grained updates. Dioxus is the one to pick if the same UI must also be a desktop or phone shell. Yew is the older virtual-DOM kit. None of them should replace this CSS.

`scripts/build-ui.sh` builds the crate for `wasm32-unknown-unknown` and writes `ui/pkg`, which is gitignored.

## What the page already does

The stylesheet reacts to attributes. Your `start` sets them. It does not restyle.

| You set | Effect |
| --- | --- |
| `[data-depth="0\|1\|2"]` on a card | Stack. Only depth 0 uses `--x` / `--y` / `--rot`. |
| `--x`, `--y`, `--rot` | Drag position. Defaults are zero, so the card springs back when you clear them and remove `data-dragging`. |
| `data-dragging="true"` | Turns the transform transition off while the pointer is down. |
| `--lean-keep`, `--lean-pass` | Opacity of the Keep and Pass stamps. |
| `data-lean="keep\|pass\|later\|none"` on `[data-stage]` | Highlights the matching action. |
| `data-leaving="true"` and `data-choice="keep\|pass\|later"` | Flies the card off. Remove the node on `transitionend` for `transform` (and on a timeout, because reduced motion may not fire one). |
| `data-enter="true"` | Rises the new top card out of the stack. |
| `--c1`…`--c4` | Artwork gradient when `artwork.url` is missing. Also set `--glow` on `<html>` from one of those colors. |
| `data-preview-state="paused\|playing\|ended\|missing"` | Play icon, pause icon, or the "No preview" label. |
| `--progress` on `[data-progress]` | Preview bar, 0 to 1. |
| `data-open="true"` on `[data-playlist-panel]` | Opens the sheet below 960px. At 960px and up the rail is always visible. |
| `hidden` on `[data-empty]`, `[data-deck]`, `[data-hint]`, `[data-actions-row]` | End of the stack. |

Pointer rules worth keeping: ignore the gesture when it starts on a `button` or an `a`, capture the pointer, and treat a movement under about 8px on the art as a tap that toggles the preview. Arrow left, arrow right, and arrow down map to pass, keep, and later. Space toggles the preview when a button is not focused. Do not autoplay the next card after the fly-off. The user-gesture token is gone by the time the transition ends, and a surprise start is worse than a play button.

A preview is optional. `Paper Boats` in the mock has `"preview": null`. Set `data-preview-state="missing"`, disable the play button, and still allow a swipe. Trait meters use `value` in 0..=1. BPM and loudness are normalized by the backend. `display` is the text ("128", "High"). Omit a trait you could not measure. Do not send a zero and pretend it was measured. Show at most four traits and two badges so the card does not collapse the art.

`is_safe_asset_url` and `is_safe_color` are the checks to use before assigning `img.src`, `audio.src`, or a CSS color. `is_safe_endpoint` is the check before `POST`.

Decisions `POST` to `{deck-endpoint}/decisions` when `<meta name="deck-endpoint">` is non-empty. Until then, keep state in memory. The playlist line reads "On this device", and "Saved" after a successful export. Export is the `actions[]` entry: `POST` its `href` with `{ session_id, card_ids }` in playlist order. Only `POST` is in scope. A `GET` action would navigate away from the deck.

The decision body is fixed by `ui/fixtures/decision.json`:

- `heard_ms` — how far into the preview playback got
- `elapsed_ms` — how long the card sat on top
- `play_count` — how many times playback started

A pass at under about 400ms of audio is a stronger signal than a pass after a full preview. A keep after barely any audio is a weaker one. `later` is not a vote on taste. `undo` reverses a keep.

## Catalog shape

Your Java client is one request class per Spotify call, executed through `SpotifyClient`, returning JSON. That is a reasonable way to learn the HTTP surface. It is a poor type for this screen. The deck should not import Spotify types.

```text
trait Catalog {
    async fn deck(&self, session: &Session) -> DeckFeed;
    async fn record(&self, decision: &Decision);
    async fn export(&self, playlist: &[Card]) -> Result<(), Error>;
}

trait Analyzer {
    async fn traits(&self, audio: &AudioRef) -> Vec<TraitMeter>;
}
```

Spotify is one `Catalog`. Another provider implements the same traits and returns the same `DeckFeed`. Join recordings by ISRC (`external_ids.isrc` on a Spotify track) so a row can move between providers.

## Audio features

On 27 November 2024 Spotify stopped serving these to new apps, and to development-mode apps that did not already have an extension pending. Apps that already had extended quota kept them. There is no official replacement. The post is [Introducing some changes to our Web API](https://developer.spotify.com/blog/2024-11-27-changes-to-the-web-api).

Removed for those apps:

- Audio Features and Audio Analysis
- Recommendations and the genre-seed list
- Related Artists
- Featured playlists, category playlists, and other algorithmic or Spotify-owned editorial playlists
- 30-second preview URLs on multi-get track objects (`SimpleTrack`)

A single-track fetch can still carry `preview_url`, and it is often null anyway. Search, playlist create, and the user-library endpoints were not on that list. Confirm each one against the current reference before you depend on it. `artists.genres` was not on the list either. Verify it before using it as a neighbor signal.

Do not build the product on unofficial "Spotify-shaped" APIs. The numbers are not Spotify's, the catalogs are unclear, and they disappear.

Own the analysis, on audio you are allowed to process (a preview URL when one exists, or a file the listener has):

- [Essentia](https://github.com/mtg/essentia) is the serious open-source stack: tempo, key, loudness, plus pretrained models for danceability and arousal/valence. It is AGPL. That license applies if you link it into something you ship.
- librosa is the small Python path for a prototype: beat tracking, RMS energy, chroma for a rough key. It does not emit Spotify's danceability or valence by itself.
- [Sonara](https://github.com/kkollsga/sonara) is a Rust analysis library with its own energy, danceability, and valence. Read the license and the definitions before treating those scores as drop-in replacements. They will not match Spotify's old numbers, and they do not need to. The ranker only needs them to be stable.

Normalize whatever you extract into `TraitMeter`. Missing dimensions are left out.

## Suggestion algorithm

This is the spec to implement on the server. The page never ranks.

Build one vector per track. Use only dimensions present on both vectors when you compare (masked cosine). Suggested axes, each about 0..=1 except the key pair:

```text
energy, valence, danceability, acousticness,
instrumentalness, speechiness,
tempo / 200 clamped to 0..=1,
loudness mapped from about -60dB..0dB into 0..=1,
sin(2π · key / 12), cos(2π · key / 12)
```

Key is a circle so C and B are neighbors. Major/minor can be a 0/1 if you have it.

Keep two centroids, `mu_keep` and `mu_pass`, with counts. Update with an exponential moving average. Weight `w` is the strength of the swipe, and `alpha` is `0.35` while that centroid has fewer than 8 votes, then `0.18`.

| Swipe | `w` |
| --- | --- |
| Pass, `heard_ms` under 15% of the preview and `elapsed_ms` under 1500 | 1.0 on `mu_pass` |
| Any other pass | 0.55 on `mu_pass` |
| Keep, heard ratio over 0.7 | 1.0 on `mu_keep` |
| Keep after little or no audio | 0.45 on `mu_keep` |
| Later | no centroid update |
| Undo | subtract the earlier keep update |

```text
mu := (1 - alpha · w) · mu + (alpha · w) · v
```

Score a candidate:

```text
cos(v, mu_keep) - 0.65 · cos(v, mu_pass) + 0.15 · novelty + jitter
```

`novelty` is 1 when the artist was not among the last 8 cards shown, otherwise 0.2. `jitter` is about ±0.05 for the first few keeps and ±0.02 after that, so the deck does not collapse onto one cluster. Drop anything already decided. A later'd card is allowed back, under the queue rule in `session::decide`.

Cold start, before four keeps: do not trust the centroid. Half the batch can come from the neighborhood of a seed (the listener's saved or top tracks, or the artist of something they just kept, via search and album tracks). Half should be wider. Spotify recommendations and related-artists are not available to a new app, so this expansion is yours.

The playlist the UI shows is swipe order. If you want a shaped set, do it when exporting, behind a query such as `?shape=arc`: arrange keeps so energy rises and then settles, and so adjacent Camelot keys differ by at most one step when both keys exist. That is a heuristic, not an optimizer. The page does not reorder.

## What you write next

1. `gesture::frame` and `gesture::release`, with tests for a small wobble, a right commit, a left commit, and a downward later that loses when the horizontal distance is larger.
2. `session::decide` and `session::undo`, with tests against `bundled_feed()`.
3. `start`: clone the templates, bind pointer and key events, drive `<audio data-preview-audio>`, and `POST` decisions when `deck-endpoint` is set.
4. A `Catalog` for Spotify that speaks the Web API you already modeled in spotify4Java, and returns `DeckFeed` rather than raw Spotify JSON.
5. An `Analyzer` behind the trait list, then the centroid ranker above.
