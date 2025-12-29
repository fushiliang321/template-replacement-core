use crate::office::office::Office;
use regex::Regex;
use std::sync::LazyLock;

static DOCUMENT_FILE_REG_EXP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^word/(document|footer|header|diagrams/data|diagrams/drawing)(\d*).xml").unwrap()
});

const ROOT_DIR: &str = "word/";

pub struct Word {}

impl Office for Word {
    fn regex(&self) -> &Regex {
        &DOCUMENT_FILE_REG_EXP
    }

    fn root_dir(&self) -> &str {
        ROOT_DIR
    }
}
