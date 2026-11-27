//! Offline, declarative presentation layer. Rust owns page composition, navigation,
//! content ordering and action descriptions; the Rust/WASM renderer builds the UI.
//! This module never receives credentials, requests a URL or mutates playback state.
use serde_json::{json, Value};

fn tr(key: &str) -> Value {
    json!({"resource":key})
}
fn arg(key: &str, args: Value) -> Value {
    json!({"resource":key,"args":args})
}
fn value(text: &str) -> Value {
    json!({"value":text})
}
fn string<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn count(v: &Value, key: &str) -> u64 {
    v["counts"][key].as_u64().unwrap_or(0)
}
fn yes(v: &Value, key: &str) -> bool {
    v[key].as_bool().unwrap_or(false)
}
fn text(label: Value, style: &str) -> Value {
    json!({"kind":"text","text":label,"style":style})
}
fn icon(name: &str) -> Value {
    json!({"kind":"icon","icon":name,"size":24,"tone":"primary"})
}
fn column(children: Vec<Value>) -> Value {
    json!({"kind":"column","gap":6,"weight":1,"children":children})
}
fn row(children: Vec<Value>) -> Value {
    json!({"kind":"row","gap":16,"children":children})
}
fn card(key: &str, children: Vec<Value>, action: &str) -> Value {
    json!({"key":key,"kind":"card","action":action,"inset":12,"padding":20,"children":children})
}
fn section(key: &str, label: &str, action: &str) -> Value {
    json!({"key":key,"kind":"section","text":tr(label),"action":action})
}
fn button(label: &str, action: &str) -> Value {
    json!({"kind":"button","text":tr(label),"action":action,"weight":1})
}
fn art(url: &str, size: u32, avatar: bool) -> Value {
    json!({"kind":"artwork","url":url,"size":size,"avatar":avatar})
}
fn songs(key: &str, source: &str, start: u32, limit: u32) -> Value {
    json!({"key":key,"kind":"songs","source":source,"start":start,"limit":limit})
}
fn shelf(key: &str, source: &str, playlists: bool) -> Value {
    json!({"key":key,"kind":"shelf","source":source,"playlists":playlists,"limit":if playlists{12}else{6}})
}
fn empty(key: &str, title: &str, summary: &str, action: &str, action_label: &str) -> Value {
    card(
        key,
        vec![
            icon("music"),
            text(tr(title), "heading"),
            text(tr(summary), "summary"),
            button(action_label, action),
        ],
        action,
    )
}
/// A settings row with a leading glyph and a trailing current value, following the
/// Melox grouped-list layout: icon, title/summary, value, chevron.
fn pref_full(label: &str, summary: Value, action: &str, glyph: &str, value: Value) -> Value {
    json!({"kind":"preference","text":tr(label),"summary":summary,"action":action,"glyph":glyph,"value":value})
}
/// A row that opens a sub-page; `title` and `summary` are plain UI copy.
fn nav_row(title: &str, summary: &str, action: &str, glyph: &str, value: Value) -> Value {
    json!({"kind":"preference","text":value_json(title),"summary":value_json(summary),"action":action,"glyph":glyph,"value":value})
}
fn value_json(text: &str) -> Value {
    json!({"value":text})
}
fn note(text: &str) -> Value {
    json!({"kind":"text","style":"settings-note","text":value_json(text)})
}
fn tile(label: &str, summary: Value, glyph: &str, action: &str) -> Value {
    let mut node = card(
        "",
        vec![
            icon(glyph),
            text(tr(label), "body"),
            text(summary, "caption"),
        ],
        action,
    );
    node["weight"] = json!(1);
    node["inset"] = json!(0);
    node["padding"] = json!(16);
    node
}
fn tiles(key: &str, children: Vec<Value>) -> Value {
    json!({"key":key,"kind":"row","gap":12,"inset":12,"bottom":12,"children":children})
}

