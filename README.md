# Side A

A provider-neutral song deck. Someone hears a short preview, keeps or passes, and the keeps become a playlist.

The page is HTML and CSS. `crates/deck-ui` is a Rust skeleton: the payload types, a mock feed, and empty gesture and session hooks. The suggestion ranker and the catalog client are not in this repo. See [DESIGN.md](DESIGN.md).

```sh
cargo test
python3 -m http.server -d ui 8080
```

Open `http://127.0.0.1:8080`. Set `<meta name="deck-endpoint">` when your backend can serve the same JSON as `ui/fixtures/deck.json`.
