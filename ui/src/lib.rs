//! Auralis' active UI. DOM composition, navigation gestures, modal state and motion
//! run in Rust/WASM. JavaScript is generated bindings plus a small host bootstrap.
mod icons;
mod labels;
mod lyric_motion;
mod presentation;
mod render;

use serde_json::{json, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Element, Event, HtmlElement, HtmlInputElement, PointerEvent};

#[derive(Default)]
struct App {
    data: Value,
    live: Value,
    filter: String,
    sheet: String,
    sheet_stack: Vec<String>,
    player: bool,
    player_page: usize,
    scrolls: HashMap<String, i32>,
    limits: HashMap<String, usize>,
    active_line: Option<usize>,
    lyric_mid: String,
    lines: Vec<(i32, String)>,
    translations: Vec<(i32, String)>,
    browse_until: f64,
    lyric_scroll: f64,
    lyric_velocity: f64,
    row_motion: HashMap<usize, lyric_motion::RowMotion>,
    last_frame: f64,
    mini_until: f64,
    last_load: f64,
    pointer: Option<(f64, f64, String)>,
    suppress_click: f64,
    scrubbing: bool,
    reduced: bool,
}
thread_local! {static APP:RefCell<App>=RefCell::new(App{filter:"all".into(),player_page:1,..App::default()});}
fn doc() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}
fn el(id: &str) -> Option<Element> {
    doc().get_element_by_id(id)
}
fn now() -> f64 {
    web_sys::window()
        .unwrap()
        .performance()
        .map(|p| p.now())
        .unwrap_or(0.0)
}
fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn n(v: &Value, key: &str) -> i32 {
    v[key].as_i64().unwrap_or(0) as i32
}
fn b(v: &Value, key: &str) -> bool {
    v[key].as_bool().unwrap_or(false)
}
fn set_html(id: &str, html: &str) {
    if let Some(node) = el(id) {
        node.set_inner_html(html)
    }
}
fn style(node: &Element, key: &str, value: &str) {
    if let Some(node) = node.dyn_ref::<HtmlElement>() {
        let _ = node.style().set_property(key, value);
    }
}
fn class(node: &Element, name: &str, on: bool) {
    let _ = node.class_list().toggle_with_force(name, on);
}
fn post(packet: Value) {
    let window = web_sys::window().unwrap();
    let bridge = js_sys::Reflect::get(&window, &JsValue::from_str("AuralisNative")).ok();
    if let Some(bridge) = bridge.filter(|v| !v.is_undefined() && !v.is_null()) {
        if let Ok(method) = js_sys::Reflect::get(&bridge, &JsValue::from_str("postMessage")) {
            if let Some(f) = method.dyn_ref::<js_sys::Function>() {
                let _ = f.call1(&bridge, &JsValue::from_str(&packet.to_string()));
            }
        }
    } else {
        web_sys::console::log_1(&JsValue::from_str(&format!("Auralis action: {packet}")));
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(|info| {
        web_sys::console::error_1(&JsValue::from_str(&format!("Auralis Rust UI: {info}")))
    }));
    let document = doc();
    let css = document.create_element("style")?;
    css.set_text_content(Some(include_str!("style.css")));
    document.head().unwrap().append_child(&css)?;
    document.body().unwrap().set_inner_html("<div id=app><header id=header></header><main id=content tabindex=-1></main><div id=float-actions></div><footer id=dock><div id=mini-player class=mini data-action=open_player role=button tabindex=0></div><nav id=navigation></nav></footer><section id=player aria-hidden=true inert></section><div id=overlays></div></div>");
    let click = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        let target = event
            .target()
            .and_then(|t| t.dyn_into::<Element>().ok())
            .and_then(|e| e.closest("[data-action]").ok().flatten());
        if let Some(target) = target {
            if target.has_attribute("disabled") {
                return;
            }
            event.stop_propagation();
            let action = target.get_attribute("data-action").unwrap_or_default();
            APP.with(|state| {
                let mut app = state.borrow_mut();
                if now() >= app.suppress_click {
                    app.act(&action)
                }
            });
        }
    });
    document.add_event_listener_with_callback("click", click.as_ref().unchecked_ref())?;
    click.forget();
    let input = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        if let Some(input) = event
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            if input.has_attribute("data-seek") {
                APP.with(|state| {
                    state.borrow_mut().scrubbing = true;
                });
                let fraction = input.value_as_number().clamp(0.0, 1000.0) / 1000.0;
                if let Some(parent) = input.parent_element() {
                    style(&parent, "--progress", &format!("{}%", fraction * 100.0));
                }
            }
        }
    });
    document.add_event_listener_with_callback("input", input.as_ref().unchecked_ref())?;
    input.forget();
    let change = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        if let Some(input) = event
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            // A search submission blurs its field synchronously. Its change event
            // must not re-enter UI state while the submit handler owns that state.
            if !input.has_attribute("data-seek") && input.id() != "custom-color" {
                return;
            }
            APP.with(|state|{let mut app=state.borrow_mut();
                if input.has_attribute("data-seek"){app.scrubbing=false;app.mini_until=now()+4500.0;post(json!({"type":"seek","position":(input.value_as_number()/1000.0*n(&app.live,"duration")as f64)as i32}));}
                if input.id()=="custom-color"{post(json!({"type":"monet","key":"color","value":input.value().trim_start_matches('#')}));}
            });
        }
    });
    document.add_event_listener_with_callback("change", change.as_ref().unchecked_ref())?;
    change.forget();
    let key = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(|event: web_sys::KeyboardEvent| {
        if event.key() == "Enter"
            && event
                .target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .map(|e| e.id() == "search-input")
                .unwrap_or(false)
        {
            event.prevent_default();
            APP.with(|s| s.borrow_mut().act("search_submit"));
        } else if matches!(event.key().as_str(), "Enter" | " ") {
            if let Some(target) = event
                .target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .filter(|e| {
                    e.tag_name() != "BUTTON" && e.get_attribute("role").as_deref() == Some("button")
                })
            {
                if let Some(action) = target.get_attribute("data-action") {
                    event.prevent_default();
                    APP.with(|s| s.borrow_mut().act(&action));
                }
            }
        }
    });
    document.add_event_listener_with_callback("keydown", key.as_ref().unchecked_ref())?;
    key.forget();
    let down = Closure::<dyn FnMut(PointerEvent)>::new(|event: PointerEvent| {
        let element = event.target().and_then(|t| t.dyn_into::<Element>().ok());
        let zone = element
            .as_ref()
            .and_then(|e| e.closest("#player-pages").ok().flatten())
            .map(|_| "player")
            .unwrap_or("");
        APP.with(|state| {
            let mut app = state.borrow_mut();
            app.pointer = Some((
                event.client_x() as f64,
                event.client_y() as f64,
                zone.into(),
            ));
            if element
                .and_then(|e| e.closest("#lyric-viewport").ok().flatten())
                .is_some()
            {
                app.browse_until = f64::INFINITY;
            }
        });
    });
    document.add_event_listener_with_callback("pointerdown", down.as_ref().unchecked_ref())?;
    down.forget();
    let up = Closure::<dyn FnMut(PointerEvent)>::new(|event: PointerEvent| {
        APP.with(|state| {
            let mut app = state.borrow_mut();
            if let Some((x, y, zone)) = app.pointer.take() {
                let dx = event.client_x() as f64 - x;
                let dy = event.client_y() as f64 - y;
                if zone == "player" && dx.abs() > 65.0 && dx.abs() > dy.abs() * 1.35 {
                    app.player_page = (app.player_page as i32 + if dx < 0.0 { 1 } else { -1 })
                        .clamp(0, 2) as usize;
                    app.suppress_click = now() + 350.0;
                    app.show_player_page();
                }
            }
            if app.browse_until.is_infinite() {
                app.browse_until = now() + 1800.0;
            }
        });
    });
    document.add_event_listener_with_callback("pointerup", up.as_ref().unchecked_ref())?;
    up.forget();
    let cancel = Closure::<dyn FnMut(Event)>::new(|_| {
        APP.with(|s| {
            let mut app = s.borrow_mut();
            app.pointer = None;
            if app.browse_until.is_infinite() {
                app.browse_until = now() + 1800.0;
            }
        })
    });
    document.add_event_listener_with_callback("pointercancel", cancel.as_ref().unchecked_ref())?;
    cancel.forget();
    let scroll = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        if event
            .target()
            .and_then(|t| t.dyn_into::<Element>().ok())
            .map(|e| e.id() == "content")
            .unwrap_or(false)
        {
            APP.with(|s| s.borrow_mut().scrolled());
        }
    });
    document.add_event_listener_with_callback_and_bool(
        "scroll",
        scroll.as_ref().unchecked_ref(),
        true,
    )?;
    scroll.forget();
    install_animation();
    Ok(())
}