fn home(v: &Value) -> Vec<Value> {
    let logged = yes(v, "loggedIn");
    let daily = card(
        "daily",
        vec![row(vec![
            column(vec![
                text(tr("made_for_you"), "eyebrow"),
                text(tr("daily_mix"), "hero"),
                text(
                    tr(if logged {
                        "daily_summary"
                    } else {
                        "daily_login_summary"
                    }),
                    "summary",
                ),
            ]),
            art(string(v, "dailyCover"), 104, false),
        ])],
        "daily",
    );
    let mut nodes = vec![
        section("recommended", "recommended_songs", "refresh_home"),
        daily,
    ];
    if count(v, "discovery") > 0 {
        nodes.push(shelf("recommendations", "discovery", false));
    } else {
        nodes.push(empty(
            "recommendations-empty",
            "recommendation_empty",
            "recommendation_empty_summary",
            if logged { "refresh_home" } else { "login" },
            if logged { "refresh" } else { "login_qq" },
        ));
    }
    nodes.push(section("playlists-title", "recommended_playlists", ""));
    if count(v, "recommendations") > 0 {
        nodes.push(shelf("playlists", "recommendations", true));
    } else {
        nodes.push(empty(
            "playlists-empty",
            "library_no_playlists",
            "recommendation_empty_summary",
            if logged { "refresh_home" } else { "login" },
            if logged { "refresh" } else { "login_qq" },
        ));
    }
    if count(v, "recent") > 0 {
        nodes.push(section("recent-title", "recent_plays", "navigate:recent"));
        nodes.push(songs("recent", "recent", 0, 3));
    }
    nodes.push(section("explore-title", "keep_discovering", ""));
    nodes.push(songs("explore", "discovery", 6, 0));
    nodes.push(json!({"key":"feed-status","kind":"feed_status"}));
    nodes
}

fn library(v: &Value) -> Vec<Value> {
    let logged = yes(v, "loggedIn");
    let name = string(v, "nickname");
    let mut profile = vec![row(vec![
        art(string(v, "avatar"), 60, true),
        column(vec![
            text(
                if logged && !name.is_empty() {
                    value(name)
                } else {
                    tr(if logged { "qq_user" } else { "music_with_you" })
                },
                "heading",
            ),
            text(
                if logged {
                    arg("account_identifier", json!([string(v, "uin")]))
                } else {
                    tr("login_library_summary")
                },
                "summary",
            ),
        ]),
    ])];
    if !logged {
        profile.push(button("login_qq", "login"));
    }
    let mut nodes = vec![
        card("account", profile, ""),
        section("collections-title", "collections", ""),
        tiles(
            "collections",
            vec![
                tile(
                    "liked_music",
                    if logged {
                        arg("song_count", json!([count(v, "likes")]))
                    } else {
                        tr("login_library_short")
                    },
                    "heart",
                    if logged { "open:likes" } else { "login" },
                ),
                tile(
                    "local_favorites",
                    arg("song_count", json!([count(v, "favorites")])),
                    "music",
                    "open:favorites",
                ),
            ],
        ),
        tiles(
            "tools",
            vec![
                tile(
                    "recent_plays",
                    arg("song_count", json!([count(v, "recent")])),
                    "clock",
                    "navigate:recent",
                ),
                tile("sleep_timer", tr("sleep_off_summary"), "timer", "sleep"),
            ],
        ),
        section("my-playlists", "my_playlists", "refresh_library"),
    ];
    if count(v, "playlists") > 0 {
        nodes.push(json!({"key":"playlist-filter","kind":"tabs","selection":string(v,"filter"),"items":[
            {"id":"all","text":tr("filter_all")},{"id":"created","text":tr("filter_created")},{"id":"saved","text":tr("filter_saved")}
        ]}));
        nodes.push(json!({"key":"library-playlists","kind":"playlists","source":"playlists","filter":string(v,"filter")}));
    } else {
        nodes.push(empty(
            "library-empty",
            "library_no_playlists",
            if logged {
                "library_refresh_hint"
            } else {
                "login_library_summary"
            },
            if logged { "refresh_library" } else { "login" },
            if logged { "refresh" } else { "login_qq" },
        ));
    }
    nodes
}

