//! Rust-owned appearance schema. Native code only stores this bounded JSON object.
//! Every option is validated here before it can reach a CSS custom property.
use serde_json::{json, Value};

pub struct OptionSpec {
    pub key: &'static str,
    pub title: &'static str,
    pub values: &'static [(&'static str, &'static str, &'static str)],
    pub default: &'static str,
}
pub const OPTIONS: &[OptionSpec] = &[
    OptionSpec {
        key: "bar",
        title: "底栏样式",
        values: &[
            ("glass", "磨砂玻璃", "内容从底栏后透出，胶囊高亮"),
            ("normal", "经典分栏", "系统栏底色，无浮层"),
            ("floating", "悬浮胶囊", "与屏幕边缘留出间距"),
        ],
        default: "glass",
    },
    OptionSpec {
        key: "compact",
        title: "紧凑歌曲栏",
        values: &[
            ("off", "播放时短暂展开", "切歌或暂停后自动展开数秒"),
            ("on", "始终保持紧凑", "只保留封面、标题与播放键"),
        ],
        default: "off",
    },
    OptionSpec {
        key: "blur",
        title: "界面模糊效果",
        values: &[
            ("on", "开启", "磨砂与弥散背景生效"),
            ("off", "关闭", "降低渲染负担，使用纯色"),
        ],
        default: "on",
    },
    OptionSpec {
        key: "navLabels",
        title: "底栏文字",
        values: &[("on", "显示", "图标与文字"), ("off", "仅图标", "更紧凑的底栏")],
        default: "on",
    },
    OptionSpec {
        key: "background",
        title: "播放页背景",
        values: &[
            ("artwork", "封面弥散", "当前封面取色弥散"),
            ("tonal", "主题色渐变", "使用应用主题色"),
            ("flow", "动态流光", "缓慢流动的色块"),
            ("plain", "纯色背景", "不使用背景效果"),
        ],
        default: "artwork",
    },
    OptionSpec {
        key: "shape",
        title: "封面形状",
        values: &[
            ("default", "微圆角", "8 像素圆角"),
            ("rounded", "圆角矩形", "28 像素圆角"),
            ("circle", "圆形", "唱片式圆形封面"),
        ],
        default: "rounded",
    },
    OptionSpec {
        key: "titleAlign",
        title: "播放页标题",
        values: &[("left", "左对齐", ""), ("center", "居中", "")],
        default: "left",
    },
    OptionSpec {
        key: "artwork",
        title: "歌曲列表封面",
        values: &[("on", "显示", ""), ("off", "隐藏", "列表更紧凑")],
        default: "on",
    },
    OptionSpec {
        key: "duration",
        title: "歌曲列表时长",
        values: &[("off", "隐藏", ""), ("on", "显示", "在行尾显示时长")],
        default: "off",
    },
    OptionSpec {
        key: "lyricAlign",
        title: "歌词对齐",
        values: &[("left", "左对齐", ""), ("center", "居中", "")],
        default: "left",
    },
    OptionSpec {
        key: "lyricSize",
        title: "歌词字号",
        values: &[
            ("24", "小", "24 像素"),
            ("28", "标准", "28 像素"),
            ("32", "大", "32 像素"),
            ("36", "特大", "36 像素"),
        ],
        default: "28",
    },
    OptionSpec {
        key: "lyricWeight",
        title: "歌词字重",
        values: &[
            ("500", "适中", "500"),
            ("600", "半粗", "600"),
            ("700", "粗体", "700"),
        ],
        default: "700",
    },
    OptionSpec {
        key: "lyricBlur",
        title: "歌词渐变模糊",
        values: &[
            ("on", "开启", "临近句逐渐清晰，突出当前句"),
            ("off", "关闭", "所有行保持清晰"),
        ],
        default: "on",
    },
    OptionSpec {
        key: "translation",
        title: "歌词翻译",
        values: &[
            ("on", "显示可用翻译", "只显示服务已返回的翻译"),
            ("off", "隐藏", ""),
        ],
        default: "on",
    },
    OptionSpec {
        key: "lyricScale",
        title: "当前句放大",
        values: &[("on", "开启", ""), ("off", "关闭", "")],
        default: "on",
    },
    OptionSpec {
        key: "motion",
        title: "页面动效",
        values: &[
            ("full", "舒缓弹性", "保留弹簧与滑动动画"),
            ("reduced", "减少动态效果", "减少位移与缩放"),
        ],
        default: "full",
    },
    OptionSpec {
        key: "coverFlip",
        title: "切歌封面动效",
        values: &[("on", "轻柔翻转", ""), ("off", "关闭", "")],
        default: "on",
    },
    OptionSpec {
        key: "lyricControls",
        title: "歌词播放控件",
        values: &[
            ("on", "显示", "歌词页保留播放控制"),
            ("off", "隐藏", "仅保留底部进度"),
        ],
        default: "on",
    },
];
pub fn spec(key: &str) -> Option<&'static OptionSpec> {
    OPTIONS.iter().find(|s| s.key == key)
}
pub fn get<'a>(v: &'a Value, key: &str) -> &'a str {
    let Some(spec) = spec(key) else {
        return "";
    };
    v["uiPreferences"][key]
        .as_str()
        .filter(|value| spec.values.iter().any(|(id, _, _)| id == value))
        .unwrap_or(spec.default)
}
/// Display title of the stored value, used as the trailing value of a settings row.
pub fn title(v: &Value, key: &str) -> &'static str {
    let current = get(v, key);
    spec(key)
        .and_then(|spec| {
            spec.values
                .iter()
                .find(|(id, _, _)| *id == current)
                .map(|(_, title, _)| *title)
        })
        .unwrap_or("")
}
pub fn set(v: &mut Value, key: &str, value: &str) -> bool {
    if !OPTIONS.iter().any(|s| {
        s.key == key && s.values.iter().any(|(id, _, _)| *id == value)
    }) {
        return false;
    }
    if !v["uiPreferences"].is_object() {
        v["uiPreferences"] = json!({});
    }
    v["uiPreferences"][key] = json!(value);
    true
}
/// One radio row per accepted value. Melox opens a dedicated page per setting;
/// rows keep the whole value list visible with the current one checked.
pub fn radios(v: &Value, key: &str) -> Vec<Value> {
    let Some(spec) = spec(key) else {
        return Vec::new();
    };
    let selected = get(v, key);
    spec.values
        .iter()
        .map(|(id, title, summary)| {
            json!({
                "kind": "radio",
                "text": {"value": title},
                "summary": if summary.is_empty() { Value::Null } else { json!({"value": summary}) },
                "selected": *id == selected,
                "action": format!("ui:{key}:{id}")
            })
        })
        .collect()
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
        assert_eq!(title(&v, "shape"), "圆形");
    }
    #[test]
    fn radios_mark_exactly_one_value() {
        let v = json!({"uiPreferences":{"bar":"floating"}});
        let rows = radios(&v, "bar");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows.iter().filter(|r| r["selected"] == true).count(), 1);
        assert!(rows[2]["action"] == "ui:bar:floating");
        assert!(radios(&v, "missing").is_empty());
    }
}