use base64::{engine::general_purpose::STANDARD, Engine};
use http::header::{COOKIE, ORIGIN, REFERER, SET_COOKIE, USER_AGENT};
use jni::{
    objects::{JClass, JString},
    sys::jstring,
    JNIEnv,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const GATEWAY: &str = "https://u.y.qq.com/cgi-bin/musicu.fcg";
mod discovery;
mod artwork;
const AGENT: &str = "Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36";

#[cfg(any(target_os = "android", test))]
mod android_http;
#[cfg(target_os = "android")]
fn client() -> android_http::Client {
    android_http::Client
}
#[cfg(not(target_os = "android"))]
fn client() -> ureq::Agent {
    ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .max_redirects(0)
            .build(),
    )
}
fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
fn field<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}
fn as_text(v: &Value) -> String {
    v.as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| v.to_string())
}
fn cookie_header(cookies: &Value) -> String {
    cookies
        .as_object()
        .map(|o| {
            o.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| format!("{k}={s}")))
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default()
}
fn collect_cookies(headers: &http::HeaderMap, cookies: &mut serde_json::Map<String, Value>) {
    for h in headers.get_all(SET_COOKIE) {
        if let Ok(s) = h.to_str() {
            let pair = s.split(';').next().unwrap_or("");
            if let Some((k, v)) = pair.split_once('=') {
                if !v.is_empty() {
                    cookies.insert(k.to_owned(), json!(v));
                }
            }
        }
    }
}
fn gateway(payload: Value, cookies: &Value) -> Result<Value, String> {
    let mut body = client()
        .post(GATEWAY)
        .header(USER_AGENT, AGENT)
        .header(REFERER, "https://y.qq.com/")
        .header(ORIGIN, "https://y.qq.com")
        .header(COOKIE, cookie_header(cookies))
        .header("Content-Type", "application/json; charset=utf-8")
        .send(payload.to_string())
        .map_err(|e| e.to_string())?;
    let raw = body
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| format!("QQ 网关响应格式错误: {e}"))
}
fn qr_start() -> Result<Value, String> {
    let url = format!("https://ssl.ptlogin2.qq.com/ptqrshow?appid=716027609&e=2&l=M&s=8&d=72&v=4&t={}&daid=383&pt_3rd_aid=100497308", timestamp());
    let mut resp = client()
        .get(url)
        .header(REFERER, "https://xui.ptlogin2.qq.com/")
        .header(USER_AGENT, AGENT)
        .call()
        .map_err(|e| e.to_string())?;
    let mut cookies = serde_json::Map::new();
    collect_cookies(resp.headers(), &mut cookies);
    let sig = cookies
        .get("qrsig")
        .and_then(Value::as_str)
        .ok_or("QQ 未返回 qrsig")?
        .to_owned();
    let bytes = resp.body_mut().read_to_vec().map_err(|e| e.to_string())?;
    Ok(json!({"qrsig":sig,"image":STANDARD.encode(bytes),"mime":"image/png"}))
}
fn ptqr_token(sig: &str) -> i64 {
    let mut h: i64 = 0;
    for c in sig.chars() {
        h = h.wrapping_add(h.wrapping_shl(5).wrapping_add(c as i64));
    }
    h & 0x7fffffff
}
fn gtk(skey: &str) -> i64 {
    let mut h: i64 = 5381;
    for c in skey.chars() {
        h = h.wrapping_add(h.wrapping_shl(5).wrapping_add(c as i64));
    }
    h & 0x7fffffff
}
fn qr_poll(sig: &str) -> Result<Value, String> {
    if sig.is_empty() {
        return Err("缺少二维码凭据".into());
    }
    let url = format!("https://ssl.ptlogin2.qq.com/ptqrlogin?u1=https%3A%2F%2Fgraph.qq.com%2Foauth2.0%2Flogin_jump&ptqrtoken={}&ptredirect=0&h=1&t=1&g=1&from_ui=1&ptlang=2052&action=0-0-{}&js_ver=20102616&js_type=1&pt_uistyle=40&aid=716027609&daid=383&pt_3rd_aid=100497308&has_onekey=1",ptqr_token(sig),timestamp());
    let mut response = client()
        .get(url)
        .header(COOKIE, format!("qrsig={sig};"))
        .header(REFERER, "https://xui.ptlogin2.qq.com/")
        .header(USER_AGENT, AGENT)
        .call()
        .map_err(|e| e.to_string())?;
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    let parts: Vec<String> = body
        .split('\'')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect();
    if parts.len() < 5 {
        return Err("QQ 登录状态响应格式异常".into());
    }
    match parts[0].as_str() {
        "66" => Ok(json!({"status":"waiting"})),
        "67" => Ok(json!({"status":"confirming"})),
        "65" => Ok(json!({"status":"expired"})),
        "68" => Ok(json!({"status":"canceled"})),
        "0" => {
            let mut session = exchange_login(&parts[2], sig)?;
            if let Some(nickname) = parts.get(5) {
                session["nickname"] = json!(nickname);
            }
            Ok(json!({"status":"success","session":session}))
        }
        _ => Err(format!("QQ 登录失败: {}", parts[4])),
    }
}
fn exchange_login(redirect: &str, sig: &str) -> Result<Value, String> {
    if !redirect.starts_with("https://") {
        return Err("QQ 返回非 HTTPS 登录地址".into());
    }
    let http = client();
    let mut cookies = serde_json::Map::new();
    let first = http
        .get(redirect)
        .header(COOKIE, format!("qrsig={sig};"))
        .header(USER_AGENT, AGENT)
        .call()
        .map_err(|e| e.to_string())?;
    collect_cookies(first.headers(), &mut cookies);
    if let Some(loc) = first
        .headers()
        .get("location")
        .and_then(|h| h.to_str().ok())
    {
        if (loc.starts_with("https://graph.qq.com/") || loc.starts_with("https://y.qq.com/"))
            && !loc.contains('@')
        {
            let second = http
                .get(loc)
                .header(COOKIE, cookie_header(&Value::Object(cookies.clone())))
                .header(USER_AGENT, AGENT)
                .call()
                .map_err(|e| e.to_string())?;
            collect_cookies(second.headers(), &mut cookies);
        }
    }
    let skey = cookies
        .get("p_skey")
        .or_else(|| cookies.get("skey"))
        .and_then(Value::as_str)
        .ok_or("QQ 登录未返回 p_skey/skey")?
        .to_owned();
    let form: HashMap<&str, String> = HashMap::from([
        ("response_type", "code".into()),
        ("client_id", "100497308".into()),
        (
            "redirect_uri",
            "https://y.qq.com/wk_v17/common_login.html?type=QQ&&redirect=".into(),
        ),
        ("scope", "get_user_info".into()),
        ("state", "y_new.top.pop.logout".into()),
        ("switch", "".into()),
        ("from_ptlogin", "1".into()),
        ("src", "1".into()),
        ("update_auth", "1".into()),
        ("openapi", "8090_1010_1030_1050".into()),
        ("g_tk", gtk(&skey).to_string()),
    ]);
    let auth = http
        .post("https://graph.qq.com/oauth2.0/authorize")
        .header(COOKIE, cookie_header(&Value::Object(cookies.clone())))
        .header(USER_AGENT, AGENT)
        .send_form(&form)
        .map_err(|e| e.to_string())?;
    let location = auth
        .headers()
        .get("location")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let body = auth.into_body().read_to_string().unwrap_or_default();
    let code = extract_code(&location)
        .or_else(|| extract_code(&body))
        .ok_or("QQ OAuth 未返回 code")?;
    let login = gateway(
        json!({"comm":{"ct":19,"cv":1,"tmeLoginType":"1"},
        "login":{"module":"QQConnectLogin.LoginServer","method":"QQLogin","param":{"onlyNeedAccessToken":0,"forceRefreshToken":0,"appid":100497308,"code":code}}}),
        &Value::Null,
    )?;
    let data = &login["login"]["data"];
    let key = field(data, "musickey");
    if key.is_empty() {
        return Err("音乐服务未签发 music key；请重新登录".into());
    }
    let uin = if field(data, "str_musicid").is_empty() {
        as_text(&data["musicid"])
    } else {
        field(data, "str_musicid").to_owned()
    };
    cookies.insert("uin".into(), json!(uin));
    cookies.insert("musicid".into(), json!(uin));
    cookies.insert("qm_keyst".into(), json!(key));
    cookies.insert("qqmusic_key".into(), json!(key));
    cookies.insert("tmeLoginType".into(), json!(2));
    for name in ["openid", "access_token", "refresh_token", "refresh_key"] {
        if let Some(v) = data.get(name) {
            cookies.insert(name.into(), v.clone());
        }
    }
    Ok(json!({"uin":uin,"musicKey":key,"cookies":cookies}))
}
fn search(query: &str, page: u64, cookies: &Value) -> Result<Value, String> {
    if query.trim().is_empty() {
        return Ok(json!([]));
    }
    let payload = json!({"comm":{"ct":24,"cv":0},"music.search.SearchCgiService":{
        "module":"music.search.SearchCgiService","method":"DoSearchForQQMusicDesktop",
        "param":{"query":query,"page_num":page.max(1),"num_per_page":30,"search_type":0}}});
    let mut list = gateway(payload, cookies).ok().and_then(|v| {
        v["music.search.SearchCgiService"]["data"]["body"]["song"]["list"]
            .as_array()
            .cloned()
    });
    if list.is_none() {
        let mut resp = client()
            .get("https://c.y.qq.com/soso/fcgi-bin/client_search_cp")
            .query("w", query)
            .query("p", page.max(1).to_string())
            .query("n", "30")
            .query("format", "json")
            .header(USER_AGENT, AGENT)
            .header(REFERER, "https://y.qq.com/")
            .call()
            .map_err(|e| e.to_string())?;
        let raw = resp
            .body_mut()
            .read_to_string()
            .map_err(|e| e.to_string())?;
        let resp: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        list = resp["data"]["song"]["list"].as_array().cloned();
    }
    let songs = list.ok_or("QQ 搜索接口未返回歌曲列表")?;
    Ok(Value::Array(
        songs.iter().filter_map(normalize_song).collect(),
    ))
}
fn normalize_song(raw: &Value) -> Option<Value> {
    let song = raw.get("track").unwrap_or(raw);
    let mid = field(song, "mid");
    let mid = if mid.is_empty() {
        field(song, "songmid")
    } else {
        mid
    };
    if mid.is_empty() {
        return None;
    }
    let title = if field(song, "title").is_empty() {
        if field(song, "name").is_empty() {
            field(song, "songname")
        } else {
            field(song, "name")
        }
    } else {
        field(song, "title")
    };
    let album = if song["album"].is_object() {
        field(&song["album"], "name")
    } else {
        field(song, "albumname")
    };
    let album_mid = if song["album"].is_object() {
        field(&song["album"], "mid")
    } else {
        field(song, "albummid")
    };
    let album_mid = if album_mid.is_empty() {
        field(song, "album_mid")
    } else {
        album_mid
    };
    let artists = song["singer"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v["name"].as_str())
                .collect::<Vec<_>>()
                .join(" / ")
        })
        .unwrap_or_default();
    let media_mid = field(&song["file"], "media_mid");
    let media_mid = if media_mid.is_empty() {
        field(song, "strMediaMid")
    } else {
        media_mid
    };
    let id = song
        .get("id")
        .or_else(|| song.get("songid"))
        .cloned()
        .unwrap_or(json!(0));
    let duration = song.get("interval").cloned().unwrap_or(json!(0));
    Some(
        json!({"mid":mid,"id":id,"title":title,"artist":artists,"album":album,
        "albumMid":album_mid,"coverUrl":if album_mid.is_empty(){String::new()}else{format!("https://y.gtimg.cn/music/photo_new/T002R300x300M000{album_mid}.jpg")},"mediaMid":if media_mid.is_empty(){mid}else{media_mid},
        "duration":duration,"vip":song["pay"]["pay_play"].as_i64()==Some(1)}),
    )
}
const AUDIO_QUALITIES: [(&str, &str, &str); 3] = [
    ("sq", "F000", ".flac"), ("hq", "M800", ".mp3"), ("standard", "M500", ".mp3"),
];
fn quality_candidates(preference: &str, fallback: bool) -> Result<Vec<(&'static str, &'static str, &'static str)>, String> {
    let start = match preference { "" | "auto" | "sq" => 0, "hq" => 1, "standard" => 2, _ => return Err("不支持的音质选项".into()) };
    let end = if preference.is_empty() || preference == "auto" || fallback { 3 } else { start + 1 };
    Ok(AUDIO_QUALITIES[start..end].to_vec())
}
fn select_playback(root: &Value, preference: &str, fallback: bool) -> Result<Value, String> {
    if number(&root["code"]) == Some(2000) {
        return Err("音乐服务登录已过期，请重新扫码登录".into());
    }
    let mut diagnostics = Vec::new();
    for (name, _, _) in quality_candidates(preference, fallback)? {
        if let Some(url) = parse_playback(&root[name]) {
            return Ok(json!({"url":url,"quality":name,"requestedQuality":if preference.is_empty(){"auto"}else{preference},
                "downgraded":preference!="auto"&&!preference.is_empty()&&preference!=name}));
        }
        diagnostics.push(format!("{name}:code={},result={},path={}", root[name]["code"],
            root[name]["data"]["midurlinfo"][0]["result"], !field(&root[name]["data"]["midurlinfo"][0], "purl").is_empty()));
    }
    Err(format!("所选音质未返回可用音源（网关 {}；{}）。请检查账号权益或选择其他音质。",root["code"],diagnostics.join("；")))
}
fn play_url(song: &Value, session: &Value, preference: &str, fallback: bool) -> Result<Value, String> {
    let qualities = quality_candidates(preference, fallback)?;
    let mid = field(song, "mid");
    if mid.is_empty() {
        return Err("缺少 songMid".into());
    }
    let detail=gateway(json!({"songinfo":{"module":"music.pf_song_detail_svr","method":"get_song_detail_yqq","param":{"song_mid":mid}}}),&session["cookies"]).ok();
    let resolved = detail
        .as_ref()
        .map(|v| field(&v["songinfo"]["data"]["track_info"]["file"], "media_mid"))
        .unwrap_or("");
    let media = if !resolved.is_empty() {
        resolved
    } else if field(song, "mediaMid").is_empty() {
        mid
    } else {
        field(song, "mediaMid")
    };
    let uin = field(session, "uin");
    let key = field(session, "musicKey");
    let mut payload = json!({"comm":{"uin":uin,"format":"json","ct":28,"cv":1,"authst":key}});
    for (name, prefix, ext) in &qualities {
        payload[name] = json!({"module":"vkey.GetVkeyServer","method":"CgiGetVkey",
        "param":{"guid":"10000","songmid":[mid],"songtype":[0],"uin":uin,"loginflag":1,
        "platform":"20","filename":[format!("{prefix}{media}{ext}")]}});
    }
    let root = gateway(payload, &session["cookies"])?;
    select_playback(&root, preference, fallback)
}

