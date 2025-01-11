use regex::Regex;

pub trait Office: Send + Sync {
    fn regex(&self) -> &Regex;
    fn root_dir(&self) -> &str;
}
