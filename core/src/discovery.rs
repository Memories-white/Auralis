use super::*;
use std::collections::HashSet;

// Only personalized song shelves contain song IDs. Playlist and daily cards must
// never be passed to TrackInfo as though their IDs were song IDs.
fn song_ids(data: &Value) -> Vec<u64> {
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    for shelf in data["v_shelf"].as_array().into_iter().flatten() {
        let template = field(shelf, "title_template");
        let title = format!("{template}{}", field(shelf, "title"));
        if field(shelf, "title_content").is_empty()
            || !title.contains('听')
            || !(title.contains("也在听") || title.contains("喜欢"))
        {
            continue;
        }
        for niche in shelf["v_niche"].as_array().into_iter().flatten() {
            for card in niche["v_card"].as_array().into_iter().flatten() {
                if let Some(id) = number(&card["id"]).filter(|id| *id > 0) {
                    if seen.insert(id as u64) {
                        ids.push(id as u64);
                    }
                }
            }
        }
    }
    ids
}
fn filtered_tracks(
    tracks: &Value,
    excluded: &mut HashSet<String>,
    excluded_ids: &HashSet<u64>,
) -> Vec<Value> {
    tracks
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(normalize_song)
        .filter(|song| {
            let mid = field(song, "mid").to_owned();
            !excluded_ids.contains(&(number(&song["id"]).unwrap_or(0) as u64))
                && excluded.insert(mid)
        })
        .collect()
}
pub fn recommend(input: &Value) -> Result<Value, String> {
    let session = &input["session"];
    let mut excluded: HashSet<String> = input["exclude"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    let excluded_ids: HashSet<u64> = input["excludeIds"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(number)
        .map(|n| n as u64)
        .collect();
    let start = input["page"].as_u64().unwrap_or(1).max(1);
    let mut next = start;
    let mut songs = Vec::new();
    for page in start..start + 3 {
        let root = account_request(
            session,
            json!({"feed":{"module":"music.recommend.RecommendFeed","method":"get_recommend_feed","param":{"direction":if page==1 {0}else{1},"page":page,"s_num":6,"v_cache":[],"v_uniq":[]}}}),
        )?;
        if number(&root["feed"]["code"]) != Some(0) {
            return Err("个性化推荐暂不可用，请稍后重试".into());
        }
        next = page + 1;
        let ids = song_ids(&root["feed"]["data"]);
        for batch in ids.chunks(50) {
            let details = account_request(
                session,
                json!({"tracks":{"module":"music.trackInfo.UniformRuleCtrl","method":"CgiGetTrackInfo","param":{"ids":batch,"types":vec![200;batch.len()],"source":"AiNoFree"}}}),
            )?;
            if number(&details["tracks"]["code"]) != Some(0) {
                return Err("推荐歌曲信息加载失败，请重试".into());
            }
            songs.extend(filtered_tracks(
                &details["tracks"]["data"]["tracks"],
                &mut excluded,
                &excluded_ids,
            ));
        }
        if songs.len() >= 12 {
            break;
        }
        if root["feed"]["data"]["v_shelf"]
            .as_array()
            .is_none_or(|s| s.is_empty())
        {
            break;
        }
    }
    Ok(json!({"hasMore":!songs.is_empty(),"songs":songs,"nextPage":next}))
}
pub fn add_song(session: &Value, dir: u64, song_id: u64) -> Result<Value, String> {
    if dir == 0 || song_id == 0 {
        return Err("缺少有效的歌单或歌曲编号".into());
    }
    if field(session, "musicKey").is_empty() {
        return Err("请先登录 QQ 音乐".into());
    }
    let root = gateway(
        json!({"comm":{"ct":11,"cv":14090008,"v":14090008,"chid":"10003505","tmeAppID":"qqmusic","tmeLoginType":2,"qq":field(session,"uin"),"uin":field(session,"uin"),"authst":field(session,"musicKey")},"add":{"module":"music.musicasset.PlaylistDetailWrite","method":"AddSonglist","param":{"dirId":dir,"v_songInfo":[{"songId":song_id,"songType":0}]}}}),
        &session["cookies"],
    )?;
    match write_result(&root) {
        Ok(result) => Ok(result),
        Err(error) if number(&root["add"]["code"]) == Some(0) => {
            // Some versions return only an acknowledgement. Verify membership before
            // claiming success, including the idempotent "already present" case.
            let present = playlist(session, 0, dir).ok().is_some_and(|songs| {
                songs.as_array().is_some_and(|items| {
                    items
                        .iter()
                        .any(|song| number(&song["id"]) == Some(song_id as i64))
                })
            });
            if present {
                Ok(json!({"alreadyExists":true}))
            } else {
                Ok(
                    json!({"accepted":true,"confirmed":false,"message":error,"responseKeys":root["add"]["data"].as_object().map(|v|v.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()}),
                )
            }
        }
        Err(error) => Err(error),
    }
}
fn write_result(root: &Value) -> Result<Value, String> {
    if number(&root["code"]).unwrap_or(0) != 0 || number(&root["add"]["code"]) != Some(0) {
        return Err("QQ 音乐未接受添加请求，请检查登录与歌单权限".into());
    }
    let data = &root["add"]["data"];
    if number(&data["succ_song_num"]).unwrap_or(0) > 0 {
        return Ok(json!({"alreadyExists":false}));
    }
    if number(&data["fail_song_num"]).unwrap_or(0) > 0 {
        return Err("歌曲可能已存在，或添加被服务拒绝；请刷新歌单确认".into());
    }
    Err("添加结果未确认，请刷新歌单检查".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_personalized_song_shelves() {
        let card = json!({"v_niche":[{"v_card":[{"id":123},{"id":"123"},{"id":456}]}]});
        let mut songs = card.clone();
        songs["title_template"] = json!("听「{String}」也喜欢");
        songs["title_content"] = json!("test");
        let mut daily = card;
        daily["title"] = json!("每日30首");
        assert_eq!(song_ids(&json!({"v_shelf":[daily,songs]})), vec![123, 456]);
    }
    #[test]
    fn excludes_daily_seen_and_duplicate_tracks() {
        let mut seen = HashSet::from(["seen".to_string()]);
        let tracks = json!([{"mid":"daily","id":1},{"mid":"seen","id":2},{"mid":"new","id":3},{"mid":"new","id":3}]);
        let result = filtered_tracks(&tracks, &mut seen, &HashSet::from([1]));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["mid"], "new");
    }
    #[test]
    fn writes_never_infer_success_from_missing_data() {
        assert!(write_result(&json!({"code":0})).is_err());
        assert!(write_result(
            &json!({"code":0,"add":{"code":0,"data":{"succ_song_num":0,"fail_song_num":1}}})
        )
        .is_err());
        assert!(
            write_result(&json!({"code":0,"add":{"code":0,"data":{"succ_song_num":1}}})).is_ok()
        );
    }
}
