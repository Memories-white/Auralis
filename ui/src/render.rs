use crate::{
    b, format_time,
    icons::{icon, icon_pair},
    labels::{esc, label, text},
    n, s,
};
use serde_json::{json, Value};

fn action(action: &str, label_text: &str, glyph: &str, class: &str) -> String {
    format!(
        "<button class=\"icon-button {class}\" data-action=\"{}\" aria-label=\"{}\">{}</button>",
        esc(action),
        esc(label_text),
        icon(glyph)
    )
}
fn button(action: &str, key: &str, primary: bool) -> String {
    format!(
        "<button class=\"button {}\" data-action=\"{}\">{}</button>",
        if primary { "primary" } else { "" },
        esc(action),
        esc(&label(key))
    )
}
fn art(url: &str, avatar: bool, class: &str) -> String {
    let placeholder = icon(if avatar { "account" } else { "music" });
    let image = if url.starts_with("https://") || url.starts_with("http://") {
        format!(
            "<img data-art-src=\"{}\" alt=\"\" decoding=\"sync\" />",
            esc(&crate::artwork::source(url, avatar))
        )
    } else {
        String::new()
    };
    format!(
        "<span class=\"artwork {} {class}\">{placeholder}{image}</span>",
        if avatar { "avatar" } else { "" }
    )
}
fn cover(song: &Value) -> String {
    let direct = s(song, "coverUrl");
    if !direct.is_empty() {
        return direct.replace("http://", "https://");
    }
    let album = s(song, "albumMid");
    if !album.is_empty() {
        return format!("https://y.gtimg.cn/music/photo_new/T002R500x500M000{album}.jpg");
    }
    s(song, "cover").replace("http://", "https://")
}
fn source<'a>(data: &'a Value, id: &str) -> &'a [Value] {
    data[id].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn escaped_or(value: &str, key: &str) -> String {
    if value.is_empty() {
        esc(&label(key))
    } else {
        esc(value)
    }
}
fn attr(action: &str, enabled: bool) -> String {
    if action.is_empty() {
        String::new()
    } else {
        format!(
            " data-action=\"{}\" {}",
            esc(action),
            if enabled {
                "role=\"button\" tabindex=\"0\""
            } else {
                "aria-disabled=\"true\" disabled"
            }
        )
    }
}
fn glyph(name: &str) -> String {
    if name.is_empty() {
        String::new()
    } else {
        format!("<span class=\"pref-glyph\">{}</span>", icon(name))
    }
}
fn trailing_value(value: &Value) -> String {
    if value.is_null() {
        return String::new();
    }
    let text = if value.is_object() {
        text(value)
    } else {
        value.as_str().unwrap_or("").to_owned()
    };
    if text.is_empty() {
        String::new()
    } else {
        format!("<span class=\"pref-value\">{}</span>", esc(&text))
    }
}

pub fn header(p: &Value, data: &Value) -> String {
    let root = b(p, "root");
    let back = if root {
        String::new()
    } else {
        action("back", &label("back"), "back", "")
    };
    let tools = if matches!(s(data, "page"), "home" | "mine") {
        let profile = if b(data, "loggedIn") {
            format!("<button class=\"icon-button profile-action\" data-action=\"navigate:mine\" aria-label=\"{}\">{}</button>",esc(&label("my_account")),art(s(data,"avatar"),true,""))
        } else {
            action("login", &label("login"), "account", "")
        };
        format!(
            "{}{profile}",
            action("navigate:search", &label("search"), "search", "")
        )
    } else {
        String::new()
    };
    format!(
        "<div class=\"header-tools\">{back}<span></span>{tools}</div><h1>{}</h1>",
        esc(&text(&p["title"]))
    )
}
pub fn navigation(p: &Value) -> String {
    if !b(p, "root") {
        return String::new();
    }
    let items = p["navigation"].as_array().cloned().unwrap_or_default();
    let count = items.len().max(1);
    let selected = items
        .iter()
        .position(|item| item["route"] == p["route"])
        .unwrap_or(0);
    // The indicator is a sibling of the items, so Rust can glide it between equal
    // slots without measuring each button. Slot widths are equal by CSS.
    let mut html = String::from("<i class=\"nav-pill\" aria-hidden=\"true\"></i>");
    for (index, item) in items.iter().enumerate() {
        let current = index == selected;
        html.push_str(&format!(
            "<button class=\"nav-item {}\" data-action=\"navigate:{}\" data-index=\"{index}\" style=\"--slot:{}\" {}>{}<span>{}</span></button>",
            if current { "selected" } else { "" },
            esc(s(item, "route")),
            index as f64 / count as f64,
            if current { "aria-current=\"page\"" } else { "" },
            icon_pair(s(item, "icon")),
            esc(&text(&item["text"]))
        ));
    }
    html
}

