use super::*;
use std::collections::HashSet;

const SONG_CARD: i64 = 200;
const PLAYLIST_CARD: i64 = 500;

pub(super) fn recommendation_card(card: &Value) -> Option<Value> {
    let title = field(card, "title");
    let id = number(&card["id"]).filter(|id| *id > 0).unwrap_or(0);
    let dirid = number(&card["miscellany"]["dirid"])
        .filter(|id| *id > 0)
        .unwrap_or(0);
    let kind = if title.contains("猜你喜欢") {
        "guess"
    } else if number(&card["type"]) == Some(PLAYLIST_CARD) && (id > 0 || dirid > 0) {
        "playlist"
    } else {
        return None;
    };
    (!title.trim().is_empty()).then(|| {
        json!({"title":title,"id":id,"cover":card["cover"],"dirid":dirid,"kind":kind})
    })
}

// This is a radio recommendation, not a disstid accepted by PlaylistDetailRead.
pub fn guess(input: &Value) -> Result<Value, String> {
    let session = &input["session"];
    if field(session, "musicKey").is_empty() {
        return Err("请先登录音乐服务账号".into());
    }
    let mut excluded: HashSet<String> = input["exclude"].as_array().into_iter().flatten()
        .filter_map(Value::as_str).map(str::to_owned).collect();
    // One batch per open. A retry belongs to a new explicit request from the page.
    let root = gateway(json!({"comm":{"uin":field(session,"uin"),"authst":field(session,"musicKey"),"format":"json","ct":19,"cv":1},
        "guess":{"module":"music.radioProxy.MbTrackRadioSvr","method":"get_radio_track","param":{"id":99,"num":15,"from":0,"scene":0,"song_ids":[]}}}), &session["cookies"])?;
    let mut songs = guess_tracks(&root, &mut excluded)?;
    if songs.is_empty() { return Err("猜你喜欢暂时没有新歌曲，请稍后刷新".into()); }
    songs.truncate(30);
    Ok(json!(songs))
}
fn guess_tracks(root: &Value, excluded: &mut HashSet<String>) -> Result<Vec<Value>, String> {
    if number(&root["code"]).unwrap_or(0) != 0 || number(&root["guess"]["code"]) != Some(0) {
        return Err("猜你喜欢暂不可用，请刷新或重新登录".into());
    }
    let tracks = root["guess"]["data"]["tracks"]
        .as_array()
        .ok_or("推荐服务未返回歌曲列表，请重试")?;
    Ok(filtered_tracks(tracks, excluded, &HashSet::new()))
}

pub(super) fn feed_cards(data: &Value) -> impl Iterator<Item = &Value> {
    data["v_shelf"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|shelf| shelf["v_niche"].as_array().into_iter().flatten())
        .flat_map(|niche| niche["v_card"].as_array().into_iter().flatten())
}

fn song_ids(data: &Value) -> Vec<u64> {
    let mut seen = HashSet::new();
    feed_cards(data)
        .filter(|card| number(&card["type"]) == Some(SONG_CARD))
        .filter_map(|card| number(&card["id"]).filter(|id| *id > 0))
        .map(|id| id as u64)
        .filter(|id| seen.insert(*id))
        .collect()
}
fn filtered_tracks(
    tracks: &[Value],
    excluded: &mut HashSet<String>,
    excluded_ids: &HashSet<u64>,
) -> Vec<Value> {
    tracks
        .iter()
        .filter_map(normalize_song)
        .filter(|song| {
            let mid = field(song, "mid").to_owned();
            !excluded_ids.contains(&(number(&song["id"]).unwrap_or(0) as u64))
                && excluded.insert(mid)
        })
        .collect()
}

pub(super) fn load_feed(
    page: u64,
    mut request: impl FnMut(Value) -> Result<Value, String>,
) -> Result<Value, String> {
    let mut root = request(json!({"feed":{
        "module":"music.recommend.RecommendFeed", "method":"get_recommend_feed",
        "param":{"direction":if page == 1 { 0 } else { 1 },"page":page,"s_num":6,"v_cache":[],"v_uniq":[]}
    }}))?;
    let feed = &mut root["feed"];
    if number(&feed["code"]) != Some(0) || !feed["data"]["v_shelf"].is_array() {
        return Err("个性化推荐暂不可用，请稍后重试".into());
    }
    Ok(feed["data"].take())
}

fn recommendation_page(
    page: u64,
    request: &mut impl FnMut(Value) -> Result<Value, String>,
) -> Result<(Vec<Value>, bool), String> {
    let feed = load_feed(page, &mut *request)?;
    let mut tracks = Vec::new();
    for batch in song_ids(&feed).chunks(50) {
        let mut details = request(json!({"tracks":{
            "module":"music.trackInfo.UniformRuleCtrl", "method":"CgiGetTrackInfo",
            "param":{"ids":batch,"types":vec![SONG_CARD; batch.len()],"source":"AiNoFree"}
        }}))?;
        let result = &mut details["tracks"];
        if number(&result["code"]) != Some(0) {
            return Err("推荐歌曲信息加载失败，请重试".into());
        }
        let list = result["data"]["tracks"]
            .as_array_mut()
            .ok_or("推荐服务未返回歌曲列表，请重试")?;
        tracks.append(list);
    }
    let has_more = feed["v_shelf"].as_array().is_some_and(|shelves| !shelves.is_empty());
    Ok((tracks, has_more))
}

