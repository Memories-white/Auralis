//! Auralis' active UI. DOM composition, navigation gestures, modal state and motion
//! run in Rust/WASM. JavaScript is generated bindings plus a small host bootstrap.
mod artwork;
mod customization;
mod icons;
mod karaoke;
mod labels;
mod lyric_motion;
mod motion;
mod presentation;
mod render;
mod shell;

use serde_json::{json, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Element, Event, HtmlElement, HtmlInputElement, PointerEvent};

/// Geometry of the shared cover element while it travels between the mini player
/// and the cover page. All values are viewport pixels measured once per transition.
#[derive(Clone, Copy)]
struct CoverMorph {
    from: [f64; 4],
    to: [f64; 4],
    radius_from: f64,
    radius_to: f64,
}

#[derive(Clone)]
struct SheetLayers {
    player: Element,
    surface: Element,
    content: Element,
    cover: Element,
    artwork: Element,
    mini: Element,
    dock: Element,
    nav: Element,
    overlay: Option<Element>,
    width: f64,
    height: f64,
}

#[derive(Clone)]
struct OverlayLayers {
    sheet: Element,
    backdrop: Element,
    height: f64,
}
#[derive(Clone)]
struct HeaderLayers {
    header: Element,
    large: Element,
    compact: Element,
}
#[derive(Default)]
struct App {
    data: Value,
    live: Value,
    filter: String,
    detail_artists: bool,
    mini_lines: Vec<(i32, String)>,
    painted_route: String,
    sheet: String,
    sheet_stack: Vec<String>,
    painted_overlay: Option<render::Overlay>,
    player: bool,
    player_page: usize,
    scrolls: HashMap<String, i32>,
    limits: HashMap<String, usize>,
    active_line: Option<usize>,
    lyric_mid: String,
    lines: Vec<(i32, String)>,
    translations: Vec<(i32, String)>,
    word_lines: Vec<Vec<karaoke::Word>>,
    word_nodes: Vec<Vec<HtmlElement>>,
    glyph_nodes: Vec<Vec<Vec<HtmlElement>>>,
    glyph_geometry: Vec<Vec<Vec<(f64, f64)>>>,
    glyph_fills: Vec<Vec<Vec<i32>>>,
    word_fills: Vec<Vec<i32>>,
    browse_until: f64,
    lyric_scroll: f64,
    lyric_velocity: f64,
    row_motion: HashMap<usize, lyric_motion::RowMotion>,
    lyric_nodes: Vec<HtmlElement>,
    lyric_nodes_pending: bool,
    lyric_viewport: Option<HtmlElement>,
    lyric_dirty: bool,
    lyric_height: i32,
    browsing_last: bool,
    clock_at: f64,
    mini_expanded: bool,
    mini_written: f64,
    dock_extra: f64,
    mini: motion::Spring,
    dock: motion::Spring,
    nav_element: Option<Element>,
    nav_nodes: Vec<Element>,
    dock_ready: bool,
    dock_page: String,
    dock_count: usize,
    sheet_spring: motion::Spring,
    morph: Option<CoverMorph>,
    sheet_layers: Option<SheetLayers>,
    lyrics_mounted: bool,
    sheet_bounds: [f64; 4],
    sheet_radius: f64,
    pager: motion::Spring,
    pager_layer: Option<Element>,
    pager_panes: Vec<Element>,
    pager_dragging: bool,
    pager_start: f64,
    pager_width: f64,
    last_frame: f64,
    mini_until: f64,
    cover_flip_until: f64,
    effect_stamp: i32,
    ambient_phase: f64,
    wash_transform: String,
    player_markup: String,
    player_identity: String,
    player_layout: String,
    last_load: f64,
    pointer: Option<(f64, f64, String)>,
    suppress_click: f64,
    scrubbing: bool,
    reduced: bool,
    animation_vote: bool,
    overlay_open: bool,
    overlay_closing: bool,
    overlay_spring: motion::Spring,
    overlay_layers: Option<OverlayLayers>,
    sheet_drag: Option<(f64, f64)>,
    header_layers: Option<HeaderLayers>,
    content_layer: Option<Element>,
    route_spring: motion::Spring,
    route_direction: f64,
}
thread_local! {static APP:RefCell<App>=RefCell::new(App{filter:"all".into(),player_page:1,dock_page:"home".into(),..App::default()});}
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
        artwork::replace(&node, html)
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
fn rect(node: &Element) -> [f64; 4] {
    let r = node.get_bounding_client_rect();
    [r.x(), r.y(), r.width(), r.height()]
}
fn viewport() -> (f64, f64) {
    let window = web_sys::window().unwrap();
    (
        window
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(390.0),
        window
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(800.0),
    )
}
/// Resolve a computed `border-radius` to pixels; percentages resolve against the
/// shorter side. This is what lets a `50%` circular cover morph continuously.
fn corner_radius(node: &Element, side: f64) -> f64 {
    let value = web_sys::window()
        .and_then(|w| w.get_computed_style(node).ok().flatten())
        .map(|style| {
            style
                .get_property_value("border-top-left-radius")
                .unwrap_or_default()
        })
        .unwrap_or_default();
    motion::length(&value, side).clamp(0.0, side.max(0.0))
}
fn find(selector: &str) -> Option<Element> {
    doc().query_selector(selector).ok().flatten()
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
    document.body().unwrap().set_inner_html("<div id=app><header id=header></header><main id=content tabindex=-1></main><div id=float-actions></div><footer id=dock><div id=mini-player class=mini data-action=open_player role=button tabindex=0></div><nav id=navigation role=tablist aria-label=主导航></nav></footer><section id=player aria-hidden=true inert></section><div id=overlays></div></div>");
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
            if input.id() == "playback-speed" {
                let speed = input.value_as_number().clamp(0.25, 2.0);
                if let Some(track) = input.parent_element() {
                    style(
                        &track,
                        "--speed-progress",
                        &format!("{}%", (speed - 0.25) / 1.75 * 100.0),
                    );
                }
                if let Some(label) = el("speed-value") {
                    label.set_text_content(Some(&format!("×{speed:.2}")));
                }
            }
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
            if !input.has_attribute("data-seek")
                && !matches!(input.id().as_str(), "custom-color" | "playback-speed")
            {
                return;
            }
            APP.with(|state|{let mut app=state.borrow_mut();
                if input.has_attribute("data-seek"){app.scrubbing=false;app.mini_until=now()+4500.0;post(json!({"type":"seek","position":(input.value_as_number()/1000.0*n(&app.live,"duration")as f64)as i32}));}
                if input.id()=="custom-color"{post(json!({"type":"monet","key":"color","value":input.value().trim_start_matches('#')}));}
                if input.id()=="playback-speed"{post(json!({"type":"audio_settings","speed":input.value_as_number().clamp(0.25,2.0),"fade":b(&app.live,"pauseFade")}));}
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
            let zone = if zone.is_empty()
                && event.client_x() < 24
                && !app.overlay_open
                && !app.player
                && shell::settings(s(&app.data, "page"))
                && s(&app.data, "page") != "settings"
            {
                "back"
            } else {
                zone
            };
            app.pointer = Some((
                event.client_x() as f64,
                event.client_y() as f64,
                zone.into(),
            ));
            if zone == "player" {
                app.pager_start = app.pager.position;
                app.pager_width = app
                    .pager_layer
                    .as_ref()
                    .map(|e| e.client_width() as f64)
                    .unwrap_or(1.0)
                    .max(1.0);
                app.pager_dragging = false;
            }
            if !app.overlay_closing
                && element.as_ref().is_some_and(|e| {
                    e.closest(".sheet-handle,#sheet-header")
                        .ok()
                        .flatten()
                        .is_some()
                        && e.closest("button").ok().flatten().is_none()
                })
            {
                app.sheet_drag = Some((
                    event.client_y() as f64,
                    app.overlay_spring.position.clamp(0.0, 1.0),
                ));
            }

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
    let sheet_move = Closure::<dyn FnMut(PointerEvent)>::new(|event: PointerEvent| {
        APP.with(|state| {
            let mut app = state.borrow_mut();
            if let Some((x, y, zone)) = app.pointer.as_ref() {
                if zone == "player" && app.player {
                    let dx = event.client_x() as f64 - *x;
                    let dy = event.client_y() as f64 - *y;
                    if app.pager_dragging || (dx.abs() > 10.0 && dx.abs() > dy.abs() * 1.35) {
                        if !app.pager_dragging {
                            app.pager_dragging = true;
                        }
                        let width = app.pager_width;
                        let p = app.pager_start - dx / width;
                        let p = if p < 0.0 {
                            p * 0.15
                        } else if p > 2.0 {
                            2.0 + (p - 2.0) * 0.15
                        } else {
                            p
                        };
                        app.pager.place(p);
                        app.paint_pager();
                        if p > 1.0 {
                            app.ensure_lyrics();
                            if app.lyric_nodes_pending {
                                app.cache_lyric_nodes();
                                app.sync_lyrics();
                            }
                        }
                        app.suppress_click = now() + 350.0;
                    }
                }
            }
            if let Some((y, start)) = app.sheet_drag {
                let dy = (event.client_y() as f64 - y).max(0.0);
                if let Some(layers) = app.overlay_layers.as_ref() {
                    let p = (start - dy / layers.height.max(1.0)).clamp(0.0, 1.0);
                    app.overlay_spring.place(p);
                    app.apply_overlay();
                }
                if dy > 8.0 {
                    app.suppress_click = now() + 350.0;
                }
            }
        });
    });
    document
        .add_event_listener_with_callback("pointermove", sheet_move.as_ref().unchecked_ref())?;
    sheet_move.forget();

    let up = Closure::<dyn FnMut(PointerEvent)>::new(|event: PointerEvent| {
        APP.with(|state| {
            let mut app = state.borrow_mut();
            if app.sheet_drag.take().is_some() {
                if app.overlay_spring.position < 0.82 {
                    if b(&app.data, "qrOpen") {
                        app.data["qrOpen"] = json!(false);
                        post(json!({"type":"dismiss_login"}));
                    }
                    app.sheet.clear();
                    app.sheet_stack.clear();
                    app.paint_overlay();
                } else {
                    app.overlay_spring.to(1.0);
                }
            }

            if let Some((x, y, zone)) = app.pointer.take() {
                let dx = event.client_x() as f64 - x;
                let dy = event.client_y() as f64 - y;
                if zone == "back" && dx > 72.0 && dx > dy.abs() * 1.35 {
                    app.route_direction = -1.0;
                    app.save_scroll();
                    app.suppress_click = now() + 350.0;
                    post(json!({"type":"back"}));
                }
                if zone == "player" && dx.abs() > 65.0 && dx.abs() > dy.abs() * 1.35 {
                    app.player_page = (app.player_page as i32 + if dx < 0.0 { 1 } else { -1 })
                        .clamp(0, 2) as usize;
                    app.suppress_click = now() + 350.0;
                    app.show_player_page();
                }
                if zone == "player" {
                    app.pager_dragging = false;
                    let target = app.player_page as f64;
                    app.pager.to(target);
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
            app.pager_dragging = false;
            let target = app.player_page as f64;
            app.pager.to(target);
            if app.sheet_drag.take().is_some() {
                app.overlay_spring.to(1.0);
            }

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
    let resize = Closure::<dyn FnMut(Event)>::new(|_: Event| {
        APP.with(|state| {
            let mut app = state.borrow_mut();
            app.layout_shell();
            app.cache_overlay_layers();
            app.sheet_drag = None;
            app.pointer = None;
            app.pager_dragging = false;
            let page = app.player_page as f64;
            app.pager.place(page);
            app.pager.to(page);
            app.paint_pager();
            app.sync_dock();
            app.fit_cover();
            app.unmount_morph();
            let target = if app.player { 1.0 } else { 0.0 };
            app.sheet_spring.place(target);
            app.sheet_spring.to(target);
            app.cache_sheet_layers();
            app.apply_sheet(target);
            app.lyric_dirty = true;
        });
    });
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref())?;
    resize.forget();
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
                app.reduced = b(&app.data, "reducedMotion")
                    || customization::get(&app.data, "motion") == "reduced";
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
                let had_track = !s(&app.live, "mid").is_empty();
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
                app.clock_at = now();
                if identity || toggle || controls {
                    if b(&app.live, "playing") {
                        app.mini_until = now() + 4500.0;
                    }
                    if identity {
                        app.cover_flip_until = if had_track
                            && app.player
                            && app.sheet_spring.settled()
                            && app.player_page == 1
                        {
                            now() + 720.0
                        } else {
                            0.0
                        };
                        app.active_line = None;
                        if matches!(app.sheet.as_str(), "song-actions" | "add-playlist") {
                            app.sheet.clear();
                            app.sheet_stack.clear();
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
                if matches!(app.sheet.as_str(), "audio-effects") {
                    app.paint_overlay();
                }
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
        if app.overlay_closing {
            return true;
        }
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
        if !app.sheet_spring.settled() {
            return true;
        }
        if s(&app.data, "page") != "home" {
            app.route_direction = -1.0;
            app.save_scroll();
            post(json!({"type":"back"}));
            return true;
        }
        false
    })
}

impl App {
    fn layout_shell(&mut self) {
        let (w, h) = viewport();
        let wide = shell::wide(w, h);
        let safe = n(&self.data, "safeBottom").max(0) as f64;
        if let Some(root) = el("app") {
            let _ = root.set_attribute("data-wide", if wide { "true" } else { "false" });
            let floating = customization::get(&self.data, "bar") == "floating";
            let root_page = matches!(
                s(&self.data, "page"),
                "home" | "collection" | "mine" | "settings"
            );
            let bottom = shell::floating_bottom(safe);
            let metrics = shell::dock_metrics(wide, root_page, floating, safe);
            self.dock_extra = metrics.extra;
            let clearance = if shell::dock_visible(s(&self.data, "page")) {
                metrics.clearance
            } else {
                safe + 16.0
            };
            style(&root, "--dock-base-clearance", &format!("{clearance}px"));
            style(
                &root,
                "--dock-clearance",
                &format!("calc({clearance}px + var(--dock-extra, 0px))"),
            );
            style(&root, "--floating-bottom", &format!("{bottom}px"));
            let count = self.dock_count.max(1);
            style(&root, "--floating-width", &format!("{}px", 8 + count * 100));
        }
        self.header_layers = (|| {
            Some(HeaderLayers {
                header: el("header")?,
                large: find("#header .large-title")?,
                compact: find("#header .compact-title")?,
            })
        })();
        self.content_layer = el("content");
        self.paint_header_scroll();
    }
    fn paint_header_scroll(&self) {
        let Some(layers) = self.header_layers.as_ref() else {
            return;
        };
        let (w, h) = viewport();
        let page = s(&self.data, "page");
        if page == "about" {
            let top = self
                .content_layer
                .as_ref()
                .map(|e| e.scroll_top() as f64)
                .unwrap_or(0.0);
            for (selector, start, end) in [
                (".about-version", 20.0, 80.0),
                (".about-identity h2", 80.0, 140.0),
                (".about-logo", 140.0, 200.0),
            ] {
                if let Some(node) = find(selector) {
                    let p = ((top - start) / (end - start)).clamp(0.0, 1.0);
                    style(&node, "opacity", &(1.0 - p).to_string());
                    style(&node, "transform", &format!("scale({})", 1.0 - p * 0.05));
                }
            }
        }
        let collapse = if page == "songs" {
            (self
                .content_layer
                .as_ref()
                .map(|e| e.scroll_top() as f64)
                .unwrap_or(0.0)
                / 120.0)
                .clamp(0.0, 1.0)
        } else if !shell::settings(page) {
            0.0
        } else if shell::wide(w, h) {
            1.0
        } else {
            shell::collapse(
                self.content_layer
                    .as_ref()
                    .map(|e| e.scroll_top() as f64)
                    .unwrap_or(0.0),
            )
        };
        style(&layers.header, "--collapse", &format!("{collapse:.4}"));
        style(&layers.large, "opacity", &(1.0 - collapse).to_string());
        style(
            &layers.large,
            "transform",
            &format!("translateY({}px)", -collapse * 12.0),
        );
        style(&layers.compact, "opacity", &collapse.to_string());
    }
    fn cache_overlay_layers(&mut self) {
        self.overlay_layers = (|| {
            let sheet = el("sheet")?;
            Some(OverlayLayers {
                height: rect(&sheet)[3],
                sheet,
                backdrop: find(".modal-backdrop")?,
            })
        })();
    }
    fn apply_overlay(&self) {
        if let Some(layers) = self.overlay_layers.as_ref() {
            let p = self.overlay_spring.position.clamp(0.0, 1.0);
            style(
                &layers.sheet,
                "transform",
                &format!("translate3d(0,{:.3}px,0)", layers.height * (1.0 - p)),
            );
            style(&layers.backdrop, "opacity", &p.to_string());
        }
    }
    fn advance_shell(&mut self, dt: f64) {
        if !self.route_spring.settled() {
            if self.reduced {
                self.route_spring.place(0.0);
            } else {
                self.route_spring.sample(
                    dt,
                    motion::Feel {
                        stiffness: 26.0,
                        damping: 1.0,
                    },
                );
            }
            if let Some(content) = self.content_layer.as_ref() {
                let p = self.route_spring.position.clamp(0.0, 1.0);
                style(
                    content,
                    "transform",
                    &format!("translate3d({}px,0,0)", 24.0 * self.route_direction * p),
                );
                style(content, "opacity", &(1.0 - p * 0.35).to_string());
            }
        }
        if self.sheet_drag.is_none() && !self.overlay_spring.settled() {
            if self.reduced {
                self.overlay_spring.place(self.overlay_spring.target);
            } else {
                self.overlay_spring.sample(dt, motion::Feel::REVEAL);
            }
            self.apply_overlay();
        }
        if self.overlay_closing && self.overlay_spring.settled() {
            set_html("overlays", "");
            self.painted_overlay = None;
            self.overlay_layers = None;
            self.overlay_open = false;
            self.overlay_closing = false;
        }
    }
    fn save_scroll(&mut self) {
        if let Some(main) = el("content").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
            self.scrolls.insert(self.route_key(), main.scroll_top());
        }
    }
    fn route_key(&self) -> String {
        format!(
            "{}:{}:{}",
            s(&self.data, "page"),
            if s(&self.data, "page") == "songs" {
                s(&self.data, "detailTitle")
            } else {
                ""
            },
            s(&self.data, "page") == "songs" && self.detail_artists
        )
    }
    fn source(&self, id: &str) -> &[Value] {
        self.data[id].as_array().map(Vec::as_slice).unwrap_or(&[])
    }
    fn act(&mut self, action: &str) {
        if let Some(artist) = action.strip_prefix("artist-search:") {
            post(json!({"type":"search","query":artist}));
            return;
        }
        if action == "detail-songs" || action == "detail-artists" {
            self.save_scroll();
            self.detail_artists = action == "detail-artists";
            self.paint_content();
            return;
        }
        if action.starts_with("navigate:") {
            if &action[9..] == s(&self.data, "page") {
                return;
            }
            self.route_direction =
                if matches!(&action[9..], "home" | "collection" | "mine" | "settings") {
                    0.0
                } else {
                    1.0
                };
            self.save_scroll();
            post(json!({"type":"navigate","page":&action[9..]}));
            return;
        }
        if let Some(key) = action.strip_prefix("link:") {
            if let Some(url) = shell::project_link(key) {
                post(json!({"type":"open_external","url":url}));
            }
            return;
        }
        if action == "copy-project" {
            post(json!({"type":"copy_project"}));
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
        if let Some(rest) = action.strip_prefix("ui:") {
            if let Some((key, value)) = rest.split_once(':') {
                if customization::set(&mut self.data, key, value) {
                    self.reduced = b(&self.data, "reducedMotion")
                        || customization::get(&self.data, "motion") == "reduced";
                    post(json!({"type":"ui_preferences","value":self.data["uiPreferences"]}));
                    self.paint_content();
                    if self.player {
                        self.paint_player();
                    }
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
            if !b(&self.data, "collectionBusy") {
                post(json!({"type":"add_cloud","mid":s(&self.live,"mid"),"dirid":dirid}));
            }
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
                self.route_direction = -1.0;
                self.save_scroll();
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
            "enqueue-next" | "enqueue-tail" => {
                post(
                    json!({"type":"enqueue","mid":s(&self.live,"mid"),"next":action=="enqueue-next"}),
                );
            }
            "share-song" | "system-equalizer" | "misound" => post(json!({"type":action})),
            "show-song-info" => {
                self.sheet.clear();
                self.sheet_stack.clear();
                self.paint_overlay();
                self.player_page = 0;
                self.show_player_page();
            }
            "speed-reset" | "toggle-pause-fade" => post(
                json!({"type":"audio_settings","speed":if action=="speed-reset"{1.0}else{self.live["speed"].as_f64().unwrap_or(1.0)},"fade":if action=="toggle-pause-fade"{!b(&self.live,"pauseFade")}else{b(&self.live,"pauseFade")}}),
            ),
            "previous" | "next" => {
                self.mini_until = now() + 4500.0;
                post(json!({"type":"transport","action":action.to_uppercase()}));
            }
            "toggle_monet" => post(json!({"type":"toggle_monet"})),
            "favorite" => post(json!({"type":"favorite"})),
            "mode" => post(json!({"type":"cycle_mode"})),
            "fallback" => post(json!({"type":"toggle_fallback"})),
            "daily" | "login" | "refresh_home" | "refresh_library" | "retry_playlist"
            | "clear_search" | "dismiss_login" | "load_more" | "retry_lyrics" => {
                post(json!({"type":action}))
            }
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
            "local_more" => {
                if self.render_pending() {
                    self.extend_list();
                } else if s(&self.data, "page") == "songs"
                    && b(&self.data, "detailHasMore")
                    && !b(&self.data, "detailLoadingMore")
                {
                    self.last_load = 0.0;
                    post(json!({"type":"load_more_tracks"}));
                } else {
                    // Discovery shelves grow through the same rendering batch.
                    self.extend_list();
                }
            }
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
            // Playing the whole list resolves every remaining service page first, so
            // the queue is never limited to the rows fetched for the current screen.
            "play_tracks" => post(json!({"type":"play_all","shuffle":false})),
            "shuffle_tracks" => post(json!({"type":"play_all","shuffle":true})),
            "queue" | "quality" | "sleep" | "song-actions" | "add-playlist" | "logout"
            | "audio-effects" | "lyrics-appearance" | "clear_recent" => {
                if self.sheet == "song-actions"
                    && matches!(
                        action,
                        "quality" | "sleep" | "add-playlist" | "audio-effects"
                    )
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
                if action == "add-playlist" && !b(&self.data, "libraryReady") {
                    post(json!({"type":"refresh_library"}));
                }
            }
            _ => {}
        }
    }
    fn paint_content(&mut self) {
        // Request the restored current-song image before recommendation shelves.
        self.paint_mini();
        let route = self.route_key();
        let route_changed = route != self.painted_route;
        self.painted_route = route;
        let mut input = self.data.clone();
        input["filter"] = json!(self.filter);
        input["detailArtists"] = json!(self.detail_artists);
        input["visibleLimit"] = json!(*self.limits.get(&self.route_key()).unwrap_or(&100));
        if let Ok(presentation) = presentation::screen(&input) {
            set_html("header", &render::header(&presentation, &self.data));
            set_html(
                "content",
                &render::content(&presentation, &input, &self.live),
            );
            set_html("navigation", &render::navigation(&presentation));
            if route_changed {
                self.route_spring
                    .place(if self.reduced { 0.0 } else { 1.0 });
                self.route_spring.to(0.0);
            }
            if let Some(dock) = el("dock") {
                class(&dock, "no-nav", !b(&presentation, "root"));
                class(
                    &dock,
                    "hidden",
                    !b(&presentation, "root") && s(&self.data, "page").starts_with("monet"),
                );
            }
            self.dock_count = presentation["navigation"]
                .as_array()
                .map(|items| items.len())
                .unwrap_or(0);
        }
        if let Some(root) = el("app") {
            for spec in customization::OPTIONS {
                let _ = root.set_attribute(
                    &format!("data-ui-{}", spec.key.to_lowercase()),
                    customization::get(&self.data, spec.key),
                );
            }
            style(
                &root,
                "--lyric-size",
                &format!("{}px", customization::get(&self.data, "lyricSize")),
            );
            style(
                &root,
                "--lyric-weight",
                customization::get(&self.data, "lyricWeight"),
            );
            style(
                &root,
                "--lyric-blur-unit",
                &format!(
                    "{}px",
                    3.0 / web_sys::window().unwrap().device_pixel_ratio().max(1.0)
                ),
            );
            for key in [
                "bg",
                "card",
                "fg",
                "muted",
                "primary",
                "on-primary",
                "border",
                "card-fg",
                "section-fg",
                "divider",
                "secondary",
                "on-secondary",
                "disabled",
                "error",
                "window-dimming",
                "disabled-primary",
                "disabled-secondary",
                "disabled-on-primary",
                "disabled-on-secondary",
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
            let _ = root.set_attribute(
                "data-settings-shell",
                if shell::settings(s(&self.data, "page")) {
                    "true"
                } else {
                    "false"
                },
            );
            let _ = root.set_attribute(
                "data-monet",
                if b(&self.data, "monet") {
                    "true"
                } else {
                    "false"
                },
            );
        }
        self.layout_shell();
        self.sync_dock();
        if let Some(main) = el("content").and_then(|e| e.dyn_into::<HtmlElement>().ok()) {
            let restore = *self.scrolls.get(&self.route_key()).unwrap_or(&0);
            let options = web_sys::ScrollToOptions::new();
            options.set_top(restore as f64);
            options.set_behavior(web_sys::ScrollBehavior::Auto);
            main.scroll_to_with_scroll_to_options(&options);
        }
        self.paint_mini();
        self.paint_progress();
        self.scrolled();
    }
    fn paint_mini(&mut self) {
        self.mini_lines = parse_lrc(s(&self.data, "lyrics"));
        set_html("mini-player", &render::mini(&self.live, &self.data));
        self.paint_mini_pose();
        if self.sheet_layers.is_some() {
            self.cache_sheet_layers();
            self.apply_sheet(self.sheet_spring.position);
        }
    }
    fn paint_player(&mut self) {
        let old_scroll = el("lyric-viewport")
            .and_then(|e| e.dyn_into::<HtmlElement>().ok())
            .map(|e| e.scroll_top())
            .unwrap_or(0);
        let mid = s(&self.live, "mid");
        let timeline = s(&self.data, "lyrics");
        let identity = format!(
            "{mid}:{timeline}:{}:{}:{}",
            s(&self.data, "wordLyrics"),
            s(&self.data, "translation"),
            s(&self.data, "lyricState")
        );
        let document_changed = self.lyric_mid != identity;
        if document_changed {
            self.lines = parse_lrc(timeline);
            self.translations = parse_lrc(s(&self.data, "translation"));
            self.word_lines = karaoke::parse(s(&self.data, "wordLyrics"), &self.lines);
            self.lyric_mid = identity;
            self.active_line = None;
            self.lyric_velocity = 0.0;
            self.row_motion.clear();
            self.lyric_scroll = 0.0;
        }
        let player_identity = format!("{}|{}", mid, s(&self.live, "cover"));
        let retained = self.player_identity == player_identity && el("cover-art").is_some();
        let layout = self.data["uiPreferences"].to_string();
        let layout_changed = self.player_layout != layout;
        let render_lyrics = self.player_page == 2 || (retained && self.lyrics_mounted);
        let html = render::player(
            &self.live,
            &self.data,
            &self.lines,
            &self.translations,
            &self.word_lines,
            render_lyrics && !retained,
        );
        if self.player_markup == html && !document_changed && !layout_changed {
            self.show_player_page();
            self.cache_sheet_layers();
            self.apply_sheet(self.sheet_spring.position);
            self.paint_progress();
            return;
        }
        if retained {
            // The persistent cover, background, pager and lyric viewport must not
            // lose transforms/decoded images when only controls or lyrics change.
            if let Ok(staging) = doc().create_element("div") {
                staging.set_inner_html(&html);
                for selector in [
                    ".player-footer>.transport",
                    ".player-tools",
                    ".playback-error",
                    ".lyric-options",
                    ".information-pane",
                    ".player-identity h2",
                    ".player-identity p",
                ] {
                    if let (Ok(Some(next)), Some(current)) = (
                        staging.query_selector(selector),
                        find(&format!("#player {selector}")),
                    ) {
                        let markup = next.inner_html();
                        if current.inner_html() != markup {
                            current.set_inner_html(&markup);
                        }
                    }
                }
                if document_changed && render_lyrics {
                    set_html(
                        "lyric-lines",
                        &render::lyrics(
                            &self.lines,
                            &self.translations,
                            &self.word_lines,
                            &self.data,
                        ),
                    );
                }
            }
            artwork::invalidate_markup("player");
        } else {
            self.wash_transform.clear();
            set_html("player", &html);
        }
        self.player_markup = html;
        self.player_identity = player_identity;
        self.player_layout = layout;
        self.lyrics_mounted = render_lyrics;
        if !retained || document_changed || layout_changed {
            self.cache_lyric_nodes();
            self.lyric_dirty = true;
            self.active_line = None;
            self.lyric_height = 0;
            if let Some(viewport) =
                el("lyric-viewport").and_then(|e| e.dyn_into::<HtmlElement>().ok())
            {
                viewport.set_scroll_top(if retained { old_scroll } else { 0 });
            }
        }
        self.show_player_page();
        self.cache_sheet_layers();
        self.apply_sheet(self.sheet_spring.position);
        self.paint_progress();
        self.sync_lyrics();
    }

    fn cache_lyric_nodes(&mut self) {
        if el("lyric-viewport")
            .and_then(|e| e.dyn_into::<HtmlElement>().ok())
            .is_some_and(|e| {
                e.style()
                    .get_property_value("content-visibility")
                    .unwrap_or_default()
                    == "hidden"
            })
        {
            self.lyric_nodes_pending = true;
            return;
        }
        self.lyric_nodes_pending = false;
        self.glyph_fills = self
            .word_lines
            .iter()
            .map(|words| {
                words
                    .iter()
                    .map(|w| vec![-1; karaoke::pieces(&w.text).len()])
                    .collect()
            })
            .collect();
        self.glyph_nodes = self
            .word_lines
            .iter()
            .enumerate()
            .map(|(i, words)| {
                words
                    .iter()
                    .enumerate()
                    .map(|(j, w)| {
                        karaoke::pieces(&w.text)
                            .iter()
                            .enumerate()
                            .filter_map(|(k, _)| {
                                el(&format!("glyph-{i}-{j}-{k}")).and_then(|e| e.dyn_into().ok())
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect();
        self.glyph_geometry = self
            .glyph_nodes
            .iter()
            .map(|row| {
                row.iter()
                    .map(|word| {
                        let mut x = 0.0;
                        word.iter()
                            .map(|glyph| {
                                let left = x;
                                x += glyph.offset_width() as f64;
                                (left, x)
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect();
        self.lyric_viewport = el("lyric-viewport").and_then(|e| e.dyn_into().ok());
        self.word_nodes = self
            .word_lines
            .iter()
            .enumerate()
            .map(|(i, words)| {
                words
                    .iter()
                    .enumerate()
                    .filter_map(|(j, _)| {
                        el(&format!("word-{i}-{j}")).and_then(|e| e.dyn_into().ok())
                    })
                    .collect()
            })
            .collect();
        self.word_fills = self
            .word_lines
            .iter()
            .map(|words| vec![-1; words.len()])
            .collect();
        self.lyric_nodes = (0..self.lines.len())
            .filter_map(|i| el(&format!("lyric-{i}")).and_then(|e| e.dyn_into().ok()))
            .collect();
    }
    fn ensure_lyrics(&mut self) {
        if (self.player_page != 2 && !self.pager_dragging) || self.lyrics_mounted {
            return;
        }
        set_html(
            "lyric-lines",
            &render::lyrics(
                &self.lines,
                &self.translations,
                &self.word_lines,
                &self.data,
            ),
        );
        self.lyrics_mounted = true;
        self.cache_lyric_nodes();
        self.active_line = None;
        self.lyric_dirty = true;
        self.sync_lyrics();
    }
    fn cache_sheet_layers(&mut self) {
        let (width, height) = viewport();
        self.sheet_layers = (|| {
            Some(SheetLayers {
                player: el("player")?,
                surface: el("player-surface")?,
                content: find("#player .player-safe")?,
                cover: el("cover-art")?,
                artwork: if self.player_page == 2 {
                    find("#player .identity-art")?
                } else {
                    el("cover-art")?
                },
                mini: el("mini-player")?,
                dock: el("dock")?,
                nav: el("navigation")?,
                overlay: el("shared-artwork"),
                width,
                height,
            })
        })();
    }
    /// Route → dock slot. Settings sub-pages keep the settings slot selected so the
    /// indicator does not shuttle while the user walks deeper into a group.
    fn dock_index(&self) -> usize {
        match s(&self.data, "page") {
            "home" => 0,
            "collection" => 1,
            "mine" => 2,
            _ => 3,
        }
    }
    fn dock_slot(&self) -> String {
        let page = s(&self.data, "page");
        if matches!(
            page,
            "settings"
                | "appearance"
                | "quality"
                | "monet"
                | "monet-source"
                | "monet-style"
                | "monet-spec"
                | "storage"
                | "about"
                | "ui-customize"
                | "ui-dock"
                | "ui-player"
                | "ui-list"
                | "ui-motion"
                | "lyrics-style"
                | "lyric-type"
                | "lyric-effect"
        ) {
            "settings".into()
        } else {
            page.to_owned()
        }
    }
    /// Recompute the dock target after a repaint. The DOM was just replaced, so the
    /// indicator position is written immediately to avoid a one-frame flash.
    fn sync_dock(&mut self) {
        let index = self.dock_index() as f64;
        let slot = self.dock_slot();
        if !self.dock_ready {
            self.dock.place(index);
            self.dock_ready = true;
        } else if self.dock_page != slot {
            self.dock.to(index);
        }
        self.dock_page = slot;
        if let Some(nav) = el("navigation") {
            self.nav_nodes = nav
                .query_selector_all(".nav-item")
                .ok()
                .map(|nodes| {
                    (0..nodes.length())
                        .filter_map(|i| nodes.item(i)?.dyn_into().ok())
                        .collect()
                })
                .unwrap_or_default();
            self.nav_element = Some(nav.clone());
        }
        self.paint_dock();
    }
    fn paint_dock(&mut self) {
        if let Some(nav) = self.nav_element.as_ref() {
            style(&nav, "--nav-index", &format!("{:.5}", self.dock.position));
            style(&nav, "--nav-count", &format!("{}", self.dock_count.max(1)));
        }
        for (index, item) in self.nav_nodes.iter().enumerate() {
            let distance = (index as f64 - self.dock.position).abs();
            let proximity = (1.0 - distance).clamp(0.0, 1.0);
            style(&item, "--nav-p", &format!("{proximity:.4}"));
        }
    }
    fn advance_dock(&mut self, dt: f64) {
        if self.dock.settled() {
            return;
        }
        if self.reduced {
            self.dock.place(self.dock.target);
        } else {
            self.dock.sample(dt, motion::Feel::DOCK);
        }
        self.paint_dock();
    }
    fn advance_mini(&mut self, time: f64, dt: f64) {
        if self.player || !self.sheet_spring.settled() {
            return;
        }
        let target = if !self.player
            && self.mini_until > time
            && customization::get(&self.data, "compact") != "on"
        {
            1.0
        } else {
            0.0
        };
        self.mini.to(target);
        if self.mini.settled() && (self.mini.position - self.mini_written).abs() < 0.0005 {
            return;
        }
        let value = if self.reduced {
            self.mini.place(target);
            target
        } else {
            self.mini.sample(dt, motion::Feel::REVEAL)
        };
        let expanded = value > 0.25;
        if expanded != self.mini_expanded {
            if let Some(mini) = el("mini-player") {
                class(&mini, "expanded", expanded);
            }
            self.mini_expanded = expanded;
        }
        if (value - self.mini_written).abs() > 0.0004 {
            self.mini_written = value;
            self.paint_mini_pose();
        }
    }
    fn paint_mini_pose(&self) {
        let displacement = if shell::dock_visible(s(&self.data, "page")) {
            self.dock_extra * self.mini.position.clamp(0.0, 1.0)
        } else {
            0.0
        };
        if let Some(root) = el("app") {
            style(&root, "--dock-extra", &format!("{displacement:.3}px"));
        }
        if let Some(mini) = el("mini-player") {
            style(&mini, "--mini", &format!("{:.4}", self.mini.position));
            class(&mini, "expanded", self.mini_expanded);
            for (selector, available) in [
                (".mini-compact", !self.mini_expanded),
                (
                    ".mini-expanded",
                    self.mini_expanded && self.mini.position > 0.65,
                ),
            ] {
                if let Ok(Some(group)) = mini.query_selector(selector) {
                    let _ = group
                        .set_attribute("aria-hidden", if available { "false" } else { "true" });
                    if available {
                        let _ = group.remove_attribute("inert");
                    } else {
                        let _ = group.set_attribute("inert", "");
                    }
                }
            }
        }
        if let Some(actions) = el("float-actions") {
            style(
                &actions,
                "transform",
                &format!("translateY({}px)", -displacement),
            );
        }
    }
    fn paint_pager(&self) {
        if let Some(pages) = self.pager_layer.as_ref() {
            style(pages, "--page", &self.pager.position.to_string());
        }
        // A translated, clipped sibling can still rasterize its mask and every
        // blurred glyph. Keep layout/decoded nodes, but only paint panes actually
        // intersecting the viewport. Reveal the adjacent pane during the swipe.
        for (index, pane) in self.pager_panes.iter().enumerate() {
            let visibility = if (self.pager.position - index as f64).abs() < 0.9999
                || index == self.player_page
            {
                "visible"
            } else {
                "hidden"
            };
            if let Some(pane) = pane.dyn_ref::<HtmlElement>() {
                if pane
                    .style()
                    .get_property_value("visibility")
                    .unwrap_or_default()
                    != visibility
                {
                    let _ = pane.style().set_property("visibility", visibility);
                    // Skip descendant style/layout as well as rasterization while
                    // offscreen. The fixed flex pane still owns its full size.
                    let _ = pane.style().set_property("content-visibility", visibility);
                }
            }
        }
    }
    fn advance_pager(&mut self, dt: f64) {
        if self.player && !self.pager_dragging && !self.pager.settled() {
            if self.reduced {
                self.pager.place(self.player_page as f64);
            } else {
                self.pager.sample(dt, motion::Feel::PAGER);
            }
            self.paint_pager();
        }
    }
    fn show_player_page(&mut self) {
        self.pager_layer = el("player-pages");
        self.pager_panes = [".information-pane", ".cover-pane", "#lyric-viewport"]
            .iter()
            .filter_map(|selector| find(&format!("#player {selector}")))
            .collect();
        if self.reduced {
            self.pager.place(self.player_page as f64);
        }
        self.pager.to(self.player_page as f64);
        self.paint_pager();
        self.ensure_lyrics();
        if self.player_page == 2 && self.lyric_nodes_pending {
            self.cache_lyric_nodes();
        }
        self.fit_cover();
        if let Some(player) = el("player") {
            class(&player, "lyrics-page", self.player_page == 2);
        }
        for index in 0..3 {
            if let Some(dot) = el(&format!("page-dot-{index}")) {
                class(&dot, "selected", index == self.player_page);
            }
        }
    }
    /// The cover page keeps a square that fits the pane. Measuring the pane rather
    /// than the transformed player keeps the size stable during the open animation.
    fn fit_cover(&self) {
        let (pane, cover) = match (find(".cover-pane"), el("cover-art")) {
            (Some(pane), Some(cover)) => (pane, cover),
            _ => return,
        };
        let side = (pane.client_width() - 64)
            .min(pane.client_height() - 40)
            .clamp(0, 460);
        style(&cover, "width", &format!("{side}px"));
        style(&cover, "height", &format!("{side}px"));
    }
    /// Measure the shared element endpoints. The cover is measured after the player
    /// page has been laid out, so both ends come from real layout boxes; the radius
    /// comes from computed styles, which is why a circular cover morphs correctly.
    fn measure_morph(&mut self) -> Option<CoverMorph> {
        let mini = el("mini-player")?;
        self.sheet_bounds = rect(&mini);
        self.sheet_radius = corner_radius(&mini, self.sheet_bounds[2].min(self.sheet_bounds[3]));
        let mini_art = mini.query_selector(".mini-art").ok().flatten()?;
        let cover = if self.player_page == 2 {
            find("#player .identity-art")?
        } else {
            el("cover-art")?
        };
        let from = rect(&mini_art);
        let to = rect(&cover);
        if from[2] < 2.0 || from[3] < 2.0 || to[2] < 2.0 || to[3] < 2.0 {
            return None;
        }
        Some(CoverMorph {
            from,
            to,
            radius_from: corner_radius(&mini_art, from[2].min(from[3])),
            radius_to: corner_radius(&cover, to[2].min(to[3])),
        })
    }
    fn mount_morph(&self) {
        if let Some(old) = el("shared-artwork") {
            old.remove();
        }
        let Some(mini_art) =
            el("mini-player").and_then(|mini| mini.query_selector(".mini-art").ok().flatten())
        else {
            return;
        };
        let Ok(overlay) = doc().create_element("div") else {
            return;
        };
        overlay.set_id("shared-artwork");
        if let Some(morph) = self.morph {
            style(&overlay, "width", &format!("{}px", morph.to[2]));
            style(&overlay, "height", &format!("{}px", morph.to[3]));
        }
        if let Ok(copy) = mini_art.clone_node_with_deep(true) {
            let _ = overlay.append_child(&copy);
            if let Some(inner) = overlay.first_element_child() {
                style(&inner, "border-radius", "0");
            }
        }
        if let Some(root) = el("app") {
            let _ = root.append_child(&overlay);
        }
    }
    fn unmount_morph(&mut self) {
        if let Some(overlay) = el("shared-artwork") {
            overlay.remove();
        }
        self.morph = None;
        if let Some(mini) = el("mini-player") {
            class(&mini, "sharing-artwork", false);
        }
        if let Some(layers) = self.sheet_layers.as_mut() {
            layers.overlay = None;
        }
    }
    fn begin_sheet(&mut self, open: bool) {
        self.player = open;
        if !self.reduced && !self.sheet_spring.settled() {
            self.sheet_spring.to(if open { 1.0 } else { 0.0 });
            self.apply_sheet(self.sheet_spring.position);
            return;
        }
        if let Some(content) = find("#player .player-safe") {
            style(&content, "transform", "none");
        }
        if !open {
            self.mini_until = 0.0;
            self.mini.place(0.0);
            self.mini.to(0.0);
            self.mini_written = 0.0;
            self.mini_expanded = false;
            if let Some(mini) = el("mini-player") {
                style(&mini, "--mini", "0");
                class(&mini, "expanded", false);
            }
            self.paint_mini_pose();
        }
        self.morph = if self.reduced {
            None
        } else {
            self.measure_morph()
        };
        self.cache_sheet_layers();
        if self.reduced {
            let target = if open { 1.0 } else { 0.0 };
            self.sheet_spring.place(target);
            self.sheet_spring.to(target);
            self.apply_sheet(target);
            return;
        }
        if open {
            self.sheet_spring.place(0.0);
        }
        if self.morph.is_some() {
            self.mount_morph();
            if let Some(layers) = self.sheet_layers.as_mut() {
                layers.overlay = el("shared-artwork");
            }
        }
        self.sheet_spring.to(if open { 1.0 } else { 0.0 });
        // Paint the first morph frame now so no unstyled overlay is ever shown.
        self.apply_sheet(self.sheet_spring.position);
    }
    fn open_player(&mut self) {
        if self.player {
            return;
        }
        self.animation_vote = true;
        post(json!({"type":"animation_rate","active":true}));
        if !self.reduced && !self.sheet_spring.settled() {
            self.player = true;
            self.sheet_spring.to(1.0);
            if let Some(player) = el("player") {
                class(&player, "visible", true);
                let _ = player.set_attribute("aria-hidden", "false");
                let _ = player.remove_attribute("inert");
            }
            self.apply_sheet(self.sheet_spring.position);
            return;
        }
        self.player_page = 1;
        self.cover_flip_until = 0.0;
        if let Some(cover) = el("cover-art") {
            style(&cover, "transform", "none");
        }
        self.pager.place(1.0);
        self.pager.to(1.0);
        self.pager_dragging = false;
        self.paint_player();
        if let Some(player) = el("player") {
            class(&player, "mounted", true);
            class(&player, "visible", true);
            let _ = player.set_attribute("aria-hidden", "false");
            let _ = player.remove_attribute("inert");
            let _ = player.get_bounding_client_rect();
        }
        self.begin_sheet(true);
    }
    fn close_player(&mut self) {
        if !self.player {
            return;
        }
        self.animation_vote = true;
        post(json!({"type":"animation_rate","active":true}));
        self.pager_dragging = false;
        self.pointer = None;
        if let Some(player) = el("player") {
            class(&player, "visible", false);
            let _ = player.set_attribute("aria-hidden", "true");
            let _ = player.set_attribute("inert", "");
        }
        self.begin_sheet(false);
    }
    /// Apply the player sheet and the shared artwork for a progress value in `0..=1`.
    /// `0` is exactly the mini player card and `1` the full window, so every frame is
    /// a continuous morph where the shared cover's corner radius is interpolated in
    /// screen pixels instead of being switched by a class at the end.
    fn apply_sheet(&mut self, p: f64) {
        let Some(layers) = self.sheet_layers.clone() else {
            return;
        };
        let p = p.clamp(0.0, 1.0);
        let q = 1.0 - p;
        let blend = motion::sheet_blend(p);
        let [x, y, mw, mh] = self.sheet_bounds;
        let sx = (mw + (layers.width - mw) * p) / layers.width.max(1.0);
        let sy = (mh + (layers.height - mh) * p) / layers.height.max(1.0);
        style(
            &layers.surface,
            "transform",
            &format!(
                "translate3d({:.3}px,{:.3}px,0) scale({:.6},{:.6})",
                x * q,
                y * q,
                sx,
                sy
            ),
        );
        let r = self.sheet_radius * q * q;
        style(
            &layers.surface,
            "border-radius",
            &format!("{}px / {}px", r / sx.max(0.001), r / sy.max(0.001)),
        );
        style(&layers.surface, "opacity", &blend.background.to_string());
        style(&layers.content, "opacity", &blend.content.to_string());
        style(
            &layers.content,
            "transform",
            &format!("translate3d(0,{:.3}px,0)", 18.0 * q),
        );
        style(&layers.mini, "opacity", &blend.mini.to_string());
        style(&layers.nav, "opacity", &blend.mini.to_string());
        style(
            &layers.dock,
            "z-index",
            if blend.mini_above { "6" } else { "2" },
        );
        if !self.player && p <= 0.2 {
            let _ = layers.dock.remove_attribute("inert");
        } else {
            let _ = layers.dock.set_attribute("inert", "");
        }
        style(
            &layers.player,
            "pointer-events",
            if self.player || p > 0.2 {
                "auto"
            } else {
                "none"
            },
        );
        class(
            &layers.mini,
            "sharing-artwork",
            self.morph.is_some() && p > 0.0 && p < 1.0,
        );
        style(
            &layers.nav,
            "transform",
            &format!("translate3d(0,{}px,0)", 72.0 * (1.0 - blend.mini)),
        );
        class(
            &layers.player,
            "mounted",
            motion::sheet_mounted(self.player, p, self.sheet_spring.settled()),
        );
        let Some(morph) = self.morph else {
            // Reduced motion (or an unmeasurable layout) skips the shared morph but
            // must still reveal the real cover and unmount the closed sheet.
            style(&layers.cover, "opacity", "1");
            style(&layers.artwork, "opacity", "1");
            return;
        };
        let done = if self.player {
            p >= 0.9995
        } else {
            p <= 0.0005
        };
        style(&layers.cover, "opacity", if done { "1" } else { "0" });
        style(&layers.artwork, "opacity", if done { "1" } else { "0" });
        if done {
            self.unmount_morph();
            return;
        }
        let geometry = motion::smooth(p);
        let lift = p * 0.62 + geometry * 0.38;
        let left = motion::lerp(morph.from[0], morph.to[0], geometry);
        let top = motion::lerp(morph.from[1], morph.to[1], lift);
        let side = motion::lerp(morph.from[2], morph.to[2], geometry);
        let scale = (side / morph.to[2].max(1.0)).max(0.02);
        // The overlay is scaled from a fixed box, so the CSS radius is divided by the
        // scale to keep the *rendered* radius exactly on the interpolated value.
        let radius = motion::lerp(morph.radius_from, morph.radius_to, geometry) / scale;
        if let Some(overlay) = layers.overlay {
            style(
                &overlay,
                "transform",
                &format!("translate3d({left:.3}px,{top:.3}px,0) scale({scale:.6})"),
            );
            style(&overlay, "border-radius", &format!("{radius:.2}px"));
            style(&overlay, "opacity", "1");
        }
    }
    fn advance_sheet(&mut self, dt: f64) {
        if self.sheet_spring.settled() && self.morph.is_none() {
            return;
        }
        let p = self.sheet_spring.sample(dt, motion::Feel::SHEET);
        self.apply_sheet(p);
    }
    fn paint_overlay(&mut self) {
        let open = b(&self.data, "qrOpen") || !self.sheet.is_empty();
        let entering = open && !self.overlay_open;
        if !open {
            if self.overlay_open {
                if let Some(layers) = self.overlay_layers.as_ref() {
                    let _ = layers.sheet.set_attribute("inert", "");
                }
                self.overlay_closing = true;
                self.sheet_drag = None;
                self.overlay_spring.to(0.0);
                if self.reduced {
                    self.overlay_spring.place(0.0);
                    self.advance_shell(0.0);
                }
            }
            return;
        }
        self.overlay_open = true;
        if let Some(layers) = self.overlay_layers.as_ref() {
            let _ = layers.sheet.remove_attribute("inert");
        }
        self.overlay_closing = false;
        self.overlay_spring.to(1.0);
        if entering {
            self.overlay_spring
                .place(if self.reduced { 1.0 } else { 0.0 });
        }
        let next = render::overlay(
            &self.sheet,
            &self.data,
            &self.live,
            !self.sheet_stack.is_empty(),
        );
        let geometry_changed = entering
            || match (&self.painted_overlay, &next) {
                (Some(a), Some(b)) => {
                    a.class != b.class
                        || a.header != b.header
                        || a.items != b.items
                        || a.message != b.message
                }
                _ => true,
            };
        match (&self.painted_overlay, &next) {
            (Some(old), Some(new))
                if old.class == new.class || old.class != "qr-sheet" && new.class != "qr-sheet" =>
            {
                if old.class != new.class {
                    if let Some(sheet) = el("sheet") {
                        let _ = sheet.set_attribute("class", &format!("sheet {}", new.class));
                    }
                }
                if old.title != new.title {
                    if let Some(sheet) = el("sheet") {
                        class(&sheet, "enter", false);
                        let _ = sheet.set_attribute("aria-label", &new.title);
                    }
                }
                if old.header != new.header {
                    set_html("sheet-header", &new.header);
                }
                if old.items != new.items {
                    set_html("sheet-items", &new.items);
                }
                if old.message != new.message {
                    if let Some(message) = el("sheet-message") {
                        message.set_text_content(Some(&new.message));
                    }
                }
            }
            (_, Some(new)) => set_html("overlays", &new.markup(false)),
            (Some(_), None) => set_html("overlays", ""),
            (None, None) => {}
        }
        self.painted_overlay = next;
        if geometry_changed {
            self.cache_overlay_layers();
        }
        self.apply_overlay();
    }
    fn paint_progress(&self) {
        if let Some(caption) = el("mini-caption") {
            let at = n(&self.live, "position");
            let line = self
                .mini_lines
                .iter()
                .rev()
                .find(|(t, _)| *t <= at)
                .map(|(_, s)| s.as_str())
                .unwrap_or(s(&self.live, "artist"));
            if caption.text_content().as_deref() != Some(line) {
                caption.set_text_content(Some(line));
            }
        }
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
        if let Some(mini) = el("mini-player") {
            style(&mini, "--progress", &format!("{}%", fraction * 100.0));
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
        if !self.player
            || self.lines.is_empty()
            || !self.lyrics_mounted
            || (self.player_page != 2 && !self.pager_dragging && self.pager.settled())
        {
            return;
        }
        let position = n(&self.live, "position")
            + if b(&self.live, "playing") {
                ((now() - self.clock_at).clamp(0.0, 250.0)
                    * self.live["speed"].as_f64().unwrap_or(1.0)) as i32
            } else {
                0
            };
        let next = self
            .lines
            .partition_point(|(time, _)| *time <= position)
            .checked_sub(1);
        if let Some(index) = next {
            if let (Some(words), Some(nodes), Some(fills)) = (
                self.word_lines.get(index),
                self.word_nodes.get(index),
                self.word_fills.get_mut(index),
            ) {
                for (j, ((word, node), previous)) in words.iter().zip(nodes).zip(fills).enumerate()
                {
                    let fill = karaoke::fill(word, position);
                    if *previous != fill {
                        let _ = node.style().set_property(
                            "--word-fill",
                            &format!("{:.2}%", fill as f64 * 0.116 - 8.0),
                        );
                        *previous = fill;
                        if let Some(glyphs) = self.glyph_nodes.get(index).and_then(|row| row.get(j))
                        {
                            for (k, glyph) in glyphs.iter().enumerate() {
                                if let Some(previous) = self
                                    .glyph_fills
                                    .get_mut(index)
                                    .and_then(|row| row.get_mut(j))
                                    .and_then(|word| word.get_mut(k))
                                {
                                    if *previous == fill {
                                        continue;
                                    }
                                    *previous = fill;
                                }
                                let mut effect = karaoke::effect(
                                    fill as f64 / 1000.0,
                                    k,
                                    glyphs.len(),
                                    if karaoke::eligible(word) {
                                        word.end - word.start
                                    } else {
                                        0
                                    },
                                    self.reduced,
                                );
                                let font = customization::get(&self.data, "lyricSize")
                                    .parse::<f64>()
                                    .unwrap_or(24.0);
                                let bounds = self.glyph_geometry.get(index).and_then(|r| r.get(j));
                                let reveal = bounds
                                    .and_then(|g| {
                                        g.get(k).map(|&(left, right)| {
                                            karaoke::spatial_reveal(
                                                fill as f64 / 1000.0,
                                                left,
                                                right,
                                                g.last().map(|g| g.1).unwrap_or(1.0),
                                                font,
                                                effect.scale,
                                            )
                                        })
                                    })
                                    .unwrap_or(
                                        (fill as f64 / 1000.0 * glyphs.len() as f64 - k as f64)
                                            .clamp(0.0, 1.0),
                                    );
                                if !self.reduced {
                                    effect.lift = -(4.0 / 72.0)
                                        * (1.0 - (reveal * std::f64::consts::PI).cos())
                                        / 2.0;
                                }
                                let _ = glyph.style().set_property(
                                    "transform",
                                    &format!(
                                        "translateY(calc(var(--lyric-size) * {:.5})) scale({:.4})",
                                        effect.lift, effect.scale
                                    ),
                                );
                                let _ = glyph
                                    .style()
                                    .set_property("--glyph-glow", &format!("{:.4}", effect.glow));
                                let _ = glyph.style().set_property(
                                    "--glyph-radius",
                                    &format!("{}px", effect.radius * font / 72.0),
                                );
                                let _ = glyph.style().set_property(
                                    "--glyph-fill",
                                    &format!("{:.2}%", reveal * 116.0 - 8.0),
                                );
                            }
                        }
                    }
                }
            }
        }
        if next == self.active_line {
            return;
        }
        self.active_line = next;
        self.lyric_dirty = true;
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
        self.paint_header_scroll();
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
            let near_bottom = main.scroll_height() - top - main.client_height() < 520;
            if near_bottom && self.render_pending() {
                // First show the rows already fetched, then ask for the next page.
                self.extend_list();
                return;
            }
            if near_bottom
                && page == "songs"
                && !self.detail_artists
                && b(&self.data, "detailHasMore")
                && !b(&self.data, "detailLoadingMore")
                && !b(&self.data, "detailLoading")
                && now() - self.last_load > 700.0
            {
                self.last_load = now();
                post(json!({"type":"load_more_tracks"}));
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
    /// Data source of the currently rendered long list; empty when the page has none.
    fn list_source(&self) -> &'static str {
        match s(&self.data, "page") {
            "songs" if !self.detail_artists => "tracks",
            "collection" => "likes",
            "recent" => "recent",
            "home" => "discovery",
            "search" => "search",
            _ => "",
        }
    }
    /// True when more fetched songs are available than the current render batch shows.
    fn render_pending(&self) -> bool {
        let id = self.list_source();
        if id.is_empty() {
            return false;
        }
        let shown = *self.limits.get(&self.route_key()).unwrap_or(&100) as usize;
        self.source(id).len() > shown
    }
    fn extend_list(&mut self) {
        self.save_scroll();
        let route = self.route_key();
        let next = self.limits.get(&route).copied().unwrap_or(100) + 100;
        self.limits.insert(route, next);
        self.paint_content();
    }
    fn frame(&mut self, time: f64) {
        let dt = if self.last_frame <= 0.0 {
            1.0 / 60.0
        } else {
            ((time - self.last_frame) / 1000.0).clamp(0.0, 0.1)
        };
        self.last_frame = time;
        self.advance_dock(dt);
        self.advance_mini(time, dt);
        self.advance_sheet(dt);
        self.advance_pager(dt);
        self.advance_shell(dt);
        let animating = !self.dock.settled()
            || !self.mini.settled()
            || !self.sheet_spring.settled()
            || self.morph.is_some()
            || !self.route_spring.settled()
            || !self.overlay_spring.settled()
            || self.sheet_drag.is_some()
            || (self.player && (!self.pager.settled() || self.pager_dragging))
            || (self.player && self.player_page == 2 && b(&self.live, "playing"));
        if animating != self.animation_vote {
            self.animation_vote = animating;
            post(json!({"type":"animation_rate","active":animating}));
        }
        if self.player
            && !self.reduced
            && b(&self.live, "playing")
            && customization::get(&self.data, "background") == "flow"
            && customization::get(&self.data, "blur") == "on"
        {
            self.ambient_phase += dt / 14.0;
        }
        // Slow ambient visuals are Rust sampled at 30Hz, independent of lyric vsync.
        if self.player && (time / 33.0) as i32 != self.effect_stamp {
            self.effect_stamp = (time / 33.0) as i32;
            let phase = self.ambient_phase;
            let transform = if customization::get(&self.data, "background") == "flow" {
                format!(
                    "translate3d({:.2}px,{:.2}px,0) scale(1.3) rotate({:.2}deg)",
                    phase.sin() * 24.0,
                    (phase * 0.7).cos() * 18.0,
                    phase.sin() * 12.0
                )
            } else {
                "scale(1.15)".into()
            };
            if transform != self.wash_transform {
                if let Ok(Some(wash)) = doc().query_selector(".player-wash") {
                    style(&wash, "transform", &transform);
                }
                self.wash_transform = transform;
            }
        }
        if self.player
            && self.sheet_spring.settled()
            && self.morph.is_none()
            && self.cover_flip_until > 0.0
        {
            if let Some(cover) = el("cover-art") {
                let p = ((self.cover_flip_until - time) / 720.0).clamp(0.0, 1.0);
                let wave = if self.reduced || customization::get(&self.data, "coverFlip") == "off" {
                    0.0
                } else {
                    (p * std::f64::consts::PI).sin()
                };
                style(
                    &cover,
                    "transform",
                    &format!(
                        "perspective(1000px) rotateY({:.2}deg) scale({:.4})",
                        wave * 15.0,
                        1.0 - wave * 0.035
                    ),
                );
                if p == 0.0 {
                    self.cover_flip_until = 0.0;
                }
            }
        }
        if !self.player || self.player_page != 2 {
            return;
        }
        self.sync_lyrics();
        let Some(viewport) = self.lyric_viewport.clone() else {
            return;
        };
        let height = viewport.client_height();
        let browsing = time <= self.browse_until;
        if browsing != self.browsing_last {
            class(&viewport, "browsing", browsing);
            self.browsing_last = browsing;
            self.lyric_dirty = true;
            for i in self.row_motion.keys() {
                if let Some(row) = self.lyric_nodes.get(*i) {
                    let _ = row.style().remove_property("translate");
                }
            }
            self.row_motion.clear();
        }
        if browsing {
            return;
        }
        if height != self.lyric_height {
            self.lyric_height = height;
            self.lyric_dirty = true;
            if let Some(lines) = el("lyric-lines") {
                style(
                    &lines,
                    "padding-top",
                    &format!("{}px", height as f64 * 0.36),
                );
                style(
                    &lines,
                    "padding-bottom",
                    &format!("{}px", height as f64 * 0.64),
                );
            }
        }
        let Some(index) = self.active_line else {
            return;
        };
        if self.lyric_dirty {
            self.lyric_dirty = false;
            if let Some(focus) = self.lyric_nodes.get(index) {
                let old = viewport.scroll_top() as f64;
                let target = (focus.offset_top() as f64 - height as f64 * 0.36)
                    .max(0.0)
                    .round();
                let travel = target - old;
                if travel.abs() > 0.5 {
                    let measured: Vec<(usize, f64, f64)> = self
                        .lyric_nodes
                        .iter()
                        .enumerate()
                        .map(|(i, e)| (i, e.offset_top() as f64, e.offset_height() as f64))
                        .collect();
                    let visible: Vec<usize> = measured
                        .iter()
                        .filter(|(_, y, h)| *y + *h > old && *y < old + height as f64)
                        .map(|(i, _, _)| *i)
                        .collect();
                    let animate = !self.reduced && travel.abs() < height as f64 * 1.5;
                    viewport.set_scroll_top(target as i32);
                    for (i, y, h) in measured {
                        if y + h < target - 120.0 || y > target + height as f64 + 120.0 {
                            continue;
                        }
                        let row = self.row_motion.entry(i).or_default();
                        if animate {
                            let rank =
                                visible
                                    .iter()
                                    .position(|v| *v == i)
                                    .unwrap_or(if i < index {
                                        0
                                    } else {
                                        visible.len().saturating_sub(1)
                                    });
                            row.place(travel, lyric_motion::delay(rank, visible.len()));
                        } else {
                            *row = Default::default();
                        }
                    }
                }
            }
        }
        let gap = self
            .lines
            .get(index + 1)
            .map(|l| l.0 - self.lines[index].0)
            .unwrap_or(1000);
        self.row_motion.retain(|i, row| {
            let offset = if self.reduced {
                0.0
            } else {
                row.sample(dt, lyric_motion::stiffness(gap))
            };
            if let Some(element) = self.lyric_nodes.get(*i) {
                if offset.abs() < 0.01 {
                    let _ = element.style().remove_property("translate");
                    let _ = element.style().remove_property("will-change");
                } else {
                    style(element, "will-change", "translate,transform");
                    style(element, "translate", &format!("0 {offset:.3}px"));
                }
            }
            offset.abs() >= 0.01
        });
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