pub fn content(p: &Value, data: &Value, live: &Value) -> String {
    let mut html = String::new();
    let status = s(data, "status");
    if !status.is_empty() {
        html.push_str(&format!(
            "<div class=status role=status>{}</div>",
            esc(status)
        ));
    }
    if let Some(nodes) = p["nodes"].as_array() {
        for item in nodes {
            html.push_str(&node(item, data, live, true));
        }
    }
    html
}
fn node(v: &Value, data: &Value, live: &Value, parent_enabled: bool) -> String {
    let active = parent_enabled && v["enabled"].as_bool().unwrap_or(true);
    let children = || {
        v["children"]
            .as_array()
            .map(|list| {
                list.iter()
                    .map(|child| node(child, data, live, active))
                    .collect::<String>()
            })
            .unwrap_or_default()
    };
    let action_id = s(v, "action");
    let insets = v["inset"].as_f64().unwrap_or(0.0);
    let gap = v["gap"].as_f64().unwrap_or(8.0);
    let weight = v["weight"].as_f64().unwrap_or(0.0);
    let style = format!(
        "--inset:{insets}px;--gap:{gap}px;{}",
        if weight > 0.0 {
            "flex:1;min-width:0;"
        } else {
            ""
        }
    );
    match s(v,"kind"){
        "card"=>format!("<div class=\"card {}\" style=\"{style}--padding:{}px\"{}>{}</div>",if action_id.is_empty(){""}else{"pressable"},v["padding"].as_f64().unwrap_or(20.0),attr(action_id,active),children()),
        "row"=>format!("<div class=row style=\"{style};margin-bottom:{}px\">{}</div>",v["bottom"].as_f64().unwrap_or(0.0),children()),
        "column"=>format!("<div class=column style=\"{style}\">{}</div>",children()),
        "text"=>format!("<div class=\"text {}\">{}</div>",esc(s(v,"style")),esc(&text(&v["text"]))),
        "icon"=>format!("<span class=\"tile-icon\">{}</span>",icon(s(v,"icon"))),
        "artwork"=>format!("<span style=\"width:{}px;flex:0 0 {}px\">{}</span>",n(v,"size"),n(v,"size"),art(s(v,"url"),b(v,"avatar"),"")),
        "button"=>if action_id.is_empty(){String::new()}else{format!("<button class=button style=\"{style}\" data-action=\"{}\" {}>{}</button>",esc(action_id),if active{""}else{"disabled"},esc(&text(&v["text"])))},
        "section"=>{
            let control=if action_id.is_empty(){String::new()}else{action(action_id,&label(if action_id.starts_with("refresh"){"refresh"}else if action_id=="clear_search"{"clear_history"}else{"view_all"}),if action_id.starts_with("refresh"){"refresh"}else if action_id=="clear_search"{"trash"}else{"chevron"},"section-action")};
            format!("<div class=section><h2>{}</h2>{control}</div>",esc(&text(&v["text"])))
        }
        "preference"|"radio"|"switch"=>{
            let kind=s(v,"kind");
            let selected=b(v,"selected")||b(v,"checked");
            let trailing=if kind=="switch"{format!("<span class=\"switch {}\" role=switch aria-checked=\"{}\"><i></i></span>",if b(v,"checked"){"checked"}else{""},b(v,"checked"))}else if kind=="radio"{format!("<span class=\"radio {}\" role=radio aria-checked=\"{}\">{}</span>",if selected{"checked"}else{""},selected,icon("check"))}else{format!("<span class=pref-tail>{}</span>",icon("chevron"))};
            let summary=text(&v["summary"]);
            let glyph_html = glyph(s(v,"glyph"));
            format!("<button class=\"preference pref-{kind}\" data-action=\"{}\" {} aria-pressed=\"{}\">{glyph_html}<span class=pref-text><strong>{}</strong>{}</span>{}{trailing}</button>",
                esc(action_id),
                if active{""}else{"disabled"},
                selected,
                esc(&text(&v["text"])),
                if summary.is_empty(){String::new()}else{format!("<small>{}</small>",esc(&summary))},
                trailing_value(&v["value"]),
            )
        }
        "shelf"=>{
            let id=s(v,"source");let playlists=b(v,"playlists");
            let cards=source(data,id).iter().take(v["limit"].as_u64().unwrap_or(12)as usize).enumerate().map(|(index,item)|{
                let url=if playlists{s(item,"cover").to_owned()}else{cover(item)};
                format!("<button class=shelf-card data-action=\"{}:{id}:{index}\">{}<strong>{}</strong><small>{}</small></button>",if playlists{"playlist"}else{"song"},art(&url,false,""),esc(s(item,"title")),if playlists{esc(&label("curated_playlist"))}else{esc(s(item,"artist"))})
            }).collect::<String>();format!("<div class=shelf>{cards}</div>")
        }
        "songs"=>{
            let id=s(v,"source");let start=n(v,"start").max(0)as usize;let limit=n(v,"limit");
            let visible=if limit>0{limit as usize}else{data["visibleLimit"].as_u64().unwrap_or(100)as usize};
            let loaded=source(data,id).len();
            let mut rows=source(data,id).iter().enumerate().skip(start).take(visible).map(|(index,song)|song_row(song,id,index,s(song,"mid")==s(live,"mid"))).collect::<String>();
            if limit<=0{
                let rendered=(start+visible).min(loaded);
                let render_pending=loaded>rendered;
                let fetch_pending=b(data,"detailHasMore");
                if render_pending||fetch_pending{
                    // The page loads another service page when the rendered batch is
                    // exhausted, so long playlists are never silently capped.
                    let loading=b(data,"detailLoadingMore")||b(data,"detailLoading");
                    let total=n(data,"detailTotal").max(loaded as i32);
                    let note=if total as usize>loaded{
                        format!("<span class=more-note>{}</span>",esc(&text(&json!({"resource":"songs_loaded","args":[loaded,total]}))))
                    }else{
                        String::new()
                    };
                    let control=if loading{
                        format!("<span class=more-loading><i class=spinner></i>{}</span>",esc(&label("loading_more_songs")))
                    }else{
                        button("local_more","load_more_songs",false)
                    };
                    rows.push_str(&format!("<div class=local-more>{note}{control}</div>"));
                }
            }
            rows
        }
        "playlists"=>{
            let filter=s(v,"filter");let id=s(v,"source");
            let cards=source(data,id).iter().enumerate().filter(|(_,p)|filter=="all"||filter.is_empty()||b(p,"saved")==(filter=="saved")).map(|(index,p)|{
                format!("<button class=playlist-row data-action=\"playlist:{id}:{index}\">{}<span><strong>{}</strong><small>{} · {} 首</small></span>{}</button>",art(s(p,"cover"),false,""),esc(s(p,"title")),label(if b(p,"saved"){"filter_saved"}else{"filter_created"}),n(p,"count"),icon("chevron"))
            }).collect::<String>();
            if cards.is_empty(){format!("<div class=empty>{}</div>",esc(&label("no_filtered_playlists")))}else{cards}
        }
        "tabs"=>format!("<div class=segmented>{}</div>",v["items"].as_array().map(|items|items.iter().map(|item|format!("<button class=\"{}\" data-action=\"filter:{}\" aria-pressed=\"{}\">{}</button>",if item["id"]==v["selection"]{"selected"}else{""},esc(s(item,"id")),item["id"]==v["selection"],esc(&text(&item["text"])))).collect::<String>()).unwrap_or_default()),
        "chips"=>{
            let values=if s(v,"source")=="history"{source(data,"history").to_vec()}else{source(data,"artists").to_vec()};
            format!("<div class=chips>{}</div>",values.iter().map(|q|{let q=q.as_str().unwrap_or("");format!("<button data-action=\"query:{}\">{}</button>",esc(q),esc(q))}).collect::<String>())
        }
        "search_input"=>format!("<div class=search-field>{}<input id=search-input type=search inputmode=search enterkeyhint=search placeholder=\"{}\" aria-label=\"{}\" value=\"{}\" autocomplete=off /><button data-action=search_submit aria-label=\"{}\">{}</button></div>",icon("search"),esc(&label("search_hint")),esc(&label("search")),esc(s(data,"query")),label("search"),icon("chevron")),
        "feed_status"=>{
            let error = s(data, "discoveryError");
            let loading = b(data, "discoveryLoading");
            let can_load = b(data, "loggedIn") && b(data, "homeReady") && b(data, "discoveryMore");
            let msg = if !error.is_empty() {
                error.to_owned()
            } else if loading {
                label("discovery_loading")
            } else if !b(data, "loggedIn") {
                label("login_recommendations")
            } else if b(data, "discoveryMore") {
                label("discovery_scroll")
            } else {
                label("discovery_end")
            };
            let control = if !error.is_empty() {
                button("load_more", "retry", false)
            } else if can_load && !loading {
                button("load_more", "load_more_songs", false)
            } else {
                String::new()
            };
            format!("<div class=feed-status role=status>{}{}{}</div>", if loading { "<i class=spinner></i>" } else { "" }, esc(&msg), control)
        }
        "loading"=>format!("<div class=loading role=status><i class=spinner></i>{}</div>",esc(&text(&v["text"]))),
        "platform"=>slot(s(v,"slot"),data,live),_=>String::new()
    }
}
fn song_row(song: &Value, id: &str, index: usize, selected: bool) -> String {
    format!("<button id=\"song-{id}-{index}\" class=\"song-row {}\" data-action=\"song:{id}:{index}\">{}<span><strong>{}</strong><small>{}</small></span><small class=song-duration>{}</small>{}</button>",if selected{"selected"}else{""},art(&cover(song),false,""),esc(s(song,"title")),esc(s(song,"artist")),format_time(n(song,"duration")*1000),if selected{icon("pause")}else if b(song,"vip"){"<em>VIP</em>".into()}else{String::new()})
}
fn seek() -> String {
    format!("<div class=seek><input type=range min=0 max=1000 step=1 value=0 data-seek aria-label=\"{}\" /></div>",label("playback_progress"))
}
fn times() -> String {
    "<div class=times><span data-time=position>00:00</span><span data-time=duration>00:00</span></div>".into()
}
fn transport(live: &Value, mini: bool) -> String {
    let play = action(
        "play",
        &label(if b(live, "playing") { "pause" } else { "play" }),
        if b(live, "playing") { "pause" } else { "play" },
        "play-control",
    );
    format!(
        "<div class=\"transport {}\">{}{play}{}</div>",
        if mini { "compact-controls" } else { "" },
        action("previous", &label("previous"), "previous", ""),
        action("next", &label("next"), "next", "")
    )
}
pub fn mini(live: &Value, data: &Value) -> String {
    let saved = &data["currentSong"];
    let restored_cover = cover(saved);
    let image = if s(live, "cover").is_empty() {
        restored_cover.as_str()
    } else {
        s(live, "cover")
    };
    let title = if s(live, "title").is_empty() {
        s(saved, "title")
    } else {
        s(live, "title")
    };
    let artist = if s(live, "artist").is_empty() {
        s(saved, "artist")
    } else {
        s(live, "artist")
    };
    format!(
        "<div class=mini-line aria-hidden=true></div>\
         <div class=mini-identity>{art}<span class=mini-meta><strong>{title}</strong><small>{artist}</small></span>\
         <div class=mini-compact>{play}{queue}</div>\
         <button class=\"icon-button collapse-mini\" data-action=collapse_mini aria-label=\"{collapse}\">{down}</button></div>\
         <div class=mini-expanded><div class=mini-progress>{seek}{times}</div>{transport}<div class=mini-tools>{favorite}{queue}</div></div>",
        art = art(image, false, "mini-art"),
        title = escaped_or(title, "select_song"),
        artist = esc(artist),
        play = action(
            "play",
            &label(if b(live, "playing") { "pause" } else { "play" }),
            if b(live, "playing") { "pause" } else { "play" },
            ""
        ),
        queue = action("queue", &label("play_queue"), "queue", ""),
        collapse = esc(&label("collapse_player")),
        down = icon("down"),
        seek = seek(),
        times = times(),
        transport = transport(live, true),
        favorite = action(
            "favorite",
            &label("local_favorites"),
            "heart",
            if b(live, "favorite") { "active" } else { "" }
        ),
    )
}
pub fn lyrics(
    lines: &[(i32, String)],
    translations: &[(i32, String)],
    word_lines: &[Vec<crate::karaoke::Word>],
) -> String {
    if lines.is_empty() {
        format!(
            "<div class=lyric-empty><h2>{}</h2><p>{}</p></div>",
            label("no_lyrics"),
            label("no_lyrics_summary")
        )
    } else {
        lines.iter().enumerate().map(|(index,(time,line))|{
        let translated=translations.iter().find(|(at,_)|(*time-*at).abs()<=500).map(|(_,t)|format!("<small>{}</small>",esc(t))).unwrap_or_default();
        let words=word_lines.get(index).filter(|words|!words.is_empty()).map(|words|words.iter().enumerate().map(|(j,word)|{
            let glyphs=crate::karaoke::pieces(&word.text).iter().enumerate().map(|(k,g)|format!("<span id=glyph-{index}-{j}-{k} class=karaoke-glyph>{}</span>",esc(g))).collect::<String>();
            format!("<span id=word-{index}-{j} class=karaoke-word>{glyphs}</span>")
        }).collect::<String>());
        format!("<div id=lyric-{index} class=lyric-line data-action=\"lyric-seek:{time}\"><strong>{}</strong>{translated}</div>",words.unwrap_or_else(||esc(line)))
    }).collect::<String>()
    }
}
pub fn player(
    live: &Value,
    data: &Value,
    lines: &[(i32, String)],
    translations: &[(i32, String)],
    word_lines: &[Vec<crate::karaoke::Word>],
    render_lyrics: bool,
) -> String {
    let mut information = String::new();
    let song = &data["currentSong"];
    for (key, value) in [
        ("song_title", s(live, "title").to_owned()),
        ("song_artist", s(live, "artist").to_owned()),
        ("song_album", s(song, "album").to_owned()),
        ("song_duration", format_time(n(live, "duration"))),
    ] {
        information.push_str(&format!(
            "<div class=information-row><small>{}</small><strong>{}</strong></div>",
            label(key),
            escaped_or(&value, "information_unavailable")
        ));
    }
    let lyric_lines = if render_lyrics {
        lyrics(lines, translations, word_lines)
    } else {
        String::new()
    };
    let mode = match s(live, "mode") {
        "SHUFFLE" => "shuffle",
        "LOOP" => "repeat",
        "SINGLE" => "repeat_one",
        _ => "order",
    };
    let identity_art = if s(live, "cover").is_empty() {
        String::new()
    } else {
        format!(
            "<button class=identity-open data-action=player_cover aria-label=\"{}\">{}</button>",
            esc(&label("cover_page")),
            art(s(live, "cover"), false, "identity-art")
        )
    };
    let lyric_options = format!(
        "<div class=lyric-options>{}{}</div>",
        action("lyrics-appearance", &label("lyrics_style"), "text", ""),
        action(
            if crate::customization::get(data, "translation") == "on" {
                "ui:translation:off"
            } else {
                "ui:translation:on"
            },
            &label("translation"),
            "translation",
            ""
        )
    );
    format!("<div id=player-surface class=player-surface><div class=player-wash>{}</div></div><div class=player-safe><div class=player-toolbar>{}<span>{}</span>{}</div><div class=player-identity>{identity_art}<div><h2>{}</h2><p>{}</p></div></div><div id=player-pages class=player-pages><div class=player-strip><div class=information-pane>{information}</div><div class=cover-pane><button id=cover-art class=cover-art data-action=song-actions aria-label=\"{}\">{}</button></div><div id=lyric-viewport class=lyric-viewport><div id=lyric-lines class=lyric-lines>{lyric_lines}</div></div></div></div><div class=page-dots>{}</div><div class=player-footer>{lyric_options}<div class=cover-seek>{}{}</div><div class=playback-error role=status>{}</div>{}<div class=player-tools>{}{}{}{}{}</div><div class=lyric-progress>{}</div></div></div>",
        art(s(live,"cover"),false,"player-cover"),action("close_player",&label("collapse_player"),"down",""),label("now_playing"),action("song-actions",&label("song_actions"),"more",""),
        escaped_or(s(live,"title"),"not_playing"),esc(s(live,"artist")),label("song_actions"),art(s(live,"cover"),false,"player-cover"),
        ["player_info","player_cover","player_lyrics"].iter().enumerate().map(|(i,a)|format!("<button id=page-dot-{i} data-action={a} aria-label=\"{}\"></button>",label(["song_information","cover_page","lyrics_page"][i]))).collect::<String>(),
        seek(),times(),esc(s(live,"playbackStatus")),transport(live,false),
        action("mode",&label("playback_mode"),mode,if mode=="order"{""}else{"active"}),
        action("sleep",&label("sleep_timer"),"timer",if n(live,"sleepRemaining")>0||b(live,"sleepAfterTrack"){"active"}else{""}),
        action("audio-effects",&label("audio_effects"),"equalizer",""),action("queue",&label("play_queue"),"queue",""),action("song-actions",&label("song_actions"),"more-vertical",""),
        format!("{}{}",seek(),times())).replace("loading=\"lazy\"","loading=\"eager\"")
}
fn quality_badge(live: &Value) -> &str {
    match s(live, "actualQuality") {
        "sq" => "SQ",
        "hq" => "HQ",
        "standard" => "MP3",
        _ => "AUTO",
    }
}

