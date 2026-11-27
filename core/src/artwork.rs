//! Cover URL policy inspired by Melodist CoverUrlResolver (MIT).
use serde_json::{json, Value};
fn dimension(url: &str, size: u32) -> String {
    let Some(start) = url.find('R') else {
        return url.into();
    };
    let tail = &url[start + 1..];
    let count = tail
        .bytes()
        .take_while(|c| c.is_ascii_digit() || *c == b'x')
        .count();
    let token = &tail[..count];
    if token.contains('x')
        && token
            .split('x')
            .all(|p| !p.is_empty() && p.bytes().all(|c| c.is_ascii_digit()))
    {
        format!("{}R{size}x{size}{}", &url[..start], &tail[count..])
    } else {
        url.into()
    }
}
/// Stable album identity for lists, mini player and full player. Avatars retain
/// their own identity and arbitrary URLs/query strings are not rewritten.
pub fn identity(url: &str, avatar: bool) -> String {
    let secure = url.replacen("http://", "https://", 1);
    if avatar {
        secure
    } else {
        dimension(&secure, 500)
    }
}
/// Bounded, metadata-only startup catalog. Never persist sessions or stream URLs.
pub fn catalog(v: &Value) -> Value {
    fn rows(v: &Value, limit: usize) -> Value {
        json!(v
            .as_array()
            .into_iter()
            .flatten()
            .take(limit)
            .map(|row| {
                let mut clean = serde_json::Map::new();
                for key in [
                    "mid", "id", "title", "artist", "album", "albumMid", "coverUrl", "cover",
                    "duration", "vip", "dirid", "count", "saved", "kind",
                ] {
                    if let Some(value) = row
                        .get(key)
                        .filter(|v| v.is_string() || v.is_boolean() || v.is_number())
                    {
                        clean.insert(
                            key.into(),
                            if let Some(s) = value.as_str() {
                                json!(s.chars().take(2048).collect::<String>())
                            } else {
                                value.clone()
                            },
                        );
                    }
                }
                Value::Object(clean)
            })
            .collect::<Vec<_>>())
    }
    json!({"home":{"daily":rows(&v["home"]["daily"],30),"dailyId":v["home"]["dailyId"].as_u64().unwrap_or(0),"cards":rows(&v["home"]["cards"],24)},
        "discovery":rows(&v["discovery"],60),"library":{"likes":rows(&v["library"]["likes"],100),"playlists":rows(&v["library"]["playlists"],60)}})
}
pub fn candidates(v: &Value) -> Value {
    let url = identity(v["url"].as_str().unwrap_or(""), false);
    if !url.starts_with("https://") {
        return json!([]);
    }
    let sizes: &[u32] = if v["player"].as_bool().unwrap_or(false) {
        if v["metered"].as_bool().unwrap_or(true) {
            &[800, 500]
        } else {
            &[1200, 800, 500]
        }
    } else {
        &[500, 800]
    };
    let mut urls = Vec::new();
    for size in sizes {
        let candidate = dimension(&url, *size);
        if !urls.contains(&candidate) {
            urls.push(candidate);
        }
    }
    json!(urls)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_is_bounded_and_never_copies_credentials_or_streams() {
        let song =
            json!({"mid":"a","title":"Song","streamUrl":"secret","session":{"musicKey":"private"}});
        let c = catalog(&json!({"discovery":vec![song;90],"session":"private"}));
        assert_eq!(c["discovery"].as_array().unwrap().len(), 60);
        assert!(!c.to_string().contains("private"));
        assert!(!c.to_string().contains("secret"));
    }
    #[test]
    fn list_and_player_share_identity_without_changing_avatar() {
        assert_eq!(
            identity("http://y.gtimg.cn/T002R1200x1200M000abc.jpg", false),
            identity("https://y.gtimg.cn/T002R500x500M000abc.jpg", false)
        );
        assert_eq!(
            identity("https://example.com/avatar?s=640", true),
            "https://example.com/avatar?s=640"
        );
    }
    #[test]
    fn player_falls_back_and_respects_metering() {
        let v = candidates(
            &json!({"url":"https://y.gtimg.cn/T002R300x300M000abc.jpg","player":true,"metered":false}),
        );
        assert!(v[0].as_str().unwrap().contains("1200x1200"));
        assert_eq!(v.as_array().unwrap().len(), 3);
        let v = candidates(
            &json!({"url":"https://y.gtimg.cn/T002R300x300M000abc.jpg","player":true,"metered":true}),
        );
        assert!(v[0].as_str().unwrap().contains("800x800"));
    }
    #[test]
    fn opaque_urls_are_preserved_and_local_paths_rejected() {
        assert_eq!(
            candidates(&json!({"url":"https://example.com/a.jpg"})),
            json!(["https://example.com/a.jpg"])
        );
        assert_eq!(candidates(&json!({"url":"file:///secret"})), json!([]));
    }
}
