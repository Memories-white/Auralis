//! Retain decoded artwork across Rust DOM renders. No new image request is issued
//! until a retained node has been considered; detached images have a bounded LRU.
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Blob, Element, HtmlImageElement, Url, XmlHttpRequest, XmlHttpRequestResponseType};

#[path = "../../core/src/artwork.rs"]
#[allow(dead_code)]
mod policy;

#[derive(Default)]
struct Images {
    detached: Vec<HtmlImageElement>,
    markup: HashMap<String, String>,
}
thread_local! { static IMAGES: RefCell<Images> = RefCell::new(Images::default()); }

#[derive(Default)]
struct Resources {
    ready: VecDeque<(String, String, usize)>,
    pending: HashSet<String>,
    bytes: usize,
}
thread_local! { static RESOURCES: RefCell<Resources> = RefCell::new(Resources::default()); }

fn hydrate(image: &HtmlImageElement, key: &str) {
    let cached = RESOURCES.with(|state| {
        let mut state = state.borrow_mut();
        let index = state.ready.iter().position(|entry| entry.0 == key)?;
        let entry = state.ready.remove(index)?;
        let url = entry.1.clone();
        state.ready.push_back(entry);
        Some(url)
    });
    if let Some(url) = cached {
        image.set_src(&url);
        return;
    }
    if !RESOURCES.with(|state| state.borrow_mut().pending.insert(key.into())) {
        return;
    }
    let Ok(xhr) = XmlHttpRequest::new() else {
        RESOURCES.with(|s| s.borrow_mut().pending.remove(key));
        return;
    };
    if xhr.open_with_async("GET", key, true).is_err() {
        RESOURCES.with(|s| s.borrow_mut().pending.remove(key));
        return;
    }
    xhr.set_response_type(XmlHttpRequestResponseType::Blob);
    xhr.set_timeout(20000);
    let request = xhr.clone();
    let key = key.to_owned();
    let complete = Closure::once_into_js(move || {
        RESOURCES.with(|state| {
            state.borrow_mut().pending.remove(&key);
        });
        if request.status().ok() != Some(200) {
            return;
        }
        let Some(blob) = request
            .response()
            .ok()
            .and_then(|v| v.dyn_into::<Blob>().ok())
        else {
            return;
        };
        if blob.size() > 1024.0 * 1024.0 {
            return;
        }
        let Ok(url) = Url::create_object_url_with_blob(&blob) else {
            return;
        };
        RESOURCES.with(|state| {
            let mut state = state.borrow_mut();
            state.bytes += blob.size() as usize;
            state
                .ready
                .push_back((key.clone(), url.clone(), blob.size() as usize));
            while state.bytes > 8 * 1024 * 1024 || state.ready.len() > 160 {
                // Keep the current song usable by the shared-artwork clone even
                // after browsing more images than fit in the resource LRU.
                let pinned=web_sys::window().and_then(|w|w.document()).and_then(|d|d.query_selector_all("#mini-player img[data-art-src], #player img[data-art-src], #shared-artwork img[data-art-src]").ok()).map(|nodes|(0..nodes.length()).filter_map(|i|nodes.item(i)?.dyn_into::<Element>().ok()?.get_attribute("data-art-src")).collect::<HashSet<_>>()).unwrap_or_default();
                let Some(index)=state.ready.iter().position(|entry| !pinned.contains(&entry.0))else{break;};
                if let Some((_, old, size)) = state.ready.remove(index) {
                    state.bytes -= size;
                    let _ = Url::revoke_object_url(&old);
                }
            }
        });
        if let Some(root) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            for image in images(&root) {
                if image.get_attribute("data-art-src").as_deref() == Some(&key) {
                    image.set_src(&url);
                }
            }
        }
    });
    xhr.set_onloadend(Some(complete.unchecked_ref()));
    let _ = xhr.send();
}

pub fn source(url: &str, avatar: bool) -> String {
    let identity = policy::identity(url, avatar);
    format!(
        "https://auralis.local/artwork?avatar={avatar}&url={}",
        js_sys::encode_uri_component(&identity)
            .as_string()
            .unwrap_or_default()
    )
}
pub fn invalidate_markup(id: &str) {
    IMAGES.with(|state| {
        state.borrow_mut().markup.remove(id);
    });
}
fn images(root: &Element) -> Vec<HtmlImageElement> {
    root.query_selector_all("img[data-art-src]")
        .ok()
        .map(|nodes| {
            (0..nodes.length())
                .filter_map(|i| nodes.item(i)?.dyn_into().ok())
                .collect()
        })
        .unwrap_or_default()
}
pub fn replace(root: &Element, html: &str) {
    IMAGES.with(|state| {
        let mut state = state.borrow_mut();
        if state.markup.get(&root.id()).is_some_and(|old| old == html)
            && !images(root)
                .iter()
                .any(|image| image.complete() && image.natural_width() == 0)
        {
            return;
        }
        let document = root.owner_document().unwrap();
        let staging = document.create_element("div").unwrap();
        staging.set_inner_html(html); // data-art-src is inert, unlike src.
        for image in images(root) {
            if image.complete() && image.natural_width() > 0 {
                state.detached.push(image);
            }
        }
        root.set_inner_html("");
        for pending in images(&staging) {
            let key = pending.get_attribute("data-art-src").unwrap_or_default();
            if let Some(index) = state.detached.iter().position(|image| {
                !image.is_connected()
                    && image.get_attribute("data-art-src").as_deref() == Some(&key)
            }) {
                let retained = state.detached.remove(index);
                if let Some(parent) = pending.parent_node() {
                    let _ = parent.replace_child(&retained, &pending);
                }
            } else {
                hydrate(&pending, &key);
            }
        }
        while let Some(child) = staging.first_child() {
            let _ = root.append_child(&child);
        }
        let mut bytes: u64 = state
            .detached
            .iter()
            .map(|img| img.natural_width() as u64 * img.natural_height() as u64 * 4)
            .sum();
        while state.detached.len() > 64 || bytes > 16 * 1024 * 1024 {
            let removed = state.detached.remove(0);
            bytes = bytes.saturating_sub(
                removed.natural_width() as u64 * removed.natural_height() as u64 * 4,
            );
        }
        state.markup.insert(root.id(), html.into());
    });
}
