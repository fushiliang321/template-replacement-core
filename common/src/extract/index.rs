use once_cell::sync::Lazy;
use regex::Regex;

pub static TEMP_FIELD_REG_EXP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$((<[^<>{}$]*?>)|\s)*\{([^{}$]+)}").unwrap()
});
pub static TEMP_EXCLUDE_REG_EXP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)<(/|)w:(p|drawing|tc|tbl)>").unwrap()
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
        let str = capture.get(0).unwrap().as_str();
        if !TEMP_EXCLUDE_REG_EXP.is_match(str) {
            result.push(str);
        }
    }
    result
}

//过滤掉变量的标签信息
pub fn out_tag(content: &str) -> String {
    OUT_TAG_REG_EXP.replace_all(content, "").to_string()
}
