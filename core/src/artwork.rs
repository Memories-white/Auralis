//! Cover URL policy inspired by Melodist CoverUrlResolver (MIT).
use serde_json::{json,Value};
fn dimension(url:&str,size:u32)->String{
    let Some(start)=url.find('R')else{return url.into()};
    let tail=&url[start+1..];let count=tail.bytes().take_while(|c|c.is_ascii_digit()||*c==b'x').count();
    let token=&tail[..count];
    if token.contains('x')&&token.split('x').all(|p|!p.is_empty()&&p.bytes().all(|c|c.is_ascii_digit())){
        format!("{}R{size}x{size}{}",&url[..start],&tail[count..])
    }else{url.into()}
}
pub fn candidates(v:&Value)->Value{
    let url=v["url"].as_str().unwrap_or("").replacen("http://","https://",1);
    if !url.starts_with("https://"){return json!([])}
    let sizes:&[u32]=if v["player"].as_bool().unwrap_or(false){if v["metered"].as_bool().unwrap_or(true){&[800,500]}else{&[1200,800,500]}}else{&[500,800]};
    let mut urls=Vec::new();for size in sizes{let candidate=dimension(&url,*size);if !urls.contains(&candidate){urls.push(candidate);}}
    json!(urls)
}
#[cfg(test)]mod tests{use super::*;
#[test]fn player_falls_back_and_respects_metering(){let v=candidates(&json!({"url":"https://y.gtimg.cn/T002R300x300M000abc.jpg","player":true,"metered":false}));assert!(v[0].as_str().unwrap().contains("1200x1200"));assert_eq!(v.as_array().unwrap().len(),3);let v=candidates(&json!({"url":"https://y.gtimg.cn/T002R300x300M000abc.jpg","player":true,"metered":true}));assert!(v[0].as_str().unwrap().contains("800x800"));}
#[test]fn opaque_urls_are_preserved_and_local_paths_rejected(){assert_eq!(candidates(&json!({"url":"https://example.com/a.jpg"})),json!(["https://example.com/a.jpg"]));assert_eq!(candidates(&json!({"url":"file:///secret"})),json!([]));}
}