fn number(v: &Value) -> Option<i64> {
    v.as_i64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}
fn https_cdn(raw: &str) -> Option<String> {
    let secure = if let Some(tail) = raw.strip_prefix("http://") {
        format!("https://{tail}")
    } else {
        raw.to_owned()
    };
    if !secure.starts_with("https://") {
        return None;
    }
    let host = secure[8..].split('/').next().unwrap_or("");
    if host == "qq.com"
        || host.ends_with(".qq.com")
        || host.ends_with(".qqmusic.qq.com")
        || host.ends_with(".gtimg.cn")
    {
        Some(secure)
    } else {
        None
    }
}
fn parse_playback(service: &Value) -> Option<String> {
    if number(&service["code"]).unwrap_or(0) != 0 {
        return None;
    }
    let data = &service["data"];
    let item = &data["midurlinfo"][0];
    if number(&item["result"]).unwrap_or(0) != 0 {
        return None;
    }
    let path = field(item, "purl");
    if path.is_empty() {
        return None;
    }
    if path.starts_with("http") {
        return https_cdn(path);
    }
    data["sip"]
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .find_map(|sip| {
            https_cdn(sip).map(|base| {
                format!(
                    "{}{}",
                    base.trim_end_matches('/').to_owned() + "/",
                    path.trim_start_matches('/')
                )
            })
        })
}
fn lyrics(mid: &str, id: u64, cookies: &Value) -> Result<Value, String> {
    let payload = json!({"comm":{"ct":24,"cv":0},"playLyricInfo":{"module":"music.musichallSong.PlayLyricInfo","method":"GetPlayLyricInfo",
        "param":{"songMID":mid,"songID":id,"qrc":0,"trans":1,"roma":0,"isHQ":1}}});
    let data = gateway(payload, cookies)?;
    let info = &data["playLyricInfo"]["data"];
    let decode = |s: &str| {
        STANDARD
            .decode(s)
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .unwrap_or_default()
    };
    Ok(json!({"lyric":decode(field(info,"lyric")),"translation":decode(field(info,"trans"))}))
}
fn account_request(session: &Value, requests: Value) -> Result<Value, String> {
    if field(session, "musicKey").is_empty() {
        return Err("请先登录音乐服务账号".into());
    }
    let mut payload = requests;
    payload["comm"] = json!({"uin":field(session,"uin"),"authst":field(session,"musicKey"),"ct":20,"cv":1770,"format":"json","platform":"wk_v17"});
    let root = gateway(payload, &session["cookies"])?;
    if root["code"].as_i64().unwrap_or(0) != 0 {
        return Err("音乐服务登录已失效或请求被拒绝，请重新登录".into());
    }
    Ok(root)
}
fn songs_in(data: &Value, key: &str) -> Result<Value, String> {
    let list = data[key]
        .as_array()
        .ok_or("音乐服务未返回歌曲列表，请刷新或重新登录")?;
    Ok(Value::Array(
        list.iter().filter_map(normalize_song).collect(),
    ))
}
fn normalized_profile(session: &Value, root: &Value) -> Value {
    let info = &root["profile"]["data"]["Info"]["BaseInfo"];
    let first = |keys: &[&str]| {
        keys.iter().find_map(|k| {
            info.get(*k)
                .and_then(Value::as_str)
                .filter(|v| !v.trim().is_empty())
        })
    };
    let nickname =
        first(&["Name", "NickName", "Nickname"]).unwrap_or_else(|| field(session, "nickname"));
    let qq = field(&session["cookies"], "p_uin")
        .trim_start_matches('o')
        .trim_start_matches('0');
    let qq = if qq.is_empty() {
        field(session, "uin")
    } else {
        qq
    };
    let fallback = if (5..=11).contains(&qq.len()) && qq.chars().all(|c| c.is_ascii_digit()) {
        format!("https://q1.qlogo.cn/g?b=qq&nk={qq}&s=640")
    } else {
        String::new()
    };
    let avatar = first(&["BigAvatar", "Avatar", "HeadPic"])
        .filter(|v| !v.contains("t_user_default"))
        .unwrap_or(&fallback)
        .replace("http://", "https://");
    json!({"nickname":nickname,"avatar":avatar,"avatarFallback":fallback,"uin":field(session,"uin")})
}
fn profile(session: &Value) -> Result<Value, String> {
    if field(session, "musicKey").is_empty() {
        return Err("请先登录".into());
    }
    let root = gateway(
        json!({"comm":{"ct":11,"cv":14090008,"v":14090008,"chid":"10003505","tmeAppID":"qqmusic","tmeLoginType":2,"qq":field(session,"uin"),"uin":field(session,"uin"),"authst":field(session,"musicKey")},"profile":{"module":"music.UnifiedHomepage.UnifiedHomepageSrv","method":"GetHomepageHeader","param":{"uin":field(session,"uin"),"IsQueryTabDetail":1}}}),
        &session["cookies"],
    )?;
    Ok(normalized_profile(session, &root))
}
fn library(session: &Value) -> Result<Value, String> {
    let uin = field(session, "uin");
    let root = account_request(
        session,
        json!({
            "likes":{"module":"music.musicasset.PlaylistDetailRead","method":"GetUniformSongDetailInfo","param":{"uin":uin,"dirid":201,"bPaged":true,"offset":0,"size":100}},
            "created":{"module":"music.musicasset.PlaylistBaseRead","method":"GetPlaylistByUin","param":{"uin":uin}},
            "saved":{"module":"music.musicasset.PlaylistFavRead","method":"GetPlaylistFavInfo","param":{"uin":uin}}
        }),
    )?;
    let likes = songs_in(&root["likes"]["data"], "list")?;
    let mut playlists = Vec::new();
    for (name, list_key, saved) in [("created", "v_playlist", false), ("saved", "v_list", true)] {
        if root[name]["code"].as_i64().unwrap_or(-1) != 0 {
            return Err("QQ 歌单获取失败，请重新登录后刷新".into());
        }
        if let Some(list) = root[name]["data"][list_key].as_array() {
            for p in list {
                let get = |keys: &[&str]| {
                    keys.iter()
                        .find_map(|k| p.get(*k))
                        .cloned()
                        .unwrap_or(Value::Null)
                };
                let dir = get(&["dirid", "dirId"]);
                if dir.as_i64() == Some(201) {
                    continue;
                }
                playlists.push(json!({"title":get(&["title","name","dirName"]),"id":get(&["tid","dissid","id"]),"dirid":dir,"count":get(&["songnum","songNum"]),"cover":get(&["picUrl","logo","pic","cover"]),"saved":saved}));
            }
        }
    }
    Ok(json!({"likes":likes,"total":root["likes"]["data"]["total"],"playlists":playlists}))
}
fn playlist(session: &Value, id: u64, dirid: u64) -> Result<Value, String> {
    if id > 0 {
        let root = account_request(
            session,
            json!({"diss":{"module":"music.srfDissInfo.aiDissInfo","method":"uniform_get_Dissinfo","param":{"disstid":id,"userinfo":1,"tag":1,"song_begin":0,"song_num":100}}}),
        )?;
        songs_in(&root["diss"]["data"], "songlist")
    } else {
        let root = account_request(
            session,
            json!({"diss":{"module":"music.musicasset.PlaylistDetailRead","method":"GetUniformSongDetailInfo","param":{"uin":field(session,"uin"),"dirid":dirid,"bPaged":true,"offset":0,"size":100}}}),
        )?;
        songs_in(&root["diss"]["data"], "list")
    }
}
fn home(session: &Value) -> Result<Value, String> {
    let root = account_request(
        session,
        json!({"feed":{"module":"music.recommend.RecommendFeed","method":"get_recommend_feed","param":{"direction":0,"page":1,"s_num":6,"v_cache":[],"v_uniq":[]}}}),
    )?;
    if root["feed"]["code"].as_i64().unwrap_or(-1) != 0 {
        return Err("推荐服务暂不可用，请刷新".into());
    }
    let mut cards = Vec::new();
    let mut daily = 0;
    if let Some(shelves) = root["feed"]["data"]["v_shelf"].as_array() {
        for shelf in shelves {
            if let Some(niches) = shelf["v_niche"].as_array() {
                for niche in niches {
                    if let Some(items) = niche["v_card"].as_array() {
                        for card in items {
                            let title = field(card, "title");
                            if title.replace(' ', "").contains("每日30首") {
                                daily = number(&card["id"])
                                    .filter(|id| *id > 0)
                                    .or_else(|| {
                                        number(&card["miscellany"]["dirid"]).filter(|id| *id > 0)
                                    })
                                    .unwrap_or(202) as u64;
                            }
                            if !title.is_empty() {
                                cards.push(discovery::recommendation_card(card));
                            }
                        }
                    }
                }
            }
        }
    }
    let songs = if daily > 0 {
        playlist(session, daily, 0).unwrap_or(json!([]))
    } else {
        json!([])
    };
    Ok(json!({"daily":songs,"dailyId":daily,"cards":cards}))
}
pub fn dispatch(command: &str, input: &str) -> String {
    let result = (|| -> Result<Value, String> {
        let v: Value = serde_json::from_str(input).map_err(|e| format!("请求 JSON 错误: {e}"))?;
        match command {
            "artwork_candidates"=>Ok(artwork::candidates(&v)),
            "profile" => profile(&v["session"]),
            "home" => home(&v["session"]),
            "recommend" => discovery::recommend(&v),
            "add_song" => discovery::add_song(&v["session"],v["dirid"].as_u64().unwrap_or(0),v["id"].as_u64().unwrap_or(0)),
            "guess" => discovery::guess(&v),
            "library" => library(&v["session"]),
            "playlist" => playlist(
                &v["session"],
                v["id"].as_u64().unwrap_or(0),
                v["dirid"].as_u64().unwrap_or(0),
            ),
            "qr_start" => qr_start(),
            "qr_poll" => qr_poll(field(&v, "qrsig")),
            "search" => search(
                field(&v, "query"),
                v["page"].as_u64().unwrap_or(1),
                &v["session"]["cookies"],
            ),
            "play_url" => play_url(&v["song"], &v["session"], field(&v,"quality"), v["fallback"].as_bool().unwrap_or(true)),
            "lyrics" => lyrics(
                field(&v, "mid"),
                v["id"].as_u64().unwrap_or(0),
                &v["session"]["cookies"],
            ),
            _ => Err("未知命令".into()),
        }
    })();
    match result {
        Ok(v) => json!({"ok":true,"data":v}).to_string(),
        Err(e) => json!({"ok":false,"error":e}).to_string(),
    }
}