fn search(v: &Value) -> Vec<Value> {
    let mut nodes = vec![json!({"key":"search-field","kind":"search_input"})];
    let query = string(v, "query").trim();
    if !query.is_empty() && query == string(v, "searchedQuery") {
        nodes.push(section("results-title", "search_results", ""));
        if count(v, "search") > 0 {
            nodes.push(songs("results", "search", 0, 0));
        } else if !yes(v, "busy") {
            nodes.push(empty(
                "results-empty",
                "no_search_results",
                "try_search_again",
                "focus_search",
                "search",
            ));
        }
    } else {
        if count(v, "history") > 0 {
            nodes.push(section("history-title", "search_history", "clear_search"));
            nodes.push(json!({"key":"history","kind":"chips","source":"history"}));
        }
        if count(v, "artists") > 0 {
            nodes.push(section("artists-title", "discover_artists", ""));
            nodes.push(json!({"key":"artists","kind":"chips","source":"artists"}));
        }
        nodes.push(section(
            "search-discover-title",
            "search_recommendations",
            "refresh_home",
        ));
        if count(v, "discovery") > 0 {
            nodes.push(shelf("search-discover", "discovery", false));
            nodes.push(songs("search-more", "discovery", 6, 6));
        } else {
            nodes.push(empty(
                "search-empty",
                "search_empty",
                "search_empty_summary",
                "",
                "explore",
            ));
        }
    }
    nodes
}

fn detail(v: &Value) -> Vec<Value> {
    let loading = yes(v, "detailLoading");
    let error = string(v, "detailError");
    let mut head = vec![row(vec![
        art(string(v, "detailCover"), 100, false),
        column(vec![
            text(value(string(v, "detailTitle")), "heading"),
            text(
                if loading {
                    tr("loading_playlist")
                } else if !error.is_empty() {
                    tr("playlist_load_failed")
                } else {
                    arg("song_count", json!([count(v, "tracks")]))
                },
                "summary",
            ),
        ]),
    ])];
    if loading {
        head.push(json!({"kind":"loading","text":tr("playlist_loading_hint")}));
    }
    if !error.is_empty() {
        head.push(text(value(error), "summary"));
        head.push(button("retry", "retry_playlist"));
    }
    let mut controls = row(vec![
        button("play_all", "play_tracks"),
        button("shuffle_play", "shuffle_tracks"),
    ]);
    controls["enabled"] = json!(!loading && count(v, "tracks") > 0);
    head.push(controls);
    let mut nodes = vec![card("playlist-overview", head, "")];
    if count(v, "tracks") == 0 && !loading && error.is_empty() {
        nodes.push(empty(
            "songs-empty",
            "no_songs",
            "retry_later",
            "",
            "explore",
        ));
    }
    nodes.push(songs("tracks", "tracks", 0, 0));
    nodes
}

fn recent(v: &Value) -> Vec<Value> {
    let mut clear = button("clear_recent", "clear_recent");
    clear["enabled"] = json!(count(v, "recent") > 0);
    let mut nodes = vec![card(
        "recent-overview",
        vec![
            text(arg("recent_count", json!([count(v, "recent")])), "heading"),
            text(tr("recent_privacy"), "summary"),
            clear,
        ],
        "",
    )];
    if count(v, "recent") == 0 {
        nodes.push(empty(
            "recent-empty",
            "recent_empty",
            "recent_empty_summary",
            "navigate:home",
            "home",
        ));
    }
    nodes.push(songs("recent-tracks", "recent", 0, 0));
    nodes
}

