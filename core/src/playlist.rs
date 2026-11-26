use super::{account_request, field, gateway, json, normalize_song, number, song_detail_query, Value};

const PAGE_SIZE: u64 = 100;

#[derive(Clone, Copy)]
pub(super) enum Source {
    Public(u64),
    Directory(u64),
}

impl Source {
    pub(super) fn query(self, uin: &str, offset: u64) -> Value {
        match self {
            Self::Public(id) => json!({
                "module":"music.srfDissInfo.aiDissInfo", "method":"uniform_get_Dissinfo",
                "param":{"disstid":id,"userinfo":1,"tag":1,"song_begin":offset,"song_num":PAGE_SIZE}
            }),
            Self::Directory(dirid) => json!({
                "module":"music.musicasset.PlaylistDetailRead", "method":"GetUniformSongDetailInfo",
                "param":{"uin":uin,"dirid":dirid,"bPaged":true,"offset":offset,"size":PAGE_SIZE}
            }),
        }
    }
}

pub(super) fn load(session: &Value, id: u64, dirid: u64) -> Result<Value, String> {
    load_with(field(session, "uin"), id, dirid, |payload| {
        account_request(session, payload)
    })
}

fn load_with(
    uin: &str,
    id: u64,
    dirid: u64,
    mut request: impl FnMut(Value) -> Result<Value, String>,
) -> Result<Value, String> {
    let source = match (id, dirid) {
        (id, _) if id > 0 => Source::Public(id),
        (_, dirid) if dirid > 0 => Source::Directory(dirid),
        _ => return Err("缺少有效的歌单编号".into()),
    };
    pages(source, |offset| {
        let mut root = request(json!({"diss":source.query(uin, offset)}))?;
        Ok(root["diss"].take())
    })
}

pub(super) fn pages(
    source: Source,
    mut fetch: impl FnMut(u64) -> Result<Value, String>,
) -> Result<Value, String> {
    let (list_key, total_key) = match source {
        Source::Public(_) => ("songlist", "total_song_num"),
        Source::Directory(_) => ("list", "total"),
    };
    let mut songs = Vec::new();
    let mut offset = 0;
    let mut total = None;
    let mut previous_rows = Value::Null;
    loop {
        let mut result = fetch(offset)?;
        if number(&result["code"]) != Some(0) {
            return Err("音乐服务未能读取歌单，请重试".into());
        }
        let data = &result["data"];
        let rows = data[list_key]
            .as_array()
            .ok_or("音乐服务未返回歌曲列表，请重试")?;
        if let Some(count) = number(&data[total_key]).and_then(|count| u64::try_from(count).ok()) {
            total = Some(total.map_or(count, |known: u64| known.max(count)));
        }
        if rows.is_empty() {
            if total.is_some_and(|count| offset < count) {
                return Err("音乐服务返回的歌单不完整，请重试".into());
            }
            break;
        }
        if data[list_key] == previous_rows {
            return Err("音乐服务重复返回同一页歌单，请重试".into());
        }
        // Offsets count service rows, including those without a playable song ID.
        offset += rows.len() as u64;
        songs.extend(rows.iter().filter_map(normalize_song));
        if total.map_or(rows.len() < PAGE_SIZE as usize, |count| offset >= count) {
            break;
        }
        previous_rows = result["data"][list_key].take();
    }
    Ok(Value::Array(songs))
}

pub(super) fn add_song(session: &Value, dirid: u64, tid: u64, song: &Value) -> Result<Value, String> {
    let request = |payload| gateway(payload, &session["cookies"]);
    add_with(session, dirid, tid, song, request, |id| {
        load(session, 0, dirid).is_ok_and(|songs| {
            songs.as_array().is_some_and(|items| items.iter().any(|song| song["id"].as_u64() == Some(id)))
        })
    })
}

