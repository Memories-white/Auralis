//! Android transport adapter. Protocol and credentials remain owned by Rust.
use base64::{engine::general_purpose::STANDARD, Engine};
use http::{HeaderMap, HeaderName, HeaderValue};
use jni::{
    objects::{GlobalRef, JClass, JValue},
    JNIEnv, JavaVM,
};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::OnceLock};

static BRIDGE: OnceLock<(JavaVM, GlobalRef)> = OnceLock::new();
pub fn initialize(env: &mut JNIEnv) -> Result<(), String> {
    if BRIDGE.get().is_none() {
        let class = env
            .find_class("dev/qqlyric/app/AndroidHttp")
            .map_err(|e| e.to_string())?;
        let global = env.new_global_ref(class).map_err(|e| e.to_string())?;
        let vm = env.get_java_vm().map_err(|e| e.to_string())?;
        let _ = BRIDGE.set((vm, global));
    }
    Ok(())
}
pub struct Client;
pub struct Request {
    url: String,
    method: &'static str,
    headers: HashMap<String, String>,
}
pub struct Response {
    headers: HeaderMap,
    body: Body,
}
pub struct Body(Vec<u8>);
impl Client {
    pub fn get(&self, url: impl ToString) -> Request {
        Request {
            url: url.to_string(),
            method: "GET",
            headers: HashMap::new(),
        }
    }
    pub fn post(&self, url: impl ToString) -> Request {
        Request {
            url: url.to_string(),
            method: "POST",
            headers: HashMap::new(),
        }
    }
}
pub fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
impl Request {
    pub fn header(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    pub fn query(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.url
            .push(if self.url.contains('?') { '&' } else { '?' });
        self.url.push_str(&format!(
            "{}={}",
            encode(&key.to_string()),
            encode(&value.to_string())
        ));
        self
    }
    pub fn call(self) -> Result<Response, String> {
        self.execute(Vec::new())
    }
    pub fn send(self, body: impl AsRef<[u8]>) -> Result<Response, String> {
        self.execute(body.as_ref().to_vec())
    }
    pub fn send_form(self, form: &HashMap<&str, String>) -> Result<Response, String> {
        let body = form
            .iter()
            .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        self.header("Content-Type", "application/x-www-form-urlencoded")
            .send(body)
    }
    fn execute(self, body: Vec<u8>) -> Result<Response, String> {
        let (vm, class) = BRIDGE.get().ok_or("Android HTTPS 桥尚未初始化")?;
        let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
        let request=json!({"url":self.url,"method":self.method,"headers":self.headers,"body":STANDARD.encode(body)}).to_string();
        let arg = env.new_string(request).map_err(|e| e.to_string())?;
        let class: &JClass = class.as_obj().into();
        let raw = match env.call_static_method(
            class,
            "execute",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(arg.as_ref())],
        ) {
            Ok(value) => value.l().map_err(|e| e.to_string())?,
            Err(_) => {
                let _ = env.exception_clear();
                return Err("Android HTTPS 调用失败".into());
            }
        };
        let raw: String = env
            .get_string(&jni::objects::JString::from(raw))
            .map_err(|e| e.to_string())?
            .into();
        decode_response(&raw)
    }
}
fn decode_response(raw: &str) -> Result<Response, String> {
    let value: Value = serde_json::from_str(raw).map_err(|_| "Android HTTPS 返回格式错误")?;
    if let Some(error) = value["error"].as_str() {
        return Err(error.to_owned());
    }
    let status = value["status"].as_u64().ok_or("Android HTTPS 缺少状态码")?;
    if !(200..400).contains(&status) {
        return Err(format!("QQ 服务返回 HTTP {status}"));
    }
    let mut headers = HeaderMap::new();
    if let Some(map) = value["headers"].as_object() {
        for (key, values) in map {
            if let (Some(values), Ok(name)) =
                (values.as_array(), HeaderName::from_bytes(key.as_bytes()))
            {
                for value in values {
                    if let Some(value) = value.as_str() {
                        if let Ok(value) = HeaderValue::from_str(value) {
                            headers.append(name.clone(), value);
                        }
                    }
                }
            }
        }
    }
    let bytes = STANDARD
        .decode(value["body"].as_str().unwrap_or(""))
        .map_err(|_| "Android HTTPS 响应解码错误")?;
    Ok(Response {
        headers,
        body: Body(bytes),
    })
}
impl Response {
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }
    pub fn into_body(self) -> Body {
        self.body
    }
}
impl Body {
    pub fn read_to_vec(&mut self) -> Result<Vec<u8>, String> {
        Ok(std::mem::take(&mut self.0))
    }
    pub fn read_to_string(&mut self) -> Result<String, String> {
        String::from_utf8(std::mem::take(&mut self.0)).map_err(|_| "QQ 响应不是 UTF-8 文本".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_encoding_preserves_values() {
        assert_eq!(encode("中 +&="), "%E4%B8%AD%20%2B%26%3D");
    }
    #[test]
    fn redirect_keeps_all_login_cookies() {
        let response=decode_response(r#"{"status":302,"headers":{"set-cookie":["a=1; Secure","b=2; Secure"],"location":["https://y.qq.com/"]},"body":"AAEC"}"#).unwrap();
        assert_eq!(response.headers.get_all("set-cookie").iter().count(), 2);
        assert_eq!(response.body.0, vec![0, 1, 2]);
    }
    #[test]
    fn network_failure_is_not_login_success() {
        assert!(decode_response(r#"{"error":"域名解析失败"}"#).is_err());
        assert!(decode_response(r#"{"status":403,"body":""}"#).is_err());
    }
}