#[wasm_bindgen]
pub fn update(input: &str) {
    let packet: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => return,
    };
    APP.with(|state| {
        let mut app = state.borrow_mut();
        match s(&packet, "type") {
            "content" => {
                app.save_scroll();
                app.data = packet["state"].clone();
                app.reduced = b(&app.data, "reducedMotion");
                app.paint_content();
                if app.player {
                    app.paint_player();
                }
                app.paint_overlay();
                if b(&app.data, "openPlayer") && !app.player {
                    app.open_player();
                }
            }
            "playback" => {
                let identity = s(&app.live, "mid") != s(&packet["state"], "mid");
                let toggle = b(&app.live, "playing") != b(&packet["state"], "playing");
                let controls = [
                    "mode",
                    "favorite",
                    "actualQuality",
                    "playbackStatus",
                    "sleepAfterTrack",
                    "qualityPending",
                ]
                .iter()
                .any(|key| app.live[key] != packet["state"][key]);
                app.live = packet["state"].clone();
                if identity || toggle || controls {
                    if b(&app.live, "playing") {
                        app.mini_until = now() + 4500.0;
                    }
                    if identity {
                        app.active_line = None;
                        if matches!(app.sheet.as_str(), "song-actions" | "add-playlist") {
                            app.sheet.clear();
                            app.paint_overlay();
                        }
                    }
                    app.paint_mini();
                    if app.player {
                        app.paint_player();
                    }
                }
                app.paint_progress();
                app.sync_lyrics();
            }
            "open_player" => app.open_player(),
            _ => {}
        }
    });
}