pub fn recommend(input: &Value) -> Result<Value, String> {
    let session = &input["session"];
    recommend_with(input, |payload| account_request(session, payload))
}
fn recommend_with(
    input: &Value,
    mut request: impl FnMut(Value) -> Result<Value, String>,
) -> Result<Value, String> {
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
    let mut has_more = true;
    for page in start..start.saturating_add(3) {
        let (tracks, more) = match recommendation_page(page, &mut request) {
            Ok(result) => result,
            Err(error) if songs.is_empty() => return Err(error),
            Err(error) => {
                return Ok(json!({"hasMore":true,"songs":songs,"nextPage":page,"error":error}));
            }
        };
        next = page + 1;
        has_more = more;
        songs.extend(filtered_tracks(&tracks, &mut excluded, &excluded_ids));
        if songs.len() >= 12 || !has_more {
            break;
        }
    }
    Ok(json!({"hasMore":has_more,"songs":songs,"nextPage":next}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recommendation_playlists_ignore_non_playlist_feed_cards() {
        let cards = [
            json!({"type":700,"id":161,"title":"feed shortcut"}),
            json!({"type":200,"id":162,"title":"song"}),
            json!({"type":400,"id":163,"title":"album"}),
            json!({"type":900,"id":164,"title":"feed shortcut"}),
            json!({"type":"500","id":165,"title":"playlist"}),
        ];
        let playlists = cards.iter().filter_map(recommendation_card).collect::<Vec<_>>();
        assert_eq!(playlists.iter().map(|card| &card["id"]).collect::<Vec<_>>(), [&json!(165)]);
    }

    #[test]
    fn recommendation_reads_song_cards_without_relying_on_shelf_title() {
        let data = json!({"v_shelf":[{"title":"本周推荐","v_niche":[{"v_card":[
            {"type":"200","id":11},{"type":500,"id":12},{"type":400,"id":13},{"type":200,"id":11}
        ]}]}]});
        assert_eq!(song_ids(&data), [11]);
    }
    #[test]
    fn recommendation_can_continue_after_a_window_of_excluded_songs() {
        let result = recommend_with(&json!({"exclude":["seen"],"page":1}), |request| {
            if request.get("feed").is_some() {
                Ok(json!({"feed":{"code":0,"data":{"v_shelf":[{
                    "v_niche":[{"v_card":[{"id":1,"type":200}]}]
                }]}}}))
            } else {
                Ok(json!({"tracks":{"code":0,"data":{"tracks":[{"mid":"seen","id":1}]}}}))
            }
        }).unwrap();
        assert_eq!(result["songs"], json!([]));
        assert_eq!(result["nextPage"], 4);
        assert_eq!(result["hasMore"], true);
    }

    #[test]
    fn recommendation_keeps_an_earlier_page_when_a_later_request_fails() {
        let result = recommend_with(&json!({"page":1}), |request| {
            if let Some(feed) = request.get("feed") {
                if feed["param"]["page"] == 2 { return Err("simulated network failure".into()); }
                Ok(json!({"feed":{"code":0,"data":{"v_shelf":[{
                    "v_niche":[{"v_card":[{"id":1,"type":200}]}]
                }]}}}))
            } else {
                Ok(json!({"tracks":{"code":0,"data":{"tracks":[{"mid":"new","id":1}]}}}))
            }
        }).unwrap();
        assert_eq!(result["songs"][0]["mid"], "new");
        assert_eq!(result["nextPage"], 2);
        assert_eq!(result["hasMore"], true);
        assert_eq!(result["error"], "simulated network failure");
    }
    #[test]
    fn guess_card_is_not_a_playlist() {
        assert_eq!(recommendation_card(&json!({"title":"猜你喜欢-沉浸刷歌","id":99})).unwrap()["kind"], "guess");
        let playlist = recommendation_card(&json!({"title":"每日30首","type":500,"id":123,"miscellany":{"dirid":202}})).unwrap();
        assert_eq!(playlist["kind"], "playlist");
        assert_eq!(playlist["dirid"], 202);
    }
    #[test]
    fn guess_rejects_failures_and_normalizes_one_batch() {
        let mut excluded = HashSet::from(["daily".to_owned()]);
        assert!(guess_tracks(&json!({"code":0,"guess":{"code":1}}), &mut excluded).is_err());
        assert!(guess_tracks(&json!({"code":0,"guess":{"code":0,"data":{}}}), &mut excluded).is_err());
        let result = guess_tracks(&json!({"code":0,"guess":{"code":0,"data":{"tracks":[
            {"mid":"daily","id":1},{"mid":"new","id":2,"title":"New"},{"mid":"new","id":2}
        ]}}}), &mut excluded).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["title"], "New");
    }
    #[test]
    fn only_personalized_song_shelves() {
        let songs = json!({"v_niche":[{"v_card":[{"id":123,"type":200},{"id":"123","type":"200"},{"id":456,"type":200}]}]});
        let daily = json!({"v_niche":[{"v_card":[{"id":999,"type":500}]}]});
        assert_eq!(song_ids(&json!({"v_shelf":[daily,songs]})), vec![123, 456]);
    }
    #[test]
    fn excludes_daily_seen_and_duplicate_tracks() {
        let mut seen = HashSet::from(["seen".to_string()]);
        let tracks = json!([{"mid":"daily","id":1},{"mid":"seen","id":2},{"mid":"new","id":3},{"mid":"new","id":3}]);
        let result = filtered_tracks(tracks.as_array().unwrap(), &mut seen, &HashSet::from([1]));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["mid"], "new");
    }
}