fn extract_code(text: &str) -> Option<String> {
    text.split(['?', '&'])
        .find_map(|part| {
            part.strip_prefix("code=").map(|value| {
                value
                    .split(|c: char| c == '&' || c.is_whitespace() || c == '\"' || c == '\'')
                    .next()
                    .unwrap_or("")
                    .to_owned()
            })
        })
        .filter(|code| !code.is_empty())
}

#[no_mangle]
pub extern "system" fn Java_dev_qqlyric_app_RustCore_call(
    mut env: JNIEnv,
    _class: JClass,
    command: JString,
    input: JString,
) -> jstring {
    #[cfg(target_os = "android")]
    if android_http::initialize(&mut env).is_err() {
        let _ = env.exception_clear();
        return env
            .new_string(json!({"ok":false,"error":"Android HTTPS 初始化失败"}).to_string())
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut());
    }
    let command: String = match env.get_string(&command) {
        Ok(s) => s.into(),
        Err(_) => return std::ptr::null_mut(),
    };
    let input: String = match env.get_string(&input) {
        Ok(s) => s.into(),
        Err(_) => return std::ptr::null_mut(),
    };
    let output = std::panic::catch_unwind(|| dispatch(&command, &input))
        .unwrap_or_else(|_| json!({"ok":false,"error":"Rust 核心发生内部错误"}).to_string());
    env.new_string(output)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profile_uses_nonempty_avatar_and_qq_fallback() {
        let session = json!({"uin":"12345678","nickname":"用户"});
        let value = normalized_profile(
            &session,
            &json!({"profile":{"data":{"Info":{"BaseInfo":{"BigAvatar":"","Avatar":"http://qlogo.cn/user.jpg","Name":"昵称"}}}}}),
        );
        assert_eq!(value["avatar"], "https://qlogo.cn/user.jpg");
        assert_eq!(value["nickname"], "昵称");
        let fallback = normalized_profile(&session, &Value::Null);
        assert_eq!(
            fallback["avatar"],
            "https://q1.qlogo.cn/g?b=qq&nk=12345678&s=640"
        );
    }
    #[test]
    fn playback_accepts_http_cdn_and_string_result() {
        let response = json!({"code":0,"data":{"sip":["http://isure.stream.qqmusic.qq.com/"],"midurlinfo":[{"result":"0","purl":"M500test.mp3?vkey=example"}]}});
        assert_eq!(
            parse_playback(&response).as_deref(),
            Some("https://isure.stream.qqmusic.qq.com/M500test.mp3?vkey=example")
        );
    }
    #[test]
    fn playback_checks_all_cdn_candidates_and_denials() {
        let mut response = json!({"code":0,"data":{"sip":["", "https://dl.stream.qqmusic.qq.com/"],"midurlinfo":[{"result":0,"purl":"song.mp3"}]}});
        assert!(parse_playback(&response).is_some());
        response["data"]["midurlinfo"][0]["result"] = json!(100);
        assert!(parse_playback(&response).is_none());
        response["code"] = json!(2000);
        assert!(parse_playback(&response).is_none());
    }
    #[test]
    fn tokens_are_stable() {
        assert_eq!(ptqr_token("abc"), 108966);
        assert_eq!(gtk("abc"), 193485963);
    }
    #[test]
    fn song_shape() {
        let s =
            normalize_song(&json!({"songmid":"abc","songname":"测试","singer":[{"name":"歌手"}]}))
                .unwrap();
        assert_eq!(s["mid"], "abc");
        assert_eq!(s["artist"], "歌手");
    }
    #[test]
    fn errors_are_json() {
        assert_eq!(
            serde_json::from_str::<Value>(&dispatch("invalid", "{}")).unwrap()["ok"],
            false
        );
    }
    fn quality_response() -> Value {
        let response=|file: &str|json!({"code":0,"data":{"sip":["https://dl.stream.qqmusic.qq.com/"],"midurlinfo":[{"result":0,"purl":file}]}});
        json!({"code":0,"sq":response("F000track.flac"),"hq":response("M800track.mp3"),"standard":response("M500track.mp3")})
    }
    #[test] fn quality_respects_explicit_selection_and_auto() {
        let root=quality_response();
        assert_eq!(select_playback(&root,"auto",true).unwrap()["quality"],"sq");
        assert_eq!(select_playback(&root,"hq",true).unwrap()["quality"],"hq");
        assert_eq!(select_playback(&root,"standard",true).unwrap()["quality"],"standard");
        assert_eq!(quality_candidates("hq",true).unwrap().len(),2);
        assert_eq!(quality_candidates("sq",false).unwrap().len(),1);
        assert!(quality_candidates("invented",true).is_err());
    }
    #[test] fn quality_denial_falls_back_only_with_permission() {
        let mut root=quality_response();root["sq"]["data"]["midurlinfo"][0]["result"]=json!(100);
        assert!(select_playback(&root,"sq",false).is_err());
        let result=select_playback(&root,"sq",true).unwrap();
        assert_eq!(result["quality"],"hq");assert_eq!(result["downgraded"],true);
        root["hq"]["data"]["midurlinfo"][0]["purl"]=json!("");
        assert_eq!(select_playback(&root,"sq",true).unwrap()["quality"],"standard");
        assert!(select_playback(&root,"hq",false).is_err());
        root["code"]=json!(2000);assert!(select_playback(&root,"auto",true).unwrap_err().contains("过期"));
    }

}
