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
}
pub fn effect(progress: f64, piece: usize, count: usize, duration: i32, reduced: bool) -> Effect {
    if reduced {
        return Effect {
            lift: 0.0,
            scale: 1.0,
            glow: 0.0,
        };
    }
    let p = (progress * count.max(1) as f64 - piece as f64).clamp(0.0, 1.0);
    let reveal = p * p * (3.0 - 2.0 * p);
    let pulse = (std::f64::consts::PI * p).sin().max(0.0).powi(2);
    // Long sustained words get a soft swell; short syllables only rise with reveal.
    let sustained = ((duration as f64 - 650.0) / 700.0).clamp(0.0, 1.0);
    Effect {
        lift: -0.065 * reveal,
        scale: 1.0 + 0.11 * pulse * sustained,
        glow: 0.48 * pulse * sustained,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reveal_has_spatial_lift_swell_and_stable_endpoints() {
        let a = effect(0.25, 0, 2, 1800, false);
        let b = effect(0.25, 1, 2, 1800, false);
        assert!(a.lift < 0.0 && a.scale > 1.09 && a.glow > 0.4);
        assert_eq!(b.lift, 0.0);
        assert_eq!(b.scale, 1.0);
        let end = effect(1.0, 0, 2, 1800, false);
        assert_eq!(end.lift, -0.065);
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
