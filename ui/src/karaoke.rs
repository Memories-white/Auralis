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
#[cfg(test)]
mod tests {
    use super::*;
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
