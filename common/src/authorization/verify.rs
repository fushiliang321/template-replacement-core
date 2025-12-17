use crate::version;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use flate2::Crc;
use once_cell::sync::OnceCell;
use rmp_serde::decode::Error;
use serde::{Deserialize, Serialize};

static SALT: OnceCell<String> = OnceCell::new();

// 需要返回盐值："351100ce837185309180455d7a54855d"
// 盐生成方式：md5("zct 1.0.0")
fn salt() -> &'static String {
    SALT.get_or_init(|| {
        // 为防止生成的wasm暴露盐值，需要对盐值做一下混淆处理
        let str1 = "35";
        // base64("1100ce837185")
        let str2 = String::from_utf8(BASE64_STANDARD.decode("MTEwMGNlODM3MTg1").unwrap()).unwrap();
        let str3 = "309";
        // base64("309180455d7a54855d")
        let str4 = String::from_utf8(BASE64_STANDARD.decode("MTgwNDU1ZDdhNTQ4NTVk").unwrap()).unwrap();
        str1.to_owned() + &*str2 + &*str3 + &*str4
    })
}

static VERSION_STR: &str = "version";
static DATA_STR: &str = "data";

pub fn verify(code: &str, data: &str) -> bool {
    let version = version();
    let salt = salt();
    let str = format!("{}={}&{}={}&{}&{}", DATA_STR, data, VERSION_STR, version, salt, version);
    let mut crc = Crc::new();
    crc.update(str.as_bytes());
    code.eq(&crc.sum().to_string())
}

pub fn encode<T>(data: &T) -> Vec<u8>
where
    T: Serialize + ?Sized,
{
    rmp_serde::to_vec(data).unwrap()
}
pub fn decode<'a, T>(input: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    rmp_serde::from_slice::<T>(input)
}