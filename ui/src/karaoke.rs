// SPDX-License-Identifier: Apache-2.0
// Word motion adapted from Melox LyricsView.kt, Melox contributors.
// Revision bfe7c5dc467b84d7a9df8e63c93208a3e6f61672; Rust and DOM geometry adaptations.
//! Genuine word timing only: ordinary LRC never gets fabricated word timestamps.
use serde_json::Value;
#[derive(Clone, Debug)]
pub struct Word {
    pub text: String,
    pub start: i32,
    pub end: i32,
}
pub fn parse(raw: &str, lines: &[(i32, String)]) -> Vec<Vec<Word>> {
    let parsed: Value = serde_json::from_str(raw).unwrap_or(Value::Null);
    lines
        .iter()
        .map(|(time, text)| {
            let matching = parsed.as_array().into_iter().flatten().find(|l| {
                l["time"].as_i64() == Some(*time as i64)
                    && l["text"].as_str().is_some_and(|s| s.trim() == text)
            });
            let words = matching
                .and_then(|l| l["words"].as_array())
                .map(|ws| {
                    ws.iter()
                        .take(1000)
                        .filter_map(|w| {
                            let start = i32::try_from(w["start"].as_i64()?).ok()?;
                            let end = i32::try_from(w["end"].as_i64()?).ok()?;
                            if end < start {
                                return None;
                            }
                            Some(Word {
                                text: w["text"].as_str()?.to_owned(),
                                start,
                                end,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if words
                .iter()
                .map(|w| w.text.as_str())
                .collect::<String>()
                .trim()
                == text
            {
                words
            } else {
                vec![]
            }
        })
        .collect()
}
pub fn fill(word: &Word, position: i32) -> i32 {
    if position < word.start {
        0
    } else if position >= word.end {
        1000
    } else {
        ((position - word.start) as i64 * 1000 / (word.end - word.start).max(1) as i64) as i32
    }
}
/// Presentation follows the real word interval. These are reveal fractions, not
/// invented character timestamps. Joined/nontrivial scripts remain shaped together.
pub fn pieces(text: &str) -> Vec<String> {
    if text
        .chars()
        .all(|c| c.is_ascii() || ('\u{3400}'..='\u{9fff}').contains(&c))
    {
        text.chars().map(|c| c.to_string()).collect()
    } else {
        vec![text.to_owned()]
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Effect {
    pub lift: f64,
    pub scale: f64,
    pub glow: f64,
    pub radius: f64,
}
pub fn effect(progress: f64, piece: usize, count: usize, duration: i32, reduced: bool) -> Effect {
    if reduced {
        return Effect {
            lift: 0.0,
            scale: 1.0,
            glow: 0.0,
            radius: 0.0,
        };
    }
    // Melox LyricsView.kt (Apache-2.0): staggered, overlapping character windows.
    let count = count.max(1);
    let window = if count <= 10 {
        0.68
    } else {
        4.0 / (count as f64 + 3.0)
    };
    let ratio = if count > 1 {
        piece as f64 / (count - 1) as f64
    } else {
        0.5
    };
    let p = ((progress - (1.0 - window) * ratio) / window).clamp(0.0, 1.0);
    let reveal = (progress * count as f64 - piece as f64).clamp(0.0, 1.0);
    let reveal = (1.0 - (reveal * std::f64::consts::PI).cos()) / 2.0;
    let pulse = (std::f64::consts::PI * p).sin().max(0.0).powi(2);
    // Long sustained words get a soft swell; short syllables only rise with reveal.
    let ratio = ((duration as f64 - 1000.0) / 2000.0).clamp(0.0, 1.0);
    let intensity = ratio * ratio * (3.0 - 2.0 * ratio);
    let sustained = duration >= 1000;
    Effect {
        lift: -(4.0 / 72.0) * reveal,
        scale: if sustained {
            1.0 + (0.06 + 0.06 * intensity) * pulse
        } else {
            1.0
        },
        glow: if sustained {
            (0.42 + 0.2 * intensity) * pulse
        } else {
            0.0
        },
        radius: 7.0 + 5.0 * intensity,
    }
}
/// Feather reveal over the measured, scale-expanded glyph bounds. Geometry is
/// captured once after mounting, never read from layout in the animation loop.
pub fn spatial_reveal(
    progress: f64,
    left: f64,
    right: f64,
    total: f64,
    font: f64,
    scale: f64,
) -> f64 {
    let width = (right - left).max(0.001) * scale;
    let left = (left + right - width) / 2.0;
    let feather = (font * 0.25).max(0.001);
    ((progress.clamp(0.0, 1.0) * (total + feather) - left) / (width + feather)).clamp(0.0, 1.0)
}
pub fn eligible(word: &Word) -> bool {
    let visible = word
        .text
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation())
        .count()
        .max(1);
    let duration = word.end - word.start;
    duration >= 1000
        && (word
            .text
            .chars()
            .any(|c| ('\u{3400}'..='\u{9fff}').contains(&c))
            || duration as f64 / visible as f64 > 200.0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reveal_has_spatial_lift_swell_and_stable_endpoints() {
        let peak = effect(0.5, 0, 1, 3000, false);
        assert!((peak.scale - 1.12).abs() < 1e-9);
        assert!((peak.glow - 0.62).abs() < 1e-9);
        assert_eq!(spatial_reveal(0.0, 0.0, 10.0, 50.0, 24.0, 1.0), 0.0);
        assert_eq!(spatial_reveal(1.0, 40.0, 50.0, 50.0, 24.0, 1.0), 1.0);
        assert!(
            spatial_reveal(0.5, 0.0, 30.0, 50.0, 24.0, 1.0)
                > spatial_reveal(0.5, 30.0, 35.0, 50.0, 24.0, 1.0)
        );
        let a = effect(0.25, 0, 2, 1800, false);
        let b = effect(0.25, 1, 2, 1800, false);
        assert!(a.lift < 0.0 && a.scale > 1.06 && a.glow > 0.3);
        assert_eq!(b.lift, 0.0);
        assert_eq!(b.scale, 1.0);
        let end = effect(1.0, 0, 2, 1800, false);
        assert_eq!(end.lift, -4.0 / 72.0);
        assert!((end.scale - 1.0).abs() < 1e-9);
        assert_eq!(effect(0.25, 0, 2, 1800, true).lift, 0.0);
        assert_eq!(pieces("你好!").len(), 3);
        assert_eq!(pieces("سلام").len(), 1);
        assert_eq!(pieces("e\u{301}").len(), 1);
    }
    #[test]
    fn progress_pauses_seeks_and_respects_real_gaps() {
        let w = Word {
            text: "word".into(),
            start: 1000,
            end: 2000,
        };
        assert_eq!(fill(&w, 900), 0);
        assert_eq!(fill(&w, 1500), 500);
        assert_eq!(fill(&w, 3000), 1000);
        assert_eq!(fill(&w, 1200), 200);
    }
    #[test]
    fn untimed_or_mismatched_text_never_gets_fake_progress() {
        let l = vec![(1000, "Hello".into())];
        assert!(parse("[]", &l)[0].is_empty());
        assert!(parse(
            r#"[{"time":1000,"text":"Hello","words":[{"text":"Other","start":1000,"end":2000}]}]"#,
            &l
        )[0]
        .is_empty());
    }
}
