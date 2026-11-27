# Melox lyric motion attribution

The Rust adaptation in `ui/src/lyric_motion.rs`, `ui/src/karaoke.rs` and associated focus/blur timing
rules are based on the following Melox files, each explicitly marked
`SPDX-License-Identifier: Apache-2.0`:

- `app/src/main/java/com/melox/player/ui/screen/playback/LyricRowMotion.kt`
- `app/src/main/java/com/melox/player/ui/screen/playback/LyricsView.kt`

Source: https://github.com/Inefy-03/Melox
Revision: `97cae45b6acf8a0d1a0a89282d1f7ca02d198ad7`
Word motion update: `bfe7c5dc467b84d7a9df8e63c93208a3e6f61672` (LyricsView.kt).
Attribution: Melox contributors.

Changes: Rust/WASM implementation, DOM placement, numerical spring integration,
bounded substeps, and CSS foreground blur/scale transitions. The Android Compose
renderer and raster caches are not included. Version 0.5.7 ports the overlapping
character animation windows and long-note pulse/intensity/glow formulas to Rust.
Feather reveal is mapped to cached DOM glyph advances, and physical effect size is
mapped to CSS font size. This does not assert identical font shaping or GPU output.

See `Melox-lyrics-Apache-2.0` for the license text. Other Melox files remain under
their respective upstream licenses; this notice does not relicense that repository.
