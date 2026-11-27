//! Inline SVG glyphs for the Rust renderer. Navigation keeps one glyph and uses
//! the Miuix selected/pressed color-alpha contract instead of cross-fading fills.
fn stroke(name: &str) -> Option<(&'static str, bool)> {
    let path = match name {
        "home" => "M3 10.5 12 3l9 7.5M5.5 9v12h5v-7h3v7h5V9",
        "home-filled" => "M3 10.2 12 3l9 7.2V21h-6v-6h-6v6H3Z",
        "library" => "M4 4v16M8 4v16M12 4l5 16M20 4v16",
        "library-filled" => "M3 4h3v16H3ZM7.5 4h3v16h-3ZM12.7 4.3l3-.8 4.2 15.4-3 .8Z",
        "settings" => "m12 3 2 3 3 .5 1 3 2 2-2 2-1 3-3 .5-2 3-2-3-3-.5-1-3-2-2 2-2 1-3L10 6Z M15 12a3 3 0 1 0-6 0 3 3 0 0 0 6 0",
        "settings-filled" => "M10.6 2h2.8l.5 2.3 1.6.7 2-1.2 2 2-1.2 2 .7 1.6 2.3.5v2.8l-2.3.5-.7 1.6 1.2 2-2 2-2-1.2-1.6.7-.5 2.3h-2.8l-.5-2.3-1.6-.7-2 1.2-2-2 1.2-2-.7-1.6L2 13.4v-2.8l2.3-.5.7-1.6-1.2-2 2-2 2 1.2 1.6-.7Z M15.2 12a3.2 3.2 0 1 0-6.4 0 3.2 3.2 0 0 0 6.4 0Z",
        "search" => "M16.5 16.5 22 22M19 10.5a8.5 8.5 0 1 0-17 0 8.5 8.5 0 0 0 17 0",
        "heart" | "favorite" => "M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z",
        "music" => "M9 17V5l11-2v12M9 17a3 3 0 1 1-3-3c1.7 0 3 1.3 3 3M20 15a3 3 0 1 1-3-3c1.7 0 3 1.3 3 3",
        "account" => "M16 7a4 4 0 1 0-8 0 4 4 0 0 0 8 0M4 22v-2a8 8 0 0 1 16 0v2",
        "clock" | "timer" => "M21 12a9 9 0 1 0-18 0 9 9 0 0 0 18 0M12 7v6l4 2",
        "pause" => "M8 5v14M16 5v14",
        "play" => "m8 4 12 8-12 8Z",
        "previous" => "M5 5v14M19 5 7 12l12 7Z",
        "next" => "M19 5v14M5 5l12 7-12 7Z",
        "queue" => "M3 5h18M3 11h18M3 17h9m4-2 5 3-5 3Z",
        "back" => "m15 4-8 8 8 8",
        "chevron" => "m9 5 7 7-7 7",
        "chevron-down" => "m5 9 7 7 7-7",
        "down" => "m5 8 7 7 7-7",
        "up" => "m5 16 7-7 7 7",
        "refresh" => "M21 8a9 9 0 1 0 0 8M21 3v6h-6",
        "close" => "m5 5 14 14M19 5 5 19",
        "more" => "M5 11v2M12 11v2M19 11v2",
        "more-vertical" => "M11 4h2M11 12h2M11 20h2",
        "equalizer" => "M3 9v6M7.5 5v14M12 2v20M16.5 5v14M21 9v6",
        "plus" => "M12 4v16M4 12h16",
        "add-queue" => "M3 5h14M3 11h10M3 17h8M18 13v8M14 17h8",
        "share" => "M12 16V2m-5 5 5-5 5 5M5 11H3v10h18V11h-2",
        "order" => "M3 5h18M3 11h18M3 17h12m3-2 4 3-4 3",
        "shuffle" => "M3 5h3c5 0 7 14 12 14h3m-4-4 4 4-4 3M3 19h3c1.5 0 3-2 4-4m4-6c1-2 2.5-4 4-4h3m-4-3 4 3-4 4",
        "repeat" | "repeat_one" => "M4 9V7a2 2 0 0 1 2-2h14m-3-3 3 3-3 3M20 15v2a2 2 0 0 1-2 2H4m3-3-3 3 3 3",
        "locate" => "M12 2v4M12 18v4M2 12h4M18 12h4M18 12a6 6 0 1 0-12 0 6 6 0 0 0 12 0",
        "check" => "m4 12 5 5L21 5",
        "trash" => "M3 6h18M8 6V3h8v3M6 6l1 15h10l1-15M10 10v7M14 10v7",
        "quality" => "M4 5h16v14H4ZM8 9h8M8 13h5",
        // Appearance glyphs.
        "palette" => "M12 3a9 9 0 0 0 0 18h1.6a2.1 2.1 0 0 0 0-4.2h-1a1.6 1.6 0 0 1 0-3.2H16a5 5 0 0 0 5-5C21 5.5 17 3 12 3ZM8 9h.01M7 13h.01M11 7h.01M15 7.5h.01",
        "dock" => "M3 17.5h18M5 14.5h4.5v-3H5ZM14.5 14.5H19v-3h-4.5ZM3 20.5h18",
        "cover" => "M4 4h16v16H4ZM4 14l4.5-4.5L14 15M14 9.5a1.5 1.5 0 1 1 3 0 1.5 1.5 0 0 1-3 0",
        "text" => "M5 5h14M12 5v14M9 19h6",
        "list" => "M4 6h16M4 12h16M4 18h10",
        "motion" => "M3 16c3 0 3.5-8 6.5-8S13 16 16 16s3-5 5-5M3 20h18",
        "lyrics" => "M4 5h9M4 10h13M4 15h7M4 20h11M17 14l3 2-3 2Z",
        "translation" => "M4 5h9M8.5 5v2c0 4-2 7-4.5 8M6 10c1.5 2.5 3 4 5.5 5M13 19l3.5-9 3.5 9M14.4 16h4.2",
        "speed" => "M4 18a8 8 0 1 1 16 0M12 18l4-5",
        "cache" => "M5 6h14v5H5ZM5 13h14v5H5ZM9 8.5h.01M9 15.5h.01",
        "info" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM12 11v6M12 7.6h.01",
        "logout" => "M15 4h4v16h-4M11 8l-4 4 4 4M7 12h10",
        "storage" => "M4 6c0-1.7 3.6-3 8-3s8 1.3 8 3-3.6 3-8 3-8-1.3-8-3ZM4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3",
        "volume" => "M4 10v4h3l4 4V6L7 10ZM16 9a4 4 0 0 1 0 6M18.5 6.5a7 7 0 0 1 0 11",
        _ => return None,
    };
    Some((path, matches!(name, "play" | "previous" | "next")))
}

pub fn icon(name: &str) -> String {
    let (path, filled) = stroke(name).unwrap_or(("M12 4v16M4 12h16", false));
    format!(
        "<svg viewBox=\"0 0 24 24\" aria-hidden=\"true\" fill=\"{}\" stroke=\"currentColor\" stroke-width=\"1.8\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"{path}\"/></svg>",
        if filled { "currentColor" } else { "none" }
    )
}
