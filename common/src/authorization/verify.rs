use crate::VERSION;
use flate2::Crc;
use once_cell::sync::Lazy;
use rmp_serde::decode::Error;
use serde::{Deserialize, Serialize};

static SALT: Lazy<String> = Lazy::new(|| {
    //md5("zct 1.0.0")
    String::from("351100ce837185309180455d7a54855d&".to_owned() + VERSION)
});

pub fn verify(code: &String, data: &String) -> bool {
    let str = data.to_string() + "&" + &**SALT;
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