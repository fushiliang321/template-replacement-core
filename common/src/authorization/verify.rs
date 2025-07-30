use flate2::Crc;
use rmp_serde::decode::Error;
use serde::{Deserialize, Serialize};

pub const VERSION: &str = "1.0.0";

//md5("zct 1.0.0")
const SALT: &str = "351100ce837185309180455d7a54855d";

pub fn verify(code: &String, data: &String) -> bool {
    let str = format!("data={}&version={}&{}&{}", data, VERSION, SALT, VERSION);
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