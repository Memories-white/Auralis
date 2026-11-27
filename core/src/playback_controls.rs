//! Shared control policy; Android executes the returned media operations only.
use serde_json::{json, Value};
pub fn audio(input: &Value) -> Result<Value, String> {
    let speed = input["speed"]
        .as_f64()
        .filter(|v| v.is_finite() && (0.25..=2.0).contains(v))
        .ok_or("播放速度超出范围")?;
    let speed = (speed * 20.0).round() / 20.0;
    let frames: Vec<Value> = (0..=20)
        .map(|i| {
            let t = i as f64 / 20.0;
            json!({"at":i*16,"value":t*t*(3.0-2.0*t)})
        })
        .collect();
    Ok(json!({"speed":speed,"fade":input["fade"].as_bool().unwrap_or(false),"frames":frames}))
}
pub fn enqueue(input: &Value) -> Result<Value, String> {
    let mut queue = input["queue"].as_array().cloned().ok_or("队列为空")?;
    let index = input["index"]
        .as_u64()
        .map(|v| v as usize)
        .filter(|i| *i < queue.len())
        .ok_or("当前歌曲已变更")?;
    let song = &input["song"];
    if song["mid"].as_str().unwrap_or("").is_empty() || queue.len() >= 10000 {
        return Err("无法加入队列".into());
    }
    let at = if input["next"].as_bool().unwrap_or(false) {
        index + 1
    } else {
        queue.len()
    };
    queue.insert(at, song.clone());
    Ok(json!({"queue":queue,"index":index}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn speed_and_fade_plan_are_bounded() {
        let p = audio(&json!({"speed":1.037,"fade":true})).unwrap();
        assert_eq!(p["speed"], 1.05);
        let f = p["frames"].as_array().unwrap();
        assert_eq!(f[0]["value"], 0.0);
        assert_eq!(f[20]["value"], 1.0);
        assert_eq!(f[20]["at"], 320);
        assert!(f
            .windows(2)
            .all(|v| v[0]["value"].as_f64() < v[1]["value"].as_f64()));
        assert!(audio(&json!({"speed":10})).is_err());
    }
    #[test]
    fn enqueue_preserves_current_and_next_or_tail() {
        let q = json!({"queue":[{"mid":"a"},{"mid":"b"}],"song":{"mid":"x"},"index":0,"next":true});
        assert_eq!(enqueue(&q).unwrap()["queue"][1]["mid"], "x");
        let mut tail = q.clone();
        tail["next"] = json!(false);
        assert_eq!(enqueue(&tail).unwrap()["queue"][2]["mid"], "x");
        tail["index"] = json!(9);
        assert!(enqueue(&tail).is_err());
    }
}
