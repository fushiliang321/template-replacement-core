use crate::office::office::Office;
use crate::office::zip::{new as new_zip, Zip};
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{Error, ErrorKind};

static DOCUMENT_FILE_REG_EXP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^word/(document|footer|header|diagrams/data|diagrams/drawing)(\d*).xml").unwrap()
});

const ROOT_DIR: &str = "word/";

pub struct Word {}

pub async fn new(file: Vec<u8>) -> Result<Zip, Error> {
    match new_zip(file, Box::new(Word {})).await {
        Ok(zip) => Ok(zip),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}
impl Office for Word {
    fn regex(&self) -> &Regex {
        &DOCUMENT_FILE_REG_EXP
    }

    fn root_dir(&self) -> &str {
        ROOT_DIR
    }
}
