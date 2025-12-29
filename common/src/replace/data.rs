use crate::replace::image::Image;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

pub enum Value {
    Text(String),
    Image(Image),
}

pub struct DataBase<T> {
    pub text: Option<HashMap<String, T>>,
    pub media: Option<HashMap<String, T>>,
}

pub type Data = DataBase<Value>;

//特殊字符编码
static CHARACTER_ENCODER_MAP: LazyLock<HashMap<char, &str>> = LazyLock::new(|| {
    HashMap::from([
        ('<', "&lt;"),
        ('>', "&gt;"),
        ("'".parse().unwrap(), "&apos;"),
        ('"', "&quot;"),
        ('&', "&amp;"),
    ])
});

//特殊字符解码
static CHARACTER_DECODER_MAP: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        ("lt", "<"),
        ("gt", ">"),
        ("apos", "'"),
        ("quot", "\""),
        ("amp", "&"),
    ])
});

static DOCUMENT_FILE_REG_EXP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)&(lt|gt|apos|amp|quot);").unwrap()
});


pub fn encode(input: &str) -> String {
    input.chars().map(|c| {
        CHARACTER_ENCODER_MAP.get(&c).unwrap_or(&&*c.to_string()).to_string()
    }).collect()
}

pub fn decode(input: &str) -> String {
    DOCUMENT_FILE_REG_EXP.replace_all(input, |caps: &regex::Captures| {
        let entity = &caps[1];
        CHARACTER_DECODER_MAP.get(entity).unwrap_or(&entity).to_string()
    }).to_string()
}

impl Data {
    pub fn new() -> Self {
        Data {
            text: None,
            media: None,
        }
    }

    // 判断是否为空参数
    pub fn is_empty(&self) -> bool {
        if self.text.is_some() && !self.text.as_ref().unwrap().is_empty() {
            return false;
        }
        if self.media.is_some() && !self.media.as_ref().unwrap().is_empty() {
            return false;
        }
        true
    }
}