#[wasm_bindgen]
pub fn system_back() -> bool {
    APP.with(|state| {
        let mut app = state.borrow_mut();
        if b(&app.data, "qrOpen") {
            post(json!({"type":"dismiss_login"}));
            return true;
        }
        if !app.sheet.is_empty() {
            app.sheet = app.sheet_stack.pop().unwrap_or_default();
            app.paint_overlay();
            return true;
        }
        if app.player {
            app.close_player();
            return true;
        }
        if s(&app.data, "page") != "home" {
            post(json!({"type":"back"}));
            return true;
        }
        false
    })
}

impl App {
    fn save_scroll(&mut self) {
        if let Some(main) = el("content").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
            self.scrolls.insert(self.route_key(), main.scroll_top());
        }
    }
    fn route_key(&self) -> String {
        format!(
            "{}:{}",
            s(&self.data, "page"),
            if s(&self.data, "page") == "songs" {
                s(&self.data, "detailTitle")
            } else {
                ""
            }
        )
    }
    fn source(&self, id: &str) -> &[Value] {
        self.data[id].as_array().map(Vec::as_slice).unwrap_or(&[])
    }
    fn act(&mut self, action: &str) {
        if action.starts_with("navigate:") {
            self.save_scroll();
            post(json!({"type":"navigate","page":&action[9..]}));
            return;
        }
        if let Some(source) = action.strip_prefix("open:") {
            post(json!({"type":"open_collection","source":source}));
            return;
        }
        if let Some(rest) = action.strip_prefix("song:") {
            let mut p = rest.split(':');
            if let (Some(source), Some(index)) =
                (p.next(), p.next().and_then(|s| s.parse::<usize>().ok()))
            {
                if let Some(song) = self.source(source).get(index) {
                    post(json!({"type":"play","source":source,"index":index,"mid":song["mid"]}));
                }
                self.mini_until = now() + 4500.0;
            }
            return;
        }
        if let Some(rest) = action.strip_prefix("playlist:") {
            let mut parts = rest.split(':');
            let source = parts.next().unwrap_or("");
            let index = parts
                .next()
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(usize::MAX);
            if let Some(p) = self.source(source).get(index) {
                post(
                    json!({"type":"playlist","source":source,"index":index,"id":p["id"],"dirid":p["dirid"],"title":p["title"],"cover":p["cover"],"kind":p["kind"]}),
                );
            }
            return;
        }
        if let Some(filter) = action.strip_prefix("filter:") {
            self.save_scroll();
            self.filter = filter.into();
            self.paint_content();
            return;
        }
        if let Some(color) = action.strip_prefix("custom-color:") {
            if b(&self.data, "monet") {
                post(json!({"type":"monet","key":"color","value":color}));
            }
            return;
        }
        if action == "custom_color_apply" {
            if !b(&self.data, "monet") {
                return;
            }
            if let Some(input) =
                el("custom-color-hex").and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
            {
                let value = input.value().trim().trim_start_matches('#').to_uppercase();
                let valid = value.len() == 6 && value.bytes().all(|c| c.is_ascii_hexdigit());
                if valid {
                    post(json!({"type":"monet","key":"color","value":value}));
                    let _ = input.blur();
                }
                let message = labels::label("color_hex_error");
                if let Some(error) = el("custom-color-error") {
                    error.set_text_content(if valid { None } else { Some(&message) });
                }
            }
            return;
        }
        if let Some(value) = action.strip_prefix("appearance:") {
            post(json!({"type":"appearance","value":value}));
            return;
        }
        for prefix in ["monet-source:", "monet-style:", "monet-spec:"] {
            if let Some(value) = action.strip_prefix(prefix) {
                post(
                    json!({"type":"monet","key":prefix.trim_end_matches(':').trim_start_matches("monet-"),"value":value}),
                );
                return;
            }
        }
        if let Some(value) = action.strip_prefix("quality:") {
            post(json!({"type":"quality","value":value}));
            return;
        }
        if let Some(value) = action.strip_prefix("sleep:") {
            post(json!({"type":"sleep","minutes":value.parse::<i32>().unwrap_or(0)}));
            self.sheet.clear();
            self.paint_overlay();
            return;
        }
        if let Some(index) = action
            .strip_prefix("queue:")
            .and_then(|v| v.parse::<usize>().ok())
        {
            post(json!({"type":"play","source":"queue","index":index}));
            self.sheet.clear();
            self.paint_overlay();
            return;
        }
        if let Some(dirid) = action
            .strip_prefix("add:")
            .and_then(|s| s.parse::<i64>().ok())
        {
            post(json!({"type":"add_cloud","mid":s(&self.live,"mid"),"dirid":dirid}));
            return;
        }
        if let Some(query) = action.strip_prefix("query:") {
            post(json!({"type":"search","query":query}));
            return;
        }
        if let Some(time) = action
            .strip_prefix("lyric-seek:")
            .and_then(|v| v.parse::<i32>().ok())
        {
            self.browse_until = 0.0;
            post(json!({"type":"seek","position":time}));
            return;
        }
        match action {
            "back" => {
                post(json!({"type":"back"}));
            }
            "open_player" => self.open_player(),
            "close_player" => self.close_player(),
            "player_info" => {
                self.player_page = 0;
                self.show_player_page();
            }
            "player_cover" => {
                self.player_page = 1;
                self.show_player_page();
            }
            "player_lyrics" => {
                self.player_page = 2;
                self.show_player_page();
            }
            "play" => {
                self.mini_until = now() + 4500.0;
                post(
                    json!({"type":"transport","action":if b(&self.live,"playing"){"PAUSE"}else{"RESUME"}}),
                );
            }
            "previous" | "next" => {
                self.mini_until = now() + 4500.0;
                post(json!({"type":"transport","action":action.to_uppercase()}));
            }
            "toggle_monet" => post(json!({"type":"toggle_monet"})),
            "favorite" => post(json!({"type":"favorite"})),
            "mode" => post(json!({"type":"cycle_mode"})),
            "fallback" => post(json!({"type":"toggle_fallback"})),
            "daily" | "login" | "refresh_home" | "refresh_library" | "retry_playlist"
            | "clear_search" | "dismiss_login" | "load_more" => post(json!({"type":action})),
            "search_submit" => {
                if let Some(input) =
                    el("search-input").and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
                {
                    let query = input.value().trim().to_owned();
                    let _ = input.blur();
                    post(json!({"type":"search","query":query}));
                }
            }
            "top" => {
                if let Some(main) = el("content") {
                    main.scroll_to_with_x_and_y(0.0, 0.0);
                }
            }
            "locate" => {
                let mid = s(&self.live, "mid");
                if let Some(index) = self
                    .source("tracks")
                    .iter()
                    .position(|v| s(v, "mid") == mid)
                {
                    if index >= *self.limits.get(&self.route_key()).unwrap_or(&100) {
                        self.save_scroll();
                        self.limits.insert(self.route_key(), index + 80);
                        self.paint_content();
                    }
                    if let (Some(row), Some(main)) =
                        (el(&format!("song-tracks-{index}")), el("content"))
                    {
                        if let Some(row) = row.dyn_ref::<HtmlElement>() {
                            main.scroll_to_with_x_and_y(0.0, (row.offset_top() - 24).max(0) as f64);
                        }
                    }
                }
            }
            "collapse_mini" => {
                self.mini_until = 0.0;
            }
            "local_more" => self.extend_list(),
            "logout_confirm" => {
                self.sheet.clear();
                post(json!({"type":"logout"}));
                self.paint_overlay();
            }
            "clear_recent_confirm" => {
                self.sheet.clear();
                post(json!({"type":"clear_recent"}));
                self.paint_overlay();
            }
            "close_sheet" => {
                self.sheet.clear();
                self.sheet_stack.clear();
                self.paint_overlay();
            }
            "sheet_back" => {
                self.sheet = self.sheet_stack.pop().unwrap_or_default();
                self.paint_overlay();
            }
            "play_tracks" => post(json!({"type":"play","source":"tracks","index":0})),
            "shuffle_tracks" => post(json!({"type":"shuffle_tracks"})),
            "queue" | "quality" | "sleep" | "song-actions" | "add-playlist" | "logout"
            | "clear_recent" => {
                if self.sheet == "song-actions"
                    && matches!(action, "quality" | "sleep" | "add-playlist")
                {
                    self.sheet_stack.push(self.sheet.clone());
                } else {
                    self.sheet_stack.clear();
                }
                self.sheet = action.into();
                if action == "song-actions" {
                    self.data["collectionMessage"] = json!("");
                    post(json!({"type":"prepare_actions"}));
                }
                self.paint_overlay();
                if action == "add-playlist" {
                    post(json!({"type":"refresh_library"}));
                }
            }
            _ => {}
        }
    }
    fn paint_content(&mut self) {
        let mut input = self.data.clone();
        input["filter"] = json!(self.filter);
        input["visibleLimit"] = json!(*self.limits.get(&self.route_key()).unwrap_or(&100));
        if let Ok(presentation) = presentation::screen(&input) {
            set_html("header", &render::header(&presentation, &self.data));
            set_html(
                "content",
                &render::content(&presentation, &input, &self.live),
            );
            set_html("navigation", &render::navigation(&presentation));
            if let Some(dock) = el("dock") {
                class(&dock, "no-nav", !b(&presentation, "root"));
                class(
                    &dock,
                    "hidden",
                    !b(&presentation, "root") && s(&self.data, "page").starts_with("monet"),
                );
            }
            if let Some(main) = el("content").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
                let _ = main.style().set_property("scroll-behavior", "auto");
                main.set_scroll_top(*self.scrolls.get(&self.route_key()).unwrap_or(&0));
                let _ = main.style().remove_property("scroll-behavior");
            }
        }
        if let Some(root) = el("app") {
            style(&root,"--lyric-blur-unit",&format!("{}px",3.0/web_sys::window().unwrap().device_pixel_ratio().max(1.0)));
            for key in [
                "bg",
                "card",
                "fg",
                "muted",
                "primary",
                "on-primary",
                "border",
            ] {
                if let Some(color) = self.data["colors"][key].as_str() {
                    style(&root, &format!("--{key}"), color);
                }
            }
            style(
                &root,
                "--safe-top",
                &format!("{}px", n(&self.data, "safeTop")),
            );
            style(
                &root,
                "--safe-bottom",
                &format!("{}px", n(&self.data, "safeBottom")),
            );
            class(&root, "reduced-motion", self.reduced);
            let _ = root.set_attribute("data-route", s(&self.data, "page"));
        }
        self.paint_mini();
        self.paint_progress();
        self.scrolled();
    }
    fn paint_mini(&self) {
        set_html("mini-player", &render::mini(&self.live, &self.data));
    }
    fn paint_player(&mut self) {
        let old_scroll = el("lyric-viewport")
            .and_then(|e| e.dyn_into::<HtmlElement>().ok())
            .map(|e| e.scroll_top())
            .unwrap_or(0);
        let mid = s(&self.live, "mid");
        let timeline = s(&self.data, "lyrics");
        let identity = format!("{mid}:{timeline}");
        if self.lyric_mid != identity {
            self.lines = parse_lrc(timeline);
            self.translations = parse_lrc(s(&self.data, "translation"));
            self.lyric_mid = identity;
            self.active_line = None;
            self.lyric_velocity = 0.0;
            self.row_motion.clear();
            self.lyric_scroll = 0.0;
        }
        set_html(
            "player",
            &render::player(&self.live, &self.data, &self.lines, &self.translations),
        );
        self.active_line = None;
        if let Some(viewport) = el("lyric-viewport").and_then(|e| e.dyn_into::<HtmlElement>().ok())
        {
            viewport.set_scroll_top(old_scroll);
        }
        self.show_player_page();
        self.paint_progress();
        self.sync_lyrics();
    }
    fn show_player_page(&mut self) {
        if let Some(pages) = el("player-pages") {
            style(&pages, "--page", &self.player_page.to_string());
        }
        if let Some(player) = el("player") {
            class(&player, "lyrics-page", self.player_page == 2);
        }
        for index in 0..3 {
            if let Some(dot) = el(&format!("page-dot-{index}")) {
                class(&dot, "selected", index == self.player_page);
            }
        }
        if let (Some(pane), Some(cover)) = (
            doc()
                .query_selector(".cover-pane")
                .ok()
                .flatten()
                .and_then(|e| e.dyn_into::<HtmlElement>().ok()),
            el("cover-art"),
        ) {
            let size = (pane.client_width() - 64)
                .min(pane.client_height() - 40)
                .min(400)
                .max(0);
            style(&cover, "width", &format!("{size}px"));
            style(&cover, "height", &format!("{size}px"));
        }
    }
    fn open_player(&mut self) {
        if self.player {
            return;
        }
        self.player = true;
        self.player_page = 1;
        self.paint_player();
        if let Some(player) = el("player") {
            self.player_origin(&player);
            class(&player, "mounted", true);
            class(&player, "visible", false);
            let _ = player.set_attribute("aria-hidden", "false");
            let _ = player.remove_attribute("inert");
            let _ = player.get_bounding_client_rect();
            let cb = Closure::once_into_js(move || {
                class(&player, "visible", true);
            });
            let _ = web_sys::window()
                .unwrap()
                .request_animation_frame(cb.unchecked_ref());
        }
    }
    fn close_player(&mut self) {
        self.player = false;
        if let Some(player) = el("player") {
            self.player_origin(&player);
            class(&player, "visible", false);
            let _ = player.set_attribute("aria-hidden", "true");
            let _ = player.set_attribute("inert", "");
        }
    }
    fn player_origin(&self, player: &Element) {
        if let Some(mini) = el("mini-player") {
            let r = mini.get_bounding_client_rect();
            let window = web_sys::window().unwrap();
            let w = window
                .inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(390.0);
            let h = window
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(800.0);
            style(player, "--origin-x", &format!("{}px", r.x()));
            style(player, "--origin-y", &format!("{}px", r.y()));
            style(player, "--origin-sx", &(r.width() / w).to_string());
            style(player, "--origin-sy", &(r.height() / h).to_string());
        }
    }
    fn paint_overlay(&self) {
        set_html(
            "overlays",
            &render::overlay(
                &self.sheet,
                &self.data,
                &self.live,
                !self.sheet_stack.is_empty(),
            ),
        );
    }
    fn paint_progress(&self) {
        let duration = n(&self.live, "duration").max(0);
        let position = n(&self.live, "position").clamp(0, duration.max(1));
        let fraction = if duration > 0 {
            position as f64 / duration as f64
        } else {
            0.0
        };
        if let Ok(nodes) = doc().query_selector_all(".seek") {
            for i in 0..nodes.length() {
                if let Some(node) = nodes.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                    if !self.scrubbing {
                        style(&node, "--progress", &format!("{}%", fraction * 100.0));
                        if let Some(range) = node
                            .first_element_child()
                            .and_then(|n| n.dyn_into::<HtmlInputElement>().ok())
                        {
                            range.set_value(&(fraction * 1000.0).to_string());
                        }
                    }
                }
            }
        }
        for (key, time) in [("position", position), ("duration", duration)] {
            if let Ok(nodes) = doc().query_selector_all(&format!("[data-time={key}]")) {
                for i in 0..nodes.length() {
                    if let Some(node) = nodes.item(i) {
                        node.set_text_content(Some(&format_time(time)));
                    }
                }
            }
        }
    }
    fn sync_lyrics(&mut self) {
        if !self.player || self.lines.is_empty() {
            return;
        }
        let position = n(&self.live, "position");
        let next = self
            .lines
            .partition_point(|(time, _)| *time <= position)
            .checked_sub(1);
        if next == self.active_line {
            return;
        }
        self.active_line = next;
        for index in 0..self.lines.len() {
            if let Some(line) = el(&format!("lyric-{index}")) {
                let selected = Some(index) == next;
                let distance = next
                    .map(|active| index.abs_diff(active))
                    .unwrap_or(0)
                    .min(5);
                class(&line, "active", selected);
                style(&line, "--distance", &distance.to_string());
                style(
                    &line,
                    "--wave-delay",
                    &format!(
                        "{}ms",
                        next.map(|active| index.saturating_sub(active).min(5) * 34)
                            .unwrap_or(0)
                    ),
                );
            }
        }
    }
    fn scrolled(&mut self) {
        if let Some(main) = el("content").and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
            let page = s(&self.data, "page");
            let top = main.scroll_top();
            set_html(
                "float-actions",
                &render::floating(
                    top > 700,
                    page == "songs"
                        && self
                            .source("tracks")
                            .iter()
                            .any(|v| s(v, "mid") == s(&self.live, "mid")),
                ),
            );
            if main.scroll_height() - top - main.client_height() < 500
                && doc()
                    .query_selector("#content .local-more")
                    .ok()
                    .flatten()
                    .is_some()
            {
                self.extend_list();
                return;
            }
            if page == "home"
                && b(&self.data, "homeReady")
                && main.scroll_height() - top - main.client_height() < 650
                && !b(&self.data, "discoveryLoading")
                && b(&self.data, "discoveryMore")
                && s(&self.data, "discoveryError").is_empty()
                && now() - self.last_load > 1800.0
            {
                self.last_load = now();
                post(json!({"type":"load_more"}));
            }
        }
    }
    fn extend_list(&mut self) {
        self.save_scroll();
        let route = self.route_key();
        let next = self.limits.get(&route).copied().unwrap_or(100) + 100;
        self.limits.insert(route, next);
        self.paint_content();
    }
    fn frame(&mut self, time: f64) {
        if let Some(mini) = el("mini-player") {
            class(&mini, "expanded", self.mini_until > time);
        }
        let dt = ((time - self.last_frame) / 1000.0).clamp(0.0, 0.032);
        self.last_frame = time;
        if self.player && self.player_page == 2 && time > self.browse_until {
            if let (Some(index), Some(viewport)) = (
                self.active_line,
                el("lyric-viewport").and_then(|v| v.dyn_into::<HtmlElement>().ok()),
            ) {
                if let Some(lines) = el("lyric-lines") {
                    let top = format!("{}px", viewport.client_height() as f64 * 0.36);
                    if lines
                        .dyn_ref::<HtmlElement>()
                        .and_then(|e| e.style().get_property_value("padding-top").ok())
                        .as_deref()
                        != Some(&top)
                    {
                        style(&lines, "padding-top", &top);
                        style(
                            &lines,
                            "padding-bottom",
                            &format!("{}px", viewport.client_height() as f64 * 0.64),
                        );
                    }
                }
                if let Some(line) =
                    el(&format!("lyric-{index}")).and_then(|v| v.dyn_into::<HtmlElement>().ok())
                {
                    let target = (line.offset_top() as f64
                        - viewport.client_height() as f64 * 0.36)
                        .max(0.0);
                    let old = viewport.scroll_top() as f64;
                    let destination = target.round();
                    let travel = destination - old;
                    if travel.abs() > 1.0 {
                        let visible: Vec<usize> = (0..self.lines.len())
                            .filter(|i| {
                                el(&format!("lyric-{i}"))
                                    .and_then(|e| e.dyn_into::<HtmlElement>().ok())
                                    .map(|e| {
                                        e.offset_top() as f64 + e.offset_height() as f64 > old
                                            && (e.offset_top() as f64)
                                                < old + viewport.client_height() as f64
                                    })
                                    .unwrap_or(false)
                            })
                            .collect();
                        viewport.set_scroll_top(destination as i32);
                        for i in 0..self.lines.len() {
                            let rank =
                                visible
                                    .iter()
                                    .position(|v| *v == i)
                                    .unwrap_or(if i < index {
                                        0
                                    } else {
                                        visible.len().saturating_sub(1)
                                    });
                            self.row_motion
                                .entry(i)
                                .or_default()
                                .place(travel, lyric_motion::delay(rank, visible.len()));
                        }
                    }
                    let gap = self
                        .lines
                        .get(index + 1)
                        .map(|l| l.0 - self.lines[index].0)
                        .unwrap_or(1000);
                    for (i, row) in &mut self.row_motion {
                        let offset = if self.reduced {
                            *row = Default::default();
                            0.0
                        } else {
                            row.sample(dt, lyric_motion::stiffness(gap))
                        };
                        if let Some(element) = el(&format!("lyric-{i}")) {
                            style(&element, "translate", &format!("0 {offset}px"));
                        }
                    }
                }
            }
        }
        if let Some(viewport) = el("lyric-viewport") {
            let browsing = time <= self.browse_until;
            class(&viewport, "browsing", browsing);
            if browsing {
                self.row_motion.clear();
                if let Ok(rows) = viewport.query_selector_all(".lyric-line") {
                    for i in 0..rows.length() {
                        if let Some(row) = rows.item(i).and_then(|e| e.dyn_into::<Element>().ok()) {
                            style(&row, "translate", "0 0");
                        }
                    }
                }
            }
        }
    }
}