fn slot(kind: &str, data: &Value, live: &Value) -> String {
    match kind{
    "quality"=>quality_choices(data,live),
    "custom_color"=>format!("<div class=card style=\"--inset:12px;--padding:20px\"><label class=color-picker>{}<input id=custom-color type=color value=\"#{}\" {} /></label><div class=hex-entry><input id=custom-color-hex type=text maxlength=7 aria-label=\"{}\" value=\"#{}\" spellcheck=false /><button data-action=custom_color_apply>{}</button></div><p id=custom-color-error class=summary role=status></p><div class=color-presets>{}</div></div>",label("source_custom"),esc(s(data,"monetColor")),if b(data,"monet"){""}else{"disabled"},label("color_hex"),esc(s(data,"monetColor")),label("apply_color"),["3482FF","7165C8","A34C68","B46A33","50836A","586C83","9C4263","77804D"].iter().map(|color|format!("<button data-action=\"custom-color:{color}\" aria-label=\"#{color}\" style=\"--swatch:#{color}\" {}>{}</button>",if b(data,"monet"){""}else{"disabled"},if s(data,"monetColor").eq_ignore_ascii_case(color){icon("check")}else{String::new()})).collect::<String>()),
    "color_preview"=>format!("<div class=color-preview><h3>{}</h3><p>{}</p><div><i></i><i></i><i></i><i></i></div></div>",label("preview_title"),label("preview_summary")),
    "cache"=>format!("<div class=cache-info><h3>{:.2} MB</h3><p>{}</p></div>",data["cacheBytes"].as_u64().unwrap_or(0)as f64/1048576.0,label("image_cache_summary")),_=>String::new()
}
}
fn quality_choices(data: &Value, live: &Value) -> String {
    let mut html = format!(
        "<div class=sheet-note>{} · {}</div>",
        label("quality_apply_summary"),
        quality_badge(live)
    );
    for quality in ["auto", "sq", "hq", "standard"] {
        html.push_str(&node(&json!({"kind":"radio","text":{"resource":format!("quality_{quality}")},"summary":{"resource":format!("quality_{quality}_summary")},"selected":s(data,"quality")==quality,"action":format!("quality:{quality}")}),data,live,true));
    }
    html.push_str(&node(&json!({"kind":"switch","text":{"resource":"quality_fallback"},"summary":{"resource":"quality_fallback_summary"},"checked":b(data,"quality_fallback"),"enabled":matches!(s(data,"quality"),"sq"|"hq"),"action":"fallback"}),data,live,true));
    html
}
pub struct Overlay {
    pub class: &'static str,
    pub title: String,
    pub header: String,
    pub items: String,
    pub message: String,
    close: &'static str,
}