fn add_with(
    session: &Value,
    dirid: u64,
    tid: u64,
    song: &Value,
    mut request: impl FnMut(Value) -> Result<Value, String>,
    verify_member: impl FnOnce(u64) -> bool,
) -> Result<Value, String> {
    if dirid == 0 {
        return Err("缺少有效的歌单编号".into());
    }
    let key = field(session, "musicKey");
    if key.is_empty() {
        return Err("请先登录音乐服务".into());
    }

    let identity = |track: &Value| {
        let id = number(&track["id"]).filter(|id| *id > 0)? as u64;
        let kind = number(&track["songType"]).filter(|kind| *kind >= 0)? as u64;
        Some((id, kind))
    };
    let (id, song_type) = if let Some(pair) = identity(song) {
        pair
    } else {
        let mid = field(song, "mid");
        if mid.is_empty() {
            return Err("无法识别这首歌曲，请重新选择后重试".into());
        }
        let details = request(song_detail_query(mid))?;
        normalize_song(&details["songinfo"]["data"]["track_info"])
            .filter(|track| field(track, "mid") == mid)
            .and_then(|track| identity(&track))
            .ok_or("无法获取这首歌曲的添加信息，请重试")?
    };

    let uin = field(session, "uin");
    let mut param = json!({"dirId":dirid,"v_songInfo":[{"songId":id,"songType":song_type}]});
    if dirid != 201 && tid > 0 {
        param["tid"] = json!(tid);
        param["bFmtUtf8"] = json!(true);
    }
    let root = request(json!({
        "comm":{"ct":11,"cv":14090008,"v":14090008,"chid":"10003505","tmeAppID":"qqmusic","tmeLoginType":2,"qq":uin,"uin":uin,"authst":key},
        "add":{"module":"music.musicasset.PlaylistDetailWrite","method":"AddSonglist","param":param}
    }))?;
    match write_result(&root)? {
        WriteReceipt::Added => Ok(json!({"alreadyExists":false})),
        _ if verify_member(id) => Ok(json!({"alreadyExists":true})),
        WriteReceipt::Accepted => Ok(json!({"confirmed":false})),
        WriteReceipt::CheckExisting => Err("音乐服务未确认添加，请打开歌单检查后重试".into()),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum WriteReceipt {
    Added,
    Accepted,
    CheckExisting,
}

fn write_result(root: &Value) -> Result<WriteReceipt, String> {
    let code = number(&root["code"]).unwrap_or(0);
    let add = &root["add"];
    let module_code = number(&add["code"]);
    let data = &add["data"];
    let business_code = number(&data["retCode"]);
    if code != 0 || module_code != Some(0) || business_code.is_some_and(|code| code != 0) {
        return Err("音乐服务拒绝添加，请检查登录状态和歌单权限".into());
    }
    if number(&data["fail_song_num"]).unwrap_or(0) > 0 {
        return Ok(WriteReceipt::CheckExisting);
    }
    if number(&data["succ_song_num"]).unwrap_or(0) > 0 {
        return Ok(WriteReceipt::Added);
    }
    if business_code == Some(0) {
        return Ok(WriteReceipt::Accepted);
    }
    Ok(WriteReceipt::CheckExisting)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_playlist_keeps_song_101_in_order() {
        let mut offsets = Vec::new();
        let songs = load_with("", 1, 0, |request| {
            let offset = request["diss"]["param"]["song_begin"].as_u64().unwrap();
            offsets.push(offset);
            let page = (offset..(offset + 100).min(101))
                .map(|i| json!({"mid":format!("track-{i}")}))
                .collect::<Vec<_>>();
            Ok(json!({"diss":{"code":0,"data":{"songlist":page,"total_song_num":101}}}))
        })
        .unwrap();
        assert_eq!(songs.as_array().unwrap().len(), 101);
        assert_eq!(songs[100]["mid"], "track-100");
        assert_eq!(offsets, [0, 100]);
    }

    #[test]
    fn library_directory_requests_next_page_after_an_unavailable_row() {
        let mut offsets = Vec::new();
        let songs = load_with("123", 0, 2, |request| {
            let offset = request["diss"]["param"]["offset"].as_u64().unwrap();
            offsets.push(offset);
            let page = (offset..(offset + 100).min(102))
                .map(|i| {
                    if i == 0 {
                        Value::Null
                    } else {
                        json!({"track":{"mid":format!("track-{i}")}})
                    }
                })
                .collect::<Vec<_>>();
            Ok(json!({"diss":{"code":0,"data":{"list":page,"total":102}}}))
        })
        .unwrap();
        assert_eq!(songs.as_array().unwrap().len(), 101);
        assert_eq!(songs[100]["mid"], "track-101");
        assert_eq!(offsets, [0, 100]);
    }

    #[test]
    fn an_incomplete_playlist_returns_a_retryable_error() {
        let result = pages(Source::Public(1), |offset| match offset {
            0 => Ok(json!({"code":0,"data":{"songlist":[{"mid":"first"}],"total_song_num":2}})),
            1 => Ok(json!({"code":0,"data":{"songlist":[],"total_song_num":0}})),
            _ => panic!("unexpected playlist offset {offset}"),
        });
        assert!(result.unwrap_err().contains("不完整"));
    }

    #[test]
    fn add_resolves_old_queue_tracks_and_verifies_the_receipt() {
        let session = json!({"uin":"123","musicKey":"fixture"});
        let mut writes = 0;
        let result = add_with(&session, 7, 29, &json!({"id":42,"mid":"fixture"}), |request| {
            if let Some(detail) = request.get("songinfo") {
                assert_eq!(detail["param"]["song_mid"], "fixture");
                return Ok(json!({"songinfo":{"data":{"track_info":{"id":42,"mid":"fixture","type":13}}}}));
            }
            writes += 1;
            assert_eq!(request["add"]["param"], json!({"dirId":7,"tid":29,"bFmtUtf8":true,
                "v_songInfo":[{"songId":42,"songType":13}]}));
            Ok(json!({"code":0,"add":{"code":0,"data":{
                "retCode":0,"result":{"songlist":[{"existed":0}]}
            }}}))
        }, |id| id == 42).unwrap();
        assert_eq!(writes, 1);
        assert_eq!(result["alreadyExists"], true);
    }

    #[test]
    fn writes_do_not_report_success_when_the_service_rejects_or_only_acknowledges() {
        assert!(write_result(&json!({"code":0})).is_err());
        assert!(write_result(&json!({"code":0,"add":{"code":0,"data":{
            "retCode":1,"succ_song_num":1
        }}})).is_err());
        assert_eq!(write_result(&json!({"code":0,"add":{"code":0,"data":{
            "fail_song_num":1
        }}})), Ok(WriteReceipt::CheckExisting));
        assert_eq!(write_result(&json!({"code":0,"add":{"code":0,"data":{
            "succ_song_num":1
        }}})), Ok(WriteReceipt::Added));
        let result = add_with(&json!({"musicKey":"fixture"}), 7, 29,
            &json!({"id":42,"songType":0}),
            |_| Ok(json!({"code":0,"add":{"code":0,"data":{"retCode":0}}})), |_| false).unwrap();
        assert_eq!(result["confirmed"], false);
    }
}