fn install_animation() {
    let callback = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));
    let next = callback.clone();
    *callback.borrow_mut() = Some(Closure::new(move |time: f64| {
        APP.with(|s| s.borrow_mut().frame(time));
        if let Some(cb) = next.borrow().as_ref() {
            let _ = web_sys::window()
                .unwrap()
                .request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }));
    if let Some(cb) = callback.borrow().as_ref() {
        let _ = web_sys::window()
            .unwrap()
            .request_animation_frame(cb.as_ref().unchecked_ref());
    };
}
fn format_time(ms: i32) -> String {
    let seconds = ms.max(0) / 1000;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}
fn parse_lrc(input: &str) -> Vec<(i32, String)> {
    let mut lines = Vec::new();
    for line in input.lines() {
        let mut rest = line;
        let mut times = Vec::new();
        while let Some(tail) = rest.strip_prefix('[') {
            let Some((tag, next)) = tail.split_once(']') else {
                break;
            };
            rest = next;
            if let Some((min, sec)) = tag.split_once(':') {
                if let (Ok(min), Ok(sec)) = (min.parse::<i32>(), sec.parse::<f64>()) {
                    times.push(min * 60_000 + (sec * 1000.0).round() as i32);
                }
            }
        }
        let text = rest.trim();
        if !text.is_empty() && text != "//" {
            for time in times {
                lines.push((time, text.to_owned()));
            }
        }
    }
    lines.sort_by_key(|(time, _)| *time);
    lines
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lrc_multiple_tags_and_subsecond_timing() {
        let lines = parse_lrc("[ar:test]\n[00:01.05][00:03.500]你好\n[00:02]世界");
        assert_eq!(
            lines,
            vec![
                (1050, "你好".into()),
                (2000, "世界".into()),
                (3500, "你好".into())
            ]
        );
    }
    #[test]
    fn formatted_time_never_shows_negative_positions() {
        assert_eq!(format_time(-2), "00:00");
        assert_eq!(format_time(125500), "02:05");
    }
}
