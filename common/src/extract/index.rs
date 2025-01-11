use once_cell::sync::Lazy;
use regex::Regex;

pub static TEMP_FIELD_REG_EXP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$((<[^<>{}$]*?>)|\s)*\{([^{}$]+)}").unwrap()
});
pub static OUT_TAG_REG_EXP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<.*?>|\s+").unwrap()
});

//提取元素变量
pub fn raw_variables(content: &str) -> Vec<&str> {
    let mut result: Vec<&str> = vec![];
    if content.is_empty() {
        return result;
    }

    for capture in TEMP_FIELD_REG_EXP.captures_iter(content) {
        let str = capture.get(0).map_or("", |m| m.as_str());
        if !str.is_empty() {
            result.push(str);
        }
    }
    result
}

//过滤掉变量的标签信息
pub fn out_tag(content: &str) -> String {
    OUT_TAG_REG_EXP.replace_all(content, "").to_string()
}
