# Melox lyric motion attribution

The Rust adaptation in `ui/src/lyric_motion.rs` and associated focus/blur timing
rules are based on the following Melox files, each explicitly marked
`SPDX-License-Identifier: Apache-2.0`:

- `app/src/main/java/com/melox/player/ui/screen/playback/LyricRowMotion.kt`
- `app/src/main/java/com/melox/player/ui/screen/playback/LyricsView.kt`

Source: https://github.com/Inefy-03/Melox
Revision: `97cae45b6acf8a0d1a0a89282d1f7ca02d198ad7`
Attribution: Melox contributors.

Changes: Rust/WASM implementation, DOM placement, numerical spring integration,
bounded substeps, and CSS foreground blur/scale transitions. The Android Compose
renderer, raster caches and word-level timestamp renderer are not included.

See `Melox-lyrics-Apache-2.0` for the license text. Other Melox files remain under
their respective upstream licenses; this notice does not relicense that repository.
