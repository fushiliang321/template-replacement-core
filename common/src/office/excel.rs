use crate::office::office::Office;
use once_cell::sync::Lazy;
use regex::Regex;

static DOCUMENT_FILE_REG_EXP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^xl/(sharedStrings|worksheets/sheet|diagrams/data|diagrams/drawing)(\d*).xml").unwrap()
});

const ROOT_DIR: &str = "xl/";

pub struct Excel {}

impl Office for Excel {
    fn regex(&self) -> &Regex {
        &DOCUMENT_FILE_REG_EXP
    }
    fn root_dir(&self) -> &str {
        ROOT_DIR
    }
}
