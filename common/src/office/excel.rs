use crate::office::office::Office;
use crate::office::zip::{new as new_zip, Zip};
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{Error, ErrorKind};

static DOCUMENT_FILE_REG_EXP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^xl/(sharedStrings|worksheets/sheet|diagrams/data|diagrams/drawing)(\d*).xml").unwrap()
});

const ROOT_DIR: &str = "xl/";

pub struct Excel {}

pub async fn new(file: Vec<u8>) -> Result<Zip, Error> {
    match new_zip(file, Box::new(Excel {})).await {
        Ok(zip) => Ok(zip),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

impl Office for Excel {
    fn regex(&self) -> &Regex {
        &DOCUMENT_FILE_REG_EXP
    }
    fn root_dir(&self) -> &str {
        ROOT_DIR
    }
}
