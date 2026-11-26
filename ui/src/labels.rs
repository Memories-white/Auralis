use serde_json::Value;

pub fn label(key: &str) -> String {
    let source = include_str!("../../android/app/src/main/res/values/strings.xml");
    let needle = format!("<string name=\"{key}\">");
    source
        .split_once(&needle)
        .and_then(|(_, s)| s.split_once("</string>").map(|(value, _)| value))
        .unwrap_or(key)
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("\\'", "'")
}
pub fn text(v: &Value) -> String {
    if let Some(s) = v["value"].as_str() {
        return s.to_owned();
    }
    let mut result = label(v["resource"].as_str().unwrap_or(""));
    if let Some(args) = v["args"].as_array() {
        for (i, arg) in args.iter().enumerate() {
            let value = arg
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| arg.to_string());
            for suffix in ["s", "d"] {
                result = result.replace(&format!("%{}${suffix}", i + 1), &value);
            }
            if let Some(number) = arg.as_f64() {
                result = result.replace(&format!("%{}$.2f", i + 1), &format!("{number:.2}"));
            }
        }
    }
    result
}
pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn service_content_cannot_inject_markup() {
        assert_eq!(
            esc("<img src=x onerror=\"go()\">"),
            "&lt;img src=x onerror=&quot;go()&quot;&gt;"
        );
    }
    #[test]
    fn numeric_and_string_labels_are_formatted() {
        assert_eq!(text(&json!({"resource":"song_count","args":[30]})), "30 首");
        assert_eq!(
            text(&json!({"resource":"account_identifier","args":["12345"]})),
            "账号 12345"
        );
    }
}
