//! Read-only protocol probe. Prints counts only, never lyric content or credentials.
fn main() {
    let mid = std::env::args()
        .nth(1)
        .expect("usage: check_word_lyrics <public song MID>");
    let response = qqlyric_core::dispatch(
        "lyrics",
        &serde_json::json!({"mid":mid,"id":0,"session":{}}).to_string(),
    );
    let result: serde_json::Value = serde_json::from_str(&response).expect("JSON response");
    assert!(
        result["ok"].as_bool().unwrap_or(false),
        "lyric request failed"
    );
    let words = result["data"]["wordLyrics"]
        .as_array()
        .expect("word timeline");
    let count: usize = words
        .iter()
        .map(|l| l["words"].as_array().map_or(0, Vec::len))
        .sum();
    println!(
        "timed lines: {}, timed words: {}, has translation: {}",
        words.len(),
        count,
        result["data"]["translation"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
    );
    assert!(count > 0, "this track has no decoded word timing");
}