fn appearance(mode: &str) -> &'static str {
    match mode {
        "light" => "light_mode",
        "dark" => "dark_mode",
        _ => "system_mode",
    }
}
fn source(mode: &str) -> &'static str {
    match mode {
        "artwork" => "source_artwork",
        "custom" => "source_custom",
        _ => "source_wallpaper",
    }
}
fn palette(mode: &str) -> &'static str {
    match mode {
        "Neutral" => "palette_neutral",
        "Vibrant" => "palette_vibrant",
        "Expressive" => "palette_expressive",
        "Rainbow" => "palette_rainbow",
        "FruitSalad" => "palette_fruit",
        "Monochrome" => "palette_mono",
        "Fidelity" => "palette_fidelity",
        "Content" => "palette_content",
        "Cmf" => "palette_cmf",
        _ => "palette_tonal",
    }
}
fn group(key: &str, children: Vec<Value>) -> Value {
    let mut node = card(key, children, "");
    node["padding"] = json!(0);
    node
}
fn radio(label: Value, summary: Value, selected: bool, enabled: bool, action: &str) -> Value {
    json!({"kind":"radio","text":label,"summary":summary,"selected":selected,"enabled":enabled,"action":action})
}
fn monet_switch(v: &Value) -> Value {
    json!({"kind":"switch","text":tr("monet_title"),"summary":tr("monet_summary"),"glyph":"palette","checked":yes(v,"monet"),"action":"toggle_monet"})
}
fn dependent(mut node: Value, enabled: bool) -> Value {
    node["enabled"] = json!(enabled);
    node
}

