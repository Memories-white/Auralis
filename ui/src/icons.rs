pub fn icon(name: &str) -> String {
    let path=match name{
        "home"=>"M3 10.5 12 3l9 7.5M5.5 9v12h5v-7h3v7h5V9",
        "library"=>"M4 4v16M8 4v16M12 4l5 16M20 4v16",
        "settings"=>"m12 3 2 3 3 .5 1 3 2 2-2 2-1 3-3 .5-2 3-2-3-3-.5-1-3-2-2 2-2 1-3L10 6Z M15 12a3 3 0 1 0-6 0 3 3 0 0 0 6 0",
        "search"=>"M16.5 16.5 22 22M19 10.5a8.5 8.5 0 1 0-17 0 8.5 8.5 0 0 0 17 0",
        "heart"|"favorite"=>"M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z",
        "music"=>"M9 17V5l11-2v12M9 17a3 3 0 1 1-3-3c1.7 0 3 1.3 3 3M20 15a3 3 0 1 1-3-3c1.7 0 3 1.3 3 3",
        "account"=>"M16 7a4 4 0 1 0-8 0 4 4 0 0 0 8 0M4 22v-2a8 8 0 0 1 16 0v2",
        "clock"|"timer"=>"M21 12a9 9 0 1 0-18 0 9 9 0 0 0 18 0M12 7v6l4 2",
        "pause"=>"M8 5v14M16 5v14",
        "play"=>"m8 4 12 8-12 8Z",
        "previous"=>"M5 5v14M19 5 7 12l12 7Z",
        "next"=>"M19 5v14M5 5l12 7-12 7Z",
        "queue"=>"M3 5h18M3 11h18M3 17h9m4-2 5 3-5 3Z",
        "back"=>"m15 4-8 8 8 8","chevron"=>"m9 5 7 7-7 7","down"=>"m5 8 7 7 7-7","up"=>"m5 16 7-7 7 7",
        "refresh"=>"M21 8a9 9 0 1 0 0 8M21 3v6h-6",
        "close"=>"m5 5 14 14M19 5 5 19","more"=>"M5 11v2M12 11v2M19 11v2",
        "order"=>"M3 5h18M3 11h18M3 17h12m3-2 4 3-4 3",
        "shuffle"=>"M3 5h3c5 0 7 14 12 14h3m-4-4 4 4-4 3M3 19h3c1.5 0 3-2 4-4m4-6c1-2 2.5-4 4-4h3m-4-3 4 3-4 4",
        "repeat"|"repeat_one"=>"M4 9V7a2 2 0 0 1 2-2h14m-3-3 3 3-3 3M20 15v2a2 2 0 0 1-2 2H4m3-3-3 3 3 3",
        "locate"=>"M12 2v4M12 18v4M2 12h4M18 12h4M18 12a6 6 0 1 0-12 0 6 6 0 0 0 12 0",
        "check"=>"m4 12 5 5L21 5","trash"=>"M3 6h18M8 6V3h8v3M6 6l1 15h10l1-15M10 10v7M14 10v7",
        "quality"=>"M4 5h16v14H4ZM8 9h8M8 13h5",
        _=>"M12 4v16M4 12h16",
    };
    let fill = matches!(name, "play" | "previous" | "next");
    format!("<svg viewBox=\"0 0 24 24\" aria-hidden=\"true\" fill=\"{}\" stroke=\"currentColor\" stroke-width=\"1.8\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"{path}\"/></svg>",if fill{"currentColor"}else{"none"})
}
