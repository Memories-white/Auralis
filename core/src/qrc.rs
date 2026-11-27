//! QRC DES variant adapted to Rust from jixunmoe-go/qrc (MIT).
//! Copyright (c) 2023 Jixun Wu. See docs/licenses/QRC-LICENSE.
use serde_json::{json, Value};
use std::io::Read;
include!("qrc_tables.rs");
fn mask(i: usize) -> u64 {
    1u64 << ((i & 32) + (31 - (i & 31)))
}
fn mapped(value: u64, table: &[u8]) -> u64 {
    let half = table.len() / 2;
    let mut out = 0;
    for (i, bit) in table.iter().enumerate() {
        if value & mask(*bit as usize) != 0 {
            out |= mask(i % half) << if i >= half { 32 } else { 0 };
        }
    }
    out
}
fn transform(bytes: &mut [u8], key: &[u8; 8], encrypt: bool) {
    let param = mapped(u64::from_le_bytes(*key), KEY_PERMUTATION_TABLE);
    let (mut c, mut d) = (param as u32, (param >> 32) as u32);
    let mut keys = [0; 16];
    for (i, shift) in KEY_RND_SHIFTS.iter().enumerate() {
        c = (c << shift) | ((c >> (28 - shift)) & 0xfffffff0);
        d = (d << shift) | ((d >> (28 - shift)) & 0xfffffff0);
        keys[if encrypt { i } else { 15 - i }] =
            mapped(((d as u64) << 32) | c as u64, KEY_COMPRESSION);
    }
    for chunk in bytes.chunks_exact_mut(8) {
        let mut state = mapped(u64::from_le_bytes(chunk.try_into().unwrap()), IP);
        for key in keys {
            let hi = (state >> 32) as u32;
            let lo = state as u32;
            let expanded = mapped(((hi as u64) << 32) | hi as u64, KEY_EXPANSION) ^ key;
            let mut next = 0u32;
            for (i, shift) in LARGE_STATE_SHIFTS.iter().enumerate() {
                next = (next << 4) | SBOXES[i][((expanded >> shift) & 63) as usize] as u32;
            }
            let mut permuted = 0u32;
            for (i, bit) in P_BOX.iter().enumerate() {
                if next & (mask(*bit as usize) as u32) != 0 {
                    permuted |= mask(i) as u32;
                }
            }
            state = (((permuted ^ lo) as u64) << 32) | hi as u64;
        }
        chunk.copy_from_slice(&mapped(state.rotate_left(32), IP_INV).to_le_bytes());
    }
}
pub fn decrypt(hex: &str) -> Option<String> {
    if hex.len() > 1024 * 1024 || hex.is_empty() || hex.len() % 16 != 0 {
        return None;
    }
    let mut bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
        .collect::<Option<_>>()?;
    transform(&mut bytes, b"!@#)(NHL", false);
    transform(&mut bytes, b"123ZXC!@", true);
    transform(&mut bytes, b"!@#)(*$%", false);
    let mut text = String::new();
    flate2::read::ZlibDecoder::new(bytes.as_slice())
        .take(2 * 1024 * 1024 + 1)
        .read_to_string(&mut text)
        .ok()?;
    (text.len() <= 2 * 1024 * 1024).then_some(text)
}
pub fn parse(xml: &str) -> Vec<Value> {
    let content = xml
        .split_once("LyricContent=\"")
        .and_then(|(_, s)| s.split_once('"').map(|(s, _)| s))
        .unwrap_or(xml)
        .replace("&#10;", "\n")
        .replace("&#13;", "\r")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    let mut lines = vec![];
    for raw in content.lines().take(4000) {
        let Some((tag, mut rest)) = raw.strip_prefix('[').and_then(|s| s.split_once(']')) else {
            continue;
        };
        let Some((time, duration)) = tag
            .split_once(',')
            .and_then(|(a, b)| Some((a.parse::<i32>().ok()?, b.parse::<i32>().ok()?)))
        else {
            continue;
        };
        if time < 0 || !(1..=3600000).contains(&duration) {
            continue;
        }
        let end = time.saturating_add(duration);
        let mut words = vec![];
        let mut text = String::new();
        let mut search = 0;
        while let Some(open) = rest
            .get(search..)
            .and_then(|s| s.find('('))
            .map(|i| i + search)
        {
            let word = &rest[..open];
            let tail = &rest[open + 1..];
            let Some((timing, next)) = tail.split_once(')') else {
                break;
            };
            let mut p = timing.split(',');
            let values = p
                .next()
                .and_then(|s| s.parse::<i32>().ok())
                .zip(p.next().and_then(|s| s.parse::<i32>().ok()));
            let Some((start, length)) = values else {
                search = open + 1;
                continue;
            };
            if words.len() >= 1000 {
                break;
            }
            text.push_str(word);
            let start = start.clamp(time, end);
            let stop = start.saturating_add(length.max(0)).clamp(start, end);
            if !word.is_empty() {
                words.push(json!({"text":word,"start":start,"end":stop}));
            }
            rest = next;
            search = 0;
        }
        if !text.trim().is_empty() && !words.is_empty() {
            lines.push(json!({"time":time,"end":end,"text":text,"words":words}));
        }
        if lines
            .iter()
            .map(|l| l["words"].as_array().map_or(0, Vec::len))
            .sum::<usize>()
            > 6000
        {
            return vec![];
        }
    }
    lines.sort_by_key(|l| l["time"].as_i64().unwrap_or(0));
    lines
}
pub fn lrc(lines: &[Value]) -> String {
    lines
        .iter()
        .map(|l| {
            let t = l["time"].as_i64().unwrap_or(0);
            format!(
                "[{:02}:{:02}.{:03}]{}",
                t / 60000,
                t / 1000 % 60,
                t % 1000,
                l["text"].as_str().unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upstream_cipher_vector() {
        let mut b = [1, 2, 3, 4, 5, 6, 7, 8];
        transform(&mut b, b"TEST!KEY", false);
        assert_eq!(b, [0xfd, 0x0e, 0x64, 0x06, 0x65, 0xbe, 0x74, 0x13]);
    }
    #[test]
    fn parses_absolute_word_timing_and_preserves_spaces() {
        let l=parse("<Qrc LyricContent=\"[1000,2000]Hello (1000,800)world(2000,1000)&#10;[4000,1000]你好(4000,1000)\"/>");
        assert_eq!(l.len(), 2);
        assert_eq!(l[0]["text"], "Hello world");
        assert_eq!(l[0]["words"][1]["start"], 2000);
        assert!(lrc(&l).starts_with("[00:01.000]Hello world"));
    }
    #[test]
    fn malformed_payload_is_not_a_timeline() {
        assert!(decrypt("garbage").is_none());
        assert!(parse("[00:01.0]line only").is_empty());
        assert!(parse("[100,0]x(100,0)").is_empty());
    }
    #[test]
    fn inspect_public_fixture_if_provided() {
        if let Ok(path) = std::env::var("AURALIS_QRC_FIXTURE") {
            let raw = std::fs::read_to_string(path).unwrap();
            let text = decrypt(raw.trim()).expect("public response must decrypt");
            let lines = parse(&text);
            assert!(lines.len() > 5);
            eprintln!(
                "QRC verified: {} lines / {} timed words",
                lines.len(),
                lines
                    .iter()
                    .map(|l| l["words"].as_array().unwrap().len())
                    .sum::<usize>()
            );
        }
    }
}