fn option_card(page: &str, key: &str, v: &Value) -> Vec<Value> {
    let title = crate::customization::spec(key)
        .map(|s| s.title)
        .unwrap_or(key);
    vec![
        json!({"kind":"section","text":value_json(title)}),
        group(
            &format!("{page}-{key}"),
            crate::customization::radios(v, key),
        ),
    ]
}
fn options_page(page: &str, keys: &[&str], v: &Value) -> Vec<Value> {
    let mut nodes = Vec::new();
    for key in keys {
        nodes.extend(option_card(page, key, v));
    }
    nodes.push(note(
        "改动即时生效并自动保存。系统关闭动画时优先遵循系统设置。",
    ));
    nodes
}
/// Hub page: one group of rows that opens a dedicated preference page, mirroring
/// the Melox theme/lyrics/interface sub-page structure instead of one long list.
fn customization_hub(
    page: &str,
    title: &str,
    summary: &str,
    rows: Vec<Value>,
    v: &Value,
) -> Vec<Value> {
    let mut nodes = vec![
        json!({"kind":"section","text":value_json(title)}),
        group(&format!("{page}-rows"), rows),
    ];
    if let Some(current) = crate::customization::spec("bar") {
        // The hub keeps a compact preview of the two most visible choices.
        nodes.push(
            json!({"kind":"text","style":"settings-note","text":value_json(&format!(
                "底栏：{} · 封面：{}",
                crate::customization::title(v, "bar"),
                crate::customization::title(v, "shape")
            ))}),
        );
        let _ = current;
    }
    nodes.push(note(summary));
    nodes
}
fn customization_page(page: &str, v: &Value) -> Vec<Value> {
    match page {
        "lyrics-style" => customization_hub(
            page,
            "歌词外观",
            "歌词排版与显示效果，只影响已有歌词数据，不请求新的翻译或逐词时间。",
            vec![
                nav_row(
                    "歌词排版",
                    "对齐、字号、字重",
                    "navigate:lyric-type",
                    "text",
                    Value::Null,
                ),
                nav_row(
                    "歌词效果",
                    "渐变模糊、翻译、当前句放大、播放控件",
                    "navigate:lyric-effect",
                    "motion",
                    Value::Null,
                ),
            ],
            v,
        ),
        "ui-customize" => customization_hub(
            page,
            "主题与外观",
            "选择适合自己的配色、布局与显示效果。",
            vec![
                nav_row(
                    "主题颜色",
                    "明暗模式与莫奈取色",
                    "navigate:appearance",
                    "palette",
                    tr(appearance(string(v, "colorMode"))),
                ),
                nav_row(
                    "底栏与材质",
                    "底栏样式、紧凑歌曲栏、模糊与文字",
                    "navigate:ui-dock",
                    "dock",
                    Value::Null,
                ),
                nav_row(
                    "播放界面",
                    "背景、封面形状、标题对齐、切歌动效",
                    "navigate:ui-player",
                    "cover",
                    Value::Null,
                ),
                nav_row(
                    "歌曲列表",
                    "列表封面与时长",
                    "navigate:ui-list",
                    "list",
                    Value::Null,
                ),
                nav_row(
                    "页面动效",
                    "舒缓弹性或减少动态效果",
                    "navigate:ui-motion",
                    "motion",
                    Value::Null,
                ),
            ],
            v,
        ),
        "ui-dock" => options_page(page, &["bar", "compact", "blur", "navLabels"], v),
        "ui-player" => options_page(page, &["background", "shape", "titleAlign", "coverFlip"], v),
        "ui-list" => options_page(page, &["artwork", "duration"], v),
        "ui-motion" => options_page(page, &["motion"], v),
        "lyric-type" => options_page(page, &["lyricAlign", "lyricSize", "lyricWeight"], v),
        "lyric-effect" => options_page(
            page,
            &["lyricBlur", "translation", "lyricScale", "lyricControls"],
            v,
        ),
        _ => options_page(page, &[], v),
    }
}
fn quality_title(v: &Value) -> Value {
    tr(match string(v, "quality") {
        "sq" => "quality_sq",
        "hq" => "quality_hq",
        "standard" => "quality_standard",
        _ => "quality_auto",
    })
}
fn settings(page: &str, v: &Value) -> Vec<Value> {
    let monet = yes(v, "monet");
    match page {
        "ui-customize" | "lyrics-style" | "ui-dock" | "ui-player" | "ui-list" | "ui-motion"
        | "lyric-type" | "lyric-effect" => customization_page(page, v),
        "settings" => {
            let mut nodes = vec![
                section("personalization-title", "personalization_group", ""),
                group(
                    "personalization",
                    vec![
                        nav_row(
                            "主题与外观",
                            "配色、底栏、播放页与动效",
                            "navigate:ui-customize",
                            "dock",
                            Value::Null,
                        ),
                        nav_row(
                            "歌词外观",
                            "排版、逐字动效与翻译",
                            "navigate:lyrics-style",
                            "lyrics",
                            Value::Null,
                        ),
                    ],
                ),
                section("player-title", "playback_group", ""),
                group(
                    "player",
                    vec![
                        pref_full(
                            "audio_quality",
                            Value::Null,
                            "navigate:quality",
                            "quality",
                            quality_title(v),
                        ),
                        pref_full(
                            "audio_effects",
                            Value::Null,
                            "audio-effects",
                            "equalizer",
                            Value::Null,
                        ),
                        pref_full("sleep_timer", Value::Null, "sleep", "timer", Value::Null),
                    ],
                ),
                section("app-title", "application_group", ""),
                group(
                    "app",
                    vec![
                        pref_full(
                            "image_cache",
                            tr("cache_short"),
                            "navigate:storage",
                            "storage",
                            Value::Null,
                        ),
                        pref_full(
                            "about_auralis",
                            Value::Null,
                            "navigate:about",
                            "info",
                            value(string(v, "version")),
                        ),
                    ],
                ),
            ];
            if yes(v, "loggedIn") {
                nodes.push(section("account-title", "account_group", ""));
                nodes.push(group(
                    "account",
                    vec![pref_full(
                        "logout",
                        Value::Null,
                        "logout",
                        "logout",
                        Value::Null,
                    )],
                ));
            }
            nodes
        }
        "appearance" => vec![
            section("title", "appearance_group", ""),
            group(
                "modes",
                ["system", "light", "dark"]
                    .iter()
                    .map(|mode| {
                        radio(
                            tr(appearance(mode)),
                            tr(match *mode {
                                "light" => "light_mode_summary",
                                "dark" => "dark_mode_summary",
                                _ => "system_mode_summary",
                            }),
                            string(v, "colorMode") == *mode,
                            true,
                            &format!("appearance:{mode}"),
                        )
                    })
                    .collect(),
            ),
            section("colors-title", "monet_title", ""),
            group(
                "appearance-colors",
                vec![
                    monet_switch(v),
                    dependent(
                        pref_full(
                            "monet_options",
                            tr("monet_summary"),
                            "navigate:monet",
                            "palette",
                            tr(source(string(v, "monetSource"))),
                        ),
                        monet,
                    ),
                ],
            ),
        ],
        "quality" => vec![
            section("title", "audio_quality", ""),
            group(
                "qualities",
                vec![json!({"kind":"platform","slot":"quality"})],
            ),
        ],
        "monet" => {
            let mut nodes = vec![
                section("title", "monet_title", ""),
                group(
                    "options",
                    vec![
                        monet_switch(v),
                        dependent(
                            pref_full(
                                "color_source",
                                tr("color_source_summary"),
                                "navigate:monet-source",
                                "palette",
                                tr(source(string(v, "monetSource"))),
                            ),
                            monet,
                        ),
                        dependent(
                            pref_full(
                                "palette_style",
                                tr("palette_style_summary"),
                                "navigate:monet-style",
                                "palette",
                                tr(palette(string(v, "monetStyle"))),
                            ),
                            monet,
                        ),
                        dependent(
                            pref_full(
                                "color_spec",
                                tr("color_spec_summary"),
                                "navigate:monet-spec",
                                "info",
                                value(if string(v, "monetSpec") == "Spec2025" {
                                    "2025"
                                } else {
                                    "2021"
                                }),
                            ),
                            monet,
                        ),
                    ],
                ),
                section("preview-title", "color_preview", ""),
                group(
                    "preview",
                    vec![json!({"kind":"platform","slot":"color_preview"})],
                ),
            ];
            if string(v, "monetSource") == "custom" {
                nodes.push(json!({"key":"custom-color","kind":"platform","slot":"custom_color"}));
            }
            nodes
        }
        "monet-source" => {
            let mut nodes = vec![
                section("title", "color_source", ""),
                group(
                    "sources",
                    ["wallpaper", "artwork", "custom"]
                        .iter()
                        .map(|item| {
                            radio(
                                tr(source(item)),
                                tr(match *item {
                                    "artwork" => "artwork_summary",
                                    "custom" => "custom_summary",
                                    _ => "wallpaper_summary",
                                }),
                                string(v, "monetSource") == *item,
                                monet,
                                &format!("monet-source:{item}"),
                            )
                        })
                        .collect(),
                ),
            ];
            if string(v, "monetSource") == "custom" {
                nodes.push(json!({"key":"custom-color","kind":"platform","slot":"custom_color"}));
            }
            nodes
        }
        "monet-style" => vec![
            section("title", "palette_style", ""),
            group(
                "palettes",
                [
                    "TonalSpot",
                    "Neutral",
                    "Vibrant",
                    "Expressive",
                    "Rainbow",
                    "FruitSalad",
                    "Monochrome",
                    "Fidelity",
                    "Content",
                    "Cmf",
                ]
                .iter()
                .map(|item| {
                    radio(
                        tr(palette(item)),
                        Value::Null,
                        string(v, "monetStyle") == *item,
                        monet,
                        &format!("monet-style:{item}"),
                    )
                })
                .collect(),
            ),
        ],
        "monet-spec" => vec![
            section("title", "color_spec", ""),
            group(
                "specs",
                ["Spec2021", "Spec2025"]
                    .iter()
                    .map(|item| {
                        radio(
                            value(&item[4..]),
                            tr(if *item == "Spec2025" {
                                "spec_2025_summary"
                            } else {
                                "spec_2021_summary"
                            }),
                            string(v, "monetSpec") == *item,
                            monet,
                            &format!("monet-spec:{item}"),
                        )
                    })
                    .collect(),
            ),
        ],
        "storage" => vec![
            section("title", "image_cache", ""),
            group("cache", vec![json!({"kind":"platform","slot":"cache"})]),
        ],
        "about" => vec![
            json!({"kind":"platform","slot":"about_identity"}),
            group(
                "about-project",
                vec![
                    pref_full(
                        "project_home",
                        Value::Null,
                        "link:project",
                        "info",
                        Value::Null,
                    ),
                    pref_full(
                        "version_history",
                        Value::Null,
                        "link:releases",
                        "refresh",
                        Value::Null,
                    ),
                    pref_full("feedback", Value::Null, "link:issues", "text", Value::Null),
                ],
            ),
            group(
                "about-license",
                vec![
                    pref_full(
                        "open_source_licenses",
                        Value::Null,
                        "navigate:licenses",
                        "library",
                        Value::Null,
                    ),
                    pref_full(
                        "copy_project_address",
                        Value::Null,
                        "copy-project",
                        "share",
                        Value::Null,
                    ),
                ],
            ),
            note(&crate::labels::label("about_footer")),
        ],
        "licenses" => vec![json!({"kind":"platform","slot":"licenses"})],
        _ => vec![],
    }
}
pub fn screen(v: &Value) -> Result<Value, String> {
    let page = string(v, "page");
    let (title, nodes) = match page {
        "home" => (tr("home"), home(v)),
        "mine" => (tr("music_library"), library(v)),
        "search" => (tr("search"), search(v)),
        "songs" => (value(string(v, "detailTitle")), detail(v)),
        "recent" => (tr("recent_plays"), recent(v)),
        "settings" | "appearance" | "quality" | "monet" | "monet-source" | "monet-style"
        | "monet-spec" | "storage" | "about" | "licenses" | "ui-customize" | "lyrics-style"
        | "ui-dock" | "ui-player" | "ui-list" | "ui-motion" | "lyric-type" | "lyric-effect" => {
            let title = match page {
                "monet" => "monet_title",
                "monet-source" => "color_source",
                "monet-style" => "palette_style",
                "monet-spec" => "color_spec",
                "quality" => "audio_quality",
                "ui-customize" => "ui_customize",
                "lyrics-style" => "lyrics_style",
                "ui-dock" => "dock_group",
                "ui-player" => "player_group",
                "ui-list" => "list_group",
                "ui-motion" => "motion_group",
                "lyric-type" => "lyric_type_group",
                "lyric-effect" => "lyric_effect_group",
                _ => page,
            };
            (tr(title), settings(page, v))
        }
        _ => return Err("Unknown presentation route".into()),
    };
    Ok(
        json!({"schema":1,"route":page,"root":matches!(page,"home"|"mine"|"settings"),"title":title,"nodes":nodes,
            "navigation":[{"route":"home","text":tr("home"),"icon":"home"},{"route":"mine","text":tr("music_library"),"icon":"library"},{"route":"settings","text":tr("settings"),"icon":"settings"}],
        "player":{"pages":[{"id":"information","text":tr("song_information")},{"id":"cover","text":tr("cover_page")},{"id":"lyrics","text":tr("lyrics_page")}],
            "initialPage":1,"transport":["previous","play","next"],"tools":["mode","quality","timer","queue","actions"]},
        "tokens":{"gutter":12,"sectionInset":28,"cardRadius":24,"artworkRadius":14,"shelfWidth":156,
                "miniRadius":32,"miniHeight":72,"expandedHeight":192,"miniCover":44,"expandedCover":56,
                "playerRadius":18,"coverInset":32,"lyricFocus":0.38,"motionDamping":0.86,"motionStiffness":290,
                "transportSize":72,"navigationRadius":36}
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn playlist_failure_keeps_retry_and_disables_transport() {
        let doc =
            screen(&json!({"page":"songs","detailError":"offline","counts":{"tracks":0}})).unwrap();
        let head = &doc["nodes"][0]["children"];
        assert!(head.to_string().contains("retry_playlist"));
        assert_eq!(head.as_array().unwrap().last().unwrap()["enabled"], false);
    }
    #[test]
    fn home_has_one_daily_entry_and_keeps_guess_playlist() {
        let doc =
            screen(&json!({"page":"home","counts":{"discovery":20,"recommendations":3}})).unwrap();
        assert_eq!(doc.to_string().matches("\"action\":\"daily\"").count(), 1);
        assert!(doc["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["source"] == "recommendations"));
    }
    #[test]
    fn settings_dependencies_and_logout_follow_account() {
        let guest = screen(&json!({"page":"settings"})).unwrap().to_string();
        assert!(!guest.contains("\"action\":\"logout\""));
        let doc = screen(&json!({"page":"settings","loggedIn":true,"monet":false})).unwrap();
        let appearance = screen(&json!({"page":"appearance","monet":false})).unwrap();
        assert_eq!(appearance["nodes"][3]["children"][1]["enabled"], false);
        assert!(doc.to_string().contains("\"action\":\"logout\""));
    }
    #[test]
    fn settings_have_one_about_entry_and_license_destination() {
        let settings = screen(&json!({"page":"settings","version":"0.5.6"}))
            .unwrap()
            .to_string();
        assert_eq!(settings.matches("navigate:about").count(), 1);
        assert!(!settings.contains("settings-intro"));
        let about = screen(&json!({"page":"about"})).unwrap().to_string();
        assert_eq!(about.matches("about_identity").count(), 1);
        assert!(about.contains("navigate:licenses"));
        assert!(screen(&json!({"page":"licenses"})).is_ok());
    }
    #[test]
    fn clearing_query_returns_discovery_without_stale_search_results() {
        let doc=screen(&json!({"page":"search","query":"","searchedQuery":"old","counts":{"search":20,"discovery":12}})).unwrap();
        assert!(!doc["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["source"] == "search"));
    }
    #[test]
    fn unknown_route_is_not_silently_presented_as_settings() {
        assert!(screen(&json!({"page":"missing"})).is_err());
    }
    #[test]
    fn interface_settings_are_split_into_navigable_sub_pages() {
        let hub = screen(&json!({"page":"ui-customize"})).unwrap().to_string();
        for action in [
            "navigate:ui-dock",
            "navigate:ui-player",
            "navigate:ui-list",
            "navigate:ui-motion",
        ] {
            assert!(
                hub.contains(action),
                "{action} missing from the interface hub"
            );
        }
        let lyrics = screen(&json!({"page":"lyrics-style"})).unwrap().to_string();
        assert!(lyrics.contains("navigate:lyric-type"));
        assert!(lyrics.contains("navigate:lyric-effect"));
        let dock = screen(&json!({"page":"ui-dock","uiPreferences":{"bar":"floating"}})).unwrap();
        let text = dock.to_string();
        assert_eq!(text.matches("\"action\":\"ui:bar:").count(), 2);
        assert!(text.contains("ui:bar:floating"));
        // One selected value per option group on this page.
        assert_eq!(text.matches("\"selected\":true").count(), 4);
        let player = screen(&json!({"page":"ui-player","uiPreferences":{"shape":"circle"}}));
        assert!(player.unwrap().to_string().contains("ui:shape:circle"));
    }
}
