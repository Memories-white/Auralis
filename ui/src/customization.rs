//! Rust-owned appearance schema. Native code only stores this bounded JSON object.
use serde_json::{json, Value};

pub struct OptionSpec {
    pub key: &'static str,
    pub title: &'static str,
    pub values: &'static [(&'static str, &'static str)],
    pub default: &'static str,
}
pub const OPTIONS: &[OptionSpec] = &[
    OptionSpec {
        key: "bar",
        title: "底栏样式",
        values: &[
            ("glass", "磨砂玻璃"),
            ("normal", "经典分栏"),
            ("floating", "悬浮胶囊"),
        ],
        default: "glass",
    },
    OptionSpec {
        key: "compact",
        title: "紧凑歌曲栏",
        values: &[
            ("off", "关闭 · 播放时短暂展开"),
            ("on", "开启 · 始终保持紧凑"),
        ],
        default: "off",
    },
    OptionSpec {
        key: "blur",
        title: "界面模糊效果",
        values: &[("on", "开启"), ("off", "关闭 · 降低渲染负担")],
        default: "on",
    },
    OptionSpec {
        key: "navLabels",
        title: "底栏文字",
        values: &[("on", "显示"), ("off", "仅图标")],
        default: "on",
    },
    OptionSpec {
        key: "background",
        title: "播放页背景",
        values: &[
            ("artwork", "封面弥散"),
            ("tonal", "主题色渐变"),
            ("flow", "动态流光"),
            ("plain", "纯色背景"),
        ],
        default: "artwork",
    },
    OptionSpec {
        key: "shape",
        title: "封面形状",
        values: &[
            ("default", "微圆角"),
            ("rounded", "圆角矩形"),
            ("circle", "圆形"),
        ],
        default: "rounded",
    },
    OptionSpec {
        key: "titleAlign",
        title: "播放页标题",
        values: &[("left", "左对齐"), ("center", "居中")],
        default: "left",
    },
    OptionSpec {
        key: "artwork",
        title: "歌曲列表封面",
        values: &[("on", "显示"), ("off", "隐藏")],
        default: "on",
    },
    OptionSpec {
        key: "duration",
        title: "歌曲列表时长",
        values: &[("off", "隐藏"), ("on", "显示")],
        default: "off",
    },
    OptionSpec {
        key: "lyricAlign",
        title: "歌词对齐",
        values: &[("left", "左对齐"), ("center", "居中")],
        default: "left",
    },
    OptionSpec {
        key: "lyricSize",
        title: "歌词字号",
        values: &[("24", "小"), ("28", "标准"), ("32", "大"), ("36", "特大")],
        default: "28",
    },
    OptionSpec {
        key: "lyricWeight",
        title: "歌词字重",
        values: &[("500", "适中"), ("600", "半粗"), ("700", "粗体")],
        default: "700",
    },
    OptionSpec {
        key: "lyricBlur",
        title: "歌词渐变模糊",
        values: &[("on", "开启 · 突出当前句"), ("off", "关闭")],
        default: "on",
    },
    OptionSpec {
        key: "translation",
        title: "歌词翻译",
        values: &[("on", "显示可用翻译"), ("off", "隐藏")],
        default: "on",
    },
    OptionSpec {
        key: "lyricScale",
        title: "当前句放大",
        values: &[("on", "开启"), ("off", "关闭")],
        default: "on",
    },
    OptionSpec {
        key: "motion",
        title: "页面动效",
        values: &[("full", "舒缓弹性"), ("reduced", "减少动态效果")],
        default: "full",
    },
    OptionSpec {
        key: "coverFlip",
        title: "切歌封面动效",
        values: &[("on", "轻柔翻转"), ("off", "关闭")],
        default: "on",
    },
    OptionSpec {
        key: "lyricControls",
        title: "歌词播放控件",
        values: &[("on", "显示"), ("off", "隐藏 · 保留底部进度")],
        default: "on",
    },
];
pub fn get<'a>(v: &'a Value, key: &str) -> &'a str {
    let Some(spec) = OPTIONS.iter().find(|s| s.key == key) else {
        return "";
    };
    v["uiPreferences"][key]
        .as_str()
        .filter(|value| spec.values.iter().any(|(id, _)| id == value))
        .unwrap_or(spec.default)
}
pub fn set(v: &mut Value, key: &str, value: &str) -> bool {
    if !OPTIONS
        .iter()
        .any(|s| s.key == key && s.values.iter().any(|(id, _)| *id == value))
    {
        return false;
    }
    if !v["uiPreferences"].is_object() {
        v["uiPreferences"] = json!({});
    }
    v["uiPreferences"][key] = json!(value);
    true
}
pub fn nodes(v: &Value, keys: &[&str]) -> Vec<Value> {
    keys.iter().filter_map(|key| OPTIONS.iter().find(|s| s.key==*key)).map(|spec| {
        let selected=get(v,spec.key);
        json!({"kind":"appearance-option","title":spec.title,"key":spec.key,"selected":selected,"items":spec.values.iter().map(|(id,title)|json!({"id":id,"title":title})).collect::<Vec<_>>()})
    }).collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_known_values_can_reach_style_properties() {
        let mut v = json!({"uiPreferences":{"shape":"url(malicious)","lyricSize":"999"}});
        assert_eq!(get(&v, "shape"), "rounded");
        assert_eq!(get(&v, "lyricSize"), "28");
        assert!(!set(&mut v, "shape", "invalid"));
        assert!(set(&mut v, "shape", "circle"));
        assert_eq!(get(&v, "shape"), "circle");
    }
}
