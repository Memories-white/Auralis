//! Bounded private lyric documents, independent of MediaPlayer or account tokens.
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
static LOCK: Mutex<()> = Mutex::new(());
const MAX_FILE: usize = 512 * 1024;
const MAX_BYTES: u64 = 8 * 1024 * 1024;
const MAX_FILES: usize = 64;
fn key(mid: &str) -> String {
    let h = mid.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    });
    format!("auralis-lyric-{h:016x}.json")
}
fn file(root: &str, mid: &str) -> Option<PathBuf> {
    if root.is_empty() || mid.is_empty() || mid.len() > 256 {
        return None;
    }
    Some(Path::new(root).join(key(mid)))
}
fn clean(data: &Value) -> Option<Value> {
    let lyric = data["lyric"].as_str()?;
    if lyric.trim().is_empty() || lyric.len() > 384 * 1024 {
        return None;
    }
    let translation = data["translation"].as_str().unwrap_or("");
    if translation.len() > 384 * 1024 {
        return None;
    }
    let words = data["wordLyrics"].as_array().cloned().unwrap_or_default();
    let value = json!({"lyric":lyric,"translation":translation,"wordLyrics":words});
    (value.to_string().len() <= MAX_FILE).then_some(value)
}
pub fn read(root: &str, mid: &str) -> Option<Value> {
    let _guard = LOCK.lock().ok()?;
    read_unlocked(root, mid)
}
fn read_unlocked(root: &str, mid: &str) -> Option<Value> {
    let path = file(root, mid)?;
    let meta = fs::symlink_metadata(&path).ok()?;
    if !meta.is_file() || meta.len() > MAX_FILE as u64 {
        return None;
    }
    let value: Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    if value["version"] != 1 || value["mid"] != mid {
        return None;
    }
    clean(&value["data"])
}
pub fn write(root: &str, mid: &str, data: &Value) -> bool {
    let Ok(_guard) = LOCK.lock() else {
        return false;
    };
    let Some(path) = file(root, mid) else {
        return false;
    };
    let Some(data) = clean(data) else {
        return false;
    };
    let envelope=json!({"version":1,"mid":mid,"savedAt":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),"data":data}).to_string();
    if envelope.len() > MAX_FILE {
        return false;
    }
    if fs::create_dir_all(root).is_err() {
        return false;
    }
    let temp = path.with_extension("json.tmp");
    if fs::write(&temp, envelope).is_err() {
        return false;
    }
    if fs::rename(&temp, &path).is_err() {
        let _ = fs::remove_file(&temp);
        return false;
    }
    prune(root, MAX_BYTES, MAX_FILES);
    true
}
fn prune(root: &str, limit: u64, count: usize) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if !name.starts_with("auralis-lyric-") || !name.ends_with(".json") {
                return None;
            }
            let m = fs::symlink_metadata(e.path()).ok()?;
            m.is_file()
                .then_some((m.modified().unwrap_or(UNIX_EPOCH), m.len(), e.path()))
        })
        .collect();
    files.sort_by_key(|f| f.0);
    let mut total: u64 = files.iter().map(|f| f.1).sum();
    let mut remaining = files.len();
    for (_, len, path) in files {
        if total <= limit && remaining <= count {
            break;
        }
        if fs::remove_file(path).is_ok() {
            total = total.saturating_sub(len);
            remaining -= 1;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persistence_survives_reopen_and_rejects_invalid_or_secret_fields() {
        let dir = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!(
                "auralis-lyric-test-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        let root = dir.to_str().unwrap();
        let d = json!({"lyric":"[00:01.000]A quiet tune","translation":"","wordLyrics":[{"time":1000,"text":"A quiet tune","words":[]}],"session":{"musicKey":"must-not-persist"}});
        assert!(write(root, "test-track", &d));
        let restored = read(root, "test-track").unwrap();
        assert!(restored["lyric"].as_str().unwrap().contains("quiet"));
        assert!(!restored.to_string().contains("must-not-persist"));
        assert!(read(root, "other").is_none());
        assert!(!write(root, "test-track", &json!({"lyric":""})));
        assert!(read(root, "test-track").is_some());
        fs::write(dir.join(key("bad")), "invalid").unwrap();
        assert!(read(root, "bad").is_none());
        fs::write(dir.join("keep.txt"), "keep").unwrap();
        prune(root, 0, 0);
        assert!(dir.join("keep.txt").exists());
        assert!(read(root, "test-track").is_none());
        let _ = fs::remove_dir_all(dir);
    }
}