impl Overlay {
    pub fn markup(&self, entering: bool) -> String {
        format!("<div class=modal-backdrop data-action={}></div><section id=sheet class=\"sheet {} {}\" role=dialog aria-modal=true aria-label=\"{}\"><div class=sheet-handle></div><header id=sheet-header>{}</header><div class=sheet-content><div id=sheet-items>{}</div><p id=sheet-message class=collection-message role=status>{}</p></div></section>",
            self.close, self.class, if entering{"enter"}else{""}, esc(&self.title), self.header, self.items, esc(&self.message))
    }
}

pub fn overlay(sheet: &str, data: &Value, live: &Value, can_back: bool) -> Option<Overlay> {
    let qr = b(data, "qrOpen");
    if !qr && sheet.is_empty() {
        return None;
    }
    let title = label(if qr {
        "login_qq"
    } else {
        match sheet {
            "queue" => "play_queue",
            "quality" => "audio_quality",
            "audio-effects" => "audio_effects",
            "lyrics-appearance" => "lyrics_style",
            "sleep" => "sleep_timer",
            "add-playlist" => "add_to_playlist",
            "logout" => "logout_title",
            "clear_recent" => "clear_recent",
            _ => "song_actions",
        }
    });
    let close = if qr { "dismiss_login" } else { "close_sheet" };
    let content = if qr {
        let qr_image = s(data, "qrImage");
        let image = if qr_image.is_empty() {
            String::new()
        } else {
            format!(
                "<img src=\"data:image/png;base64,{}\" alt=\"{}\"/>",
                esc(qr_image),
                label("login_qr")
            )
        };
        format!(
            "<div class=qr-content>{image}<p role=status>{}</p>{}{}</div>",
            esc(s(data, "qrStatus")),
            button("login", "get_qr_again", true),
            button("dismiss_login", "cancel", false)
        )
    } else {
        match sheet {
            "queue" => source(data, "queue")
                .iter()
                .enumerate()
                .map(|(i, song)| {
                    let mut row = song_row(song, "queue", i, s(song, "mid") == s(live, "mid"));
                    row = row.replace(&format!("song:queue:{i}"), &format!("queue:{i}"));
                    row
                })
                .collect(),
            "quality" => quality_choices(data, live),
            "lyrics-appearance" => ["lyricSize", "lyricAlign", "lyricWeight"]
                .iter()
                .map(|key| {
                    crate::customization::radios(data, key)
                        .iter()
                        .map(|v| node(v, data, live, true))
                        .collect::<String>()
                })
                .collect(),
            "audio-effects" => {
                let speed = live["speed"].as_f64().unwrap_or(1.0);
                let mut html=format!("<div class=effect-group><div class=effect-caption><button data-action=speed-reset>{}</button><span id=speed-value>×{speed:.2}</span></div><div class=effect-slider style=\"--speed-progress:{}%\"><input id=playback-speed type=range min=0.25 max=2 step=0.05 value={speed} aria-label=\"{}\" /></div></div><div class=effect-group>",label("playback_speed"),(speed-0.25)/1.75*100.0,label("playback_speed"));
                for (supported, title, action, icon_name) in [
                    (
                        b(data, "equalizerSupported"),
                        "system_equalizer",
                        "system-equalizer",
                        "equalizer",
                    ),
                    (b(data, "miSoundSupported"), "MiSound", "misound", "volume"),
                ] {
                    if supported {
                        html.push_str(&format!("<button class=action-row data-action={action}>{}<span>{}</span>{}</button>",icon(icon_name),if title=="MiSound"{title.into()}else{label(title)},icon("chevron")));
                    }
                }
                html.push_str("</div><div class=effect-group>");
                html.push_str(&node(&json!({"kind":"switch","text":{"resource":"playback_pause_fade"},"checked":b(live,"pauseFade"),"enabled":true,"action":"toggle-pause-fade"}),data,live,true));
                html.push_str("</div>");
                html
            }
            "sleep" => {
                let mut list = format!("<p class=sheet-note>{}</p>", label("sleep_lifecycle"));
                for minutes in [0, 15, 30, 60, 90, -1] {
                    list.push_str(&node(&json!({"kind":"radio","text":if minutes==0{json!({"resource":"sleep_off"})}else if minutes<0{json!({"resource":"sleep_after_track"})}else{json!({"resource":"sleep_minutes","args":[minutes]})},"selected":if minutes<0{b(live,"sleepAfterTrack")}else if minutes==0{n(live,"sleepRemaining")==0&&!b(live,"sleepAfterTrack")}else{n(live,"sleepMinutes")==minutes&&n(live,"sleepRemaining")>0},"enabled":!s(live,"mid").is_empty(),"action":format!("sleep:{minutes}")}),data,live,true));
                }
                list
            }
            "song-actions" => {
                let mut html=format!("<div class=action-identity>{}<span><strong>{}</strong><small>{} · {}</small></span></div><div class=action-group>",art(s(live,"cover"),false,""),esc(s(live,"title")),esc(s(live,"artist")),esc(s(&data["currentSong"],"album")));
                for (key, action, icon_name) in [
                    ("play_next", "enqueue-next", "plus"),
                    ("add_to_queue", "enqueue-tail", "add-queue"),
                    ("add_to_likes", "add:201", "heart"),
                    ("add_to_playlist", "add-playlist", "library"),
                    ("audio_quality", "quality", "quality"),
                    ("audio_effects", "audio-effects", "equalizer"),
                    ("share_song", "share-song", "share"),
                    ("song_information", "show-song-info", "info"),
                ] {
                    html.push_str(&format!("<button class=action-row data-action={action} {}>{}<span>{}</span>{}</button>",if b(data,"collectionBusy"){"disabled"}else{""},icon(icon_name),label(key),if action=="quality"{format!("<small>{}</small>",quality_badge(live))}else{String::new()}));
                    if action=="add-playlist" {html.push_str(&format!("<button class=action-row data-action=show-song-info>{}<span>{}：{}</span></button><button class=action-row data-action=show-song-info>{}<span>艺术家：{}</span></button>",icon("cover"),label("song_album"),escaped_or(s(&data["currentSong"],"album"),"information_unavailable"),icon("account"),esc(s(live,"artist"))));}
                }
                html.push_str("</div>");
                html
            }
            "add-playlist" => {
                let disabled = if b(data, "collectionBusy") {
                    "disabled"
                } else {
                    ""
                };
                let playlists=source(data,"playlists").iter().filter(|p|!b(p,"saved")&&n(p,"dirid")>0).map(|p|format!("<button class=playlist-row data-action=\"add:{}\" {disabled}>{}<span><strong>{}</strong><small>{} 首</small></span>{}</button>",n(p,"dirid"),art(s(p,"cover"),false,""),esc(s(p,"title")),n(p,"count"),icon("chevron"))).collect::<String>();
                if playlists.is_empty() {
                    let hint = if !b(data, "libraryReady") && b(data, "busy") {
                        "loading_library"
                    } else {
                        "no_writable_playlists"
                    };
                    format!("<p class=sheet-note>{}</p>", label(hint))
                } else {
                    playlists
                }
            }
            "logout" => format!(
                "<p class=sheet-note>{}</p><div class=dialog-actions>{}{}</div>",
                label("logout_summary"),
                button("close_sheet", "cancel", false),
                button("logout_confirm", "logout", true)
            ),
            "clear_recent" => format!(
                "<p class=sheet-note>{}</p><div class=dialog-actions>{}{}</div>",
                label("clear_recent_summary"),
                button("close_sheet", "cancel", false),
                button("clear_recent_confirm", "clear_history", true)
            ),
            _ => String::new(),
        }
    };
    let back = if can_back && !qr {
        action("sheet_back", &label("back"), "back", "")
    } else {
        String::new()
    };
    let cancel = action(close, &label("cancel"), "close", "");
    Some(Overlay {
        class: if qr {
            "qr-sheet"
        } else if sheet == "song-actions" {
            "song-menu-sheet"
        } else if sheet == "audio-effects" {
            "effects-sheet"
        } else if can_back || sheet == "add-playlist" {
            "half-sheet"
        } else {
            ""
        },
        header: format!("{back}<h2>{}</h2>{cancel}", esc(&title)),
        title,
        items: content,
        message: s(data, "collectionMessage").to_owned(),
        close,
    })
}
pub fn floating(show_top: bool, locate: bool) -> String {
    format!(
        "{}{}",
        if locate {
            action("locate", &label("locate_playing"), "locate", "")
        } else {
            String::new()
        },
        if show_top {
            action("top", &label("back_to_top"), "up", "")
        } else {
            String::new()
        }
    )
}
