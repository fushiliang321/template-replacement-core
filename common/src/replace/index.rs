use crate::extract::index::{out_tag, TEMP_EXCLUDE_REG_EXP, TEMP_FIELD_REG_EXP};
use crate::office::zip::{RelationshipInfo, Zip};
use crate::replace::data::Value::{Image, Text};
use crate::replace::data::{Data, Value};
use std::collections::{HashMap, VecDeque};
use std::str::from_utf8;
use std::sync::Arc;


pub enum File {
    Zip(Zip),
    Result(Vec<u8>),
}

pub struct Replace {
    files: VecDeque<File>,
    data: Arc<Data>,
}

struct ReplaceContentResult {
    content: Vec<u8>,
    medias: HashMap<String, Box<crate::replace::image::Image>>,
}

fn replace_content(
    input: &str,
    replacements: &HashMap<String, Value>,
) -> Option<ReplaceContentResult> {
    let mut medias: HashMap<String, Box<crate::replace::image::Image>> = HashMap::new();
    let mut is_change = false;
    let content = TEMP_FIELD_REG_EXP
        .replace_all(input, |caps: &regex::Captures| {
            let str = caps.get(0).unwrap().as_str();
            if TEMP_EXCLUDE_REG_EXP.is_match(str) {
                return str.to_string();
            }
            let key = out_tag(str);
            if let Some(value) = replacements.get(&key) {
                is_change = true;
                return match value {
                    Text(value) => value.to_string(),
                    Image(value) => {
                        medias.insert(value.id.clone(), Box::new(value.clone()));
                        value.to_string()
                    }
                };
            }
            str.to_string()
        })
        .as_bytes()
        .to_vec();

    if !is_change {
        return None;
    }
    Some(ReplaceContentResult { content, medias })
}

pub fn replace(file: &mut File, data: &Data) -> Vec<u8> {
    match file {
        File::Zip(office) => {
            if let Some(text) = &data.text &&
                !text.is_empty() {
                let files = office.match_document_contents();
                for (file_name, file_data) in files {
                    //提取出原始变量
                    if let Ok(content) = from_utf8(&file_data) &&
                        let Some(replace_content_result) = replace_content(content, text) {
                        office.write_file(&file_name.to_string(), replace_content_result.content);
                        if replace_content_result.medias.is_empty() {
                            continue;
                        }
                        let mut relationships = vec![];
                        let name = (&file_name[file_name.rfind('/').unwrap() + 1..]).to_owned() + ".rels";
                        for (key, file) in replace_content_result.medias {
                            let name = format!("{}media/{}", office.office().root_dir(), key);
                            office.write_media(name, file.file);
                            relationships.push(RelationshipInfo {
                                id: key.clone(),
                                target: key,
                                _type: String::from(file.relationship),
                            });
                        }
                        office.write_relationships(name, relationships);
                    }
                }
            }

            if let Some(medias) = &data.media &&
                !medias.is_empty() {
                let office_medias = office.get_media_names();
                for (key, name) in office_medias {
                    if let Some(file) = medias.get(&key) {
                        if let Image(file) = file {
                            office.write_media(name, file.file.clone());
                        }
                    }
                }
            }

            office.finish()
        }
        File::Result(file) => {
            file.to_vec()
        }
    }
}

impl Replace {
    pub fn new(files: Vec<File>, data: Data) -> Replace {
        Replace {
            files: VecDeque::from(files),
            data: Arc::new(data),
        }
    }

    pub fn set_data(&mut self, data: Data) {
        self.data = Arc::new(data)
    }

    pub fn execute(&mut self) -> Vec<Vec<u8>> {
        let mut result: Vec<_> = vec![];
        for file in self.files.iter_mut() {
            result.push(replace(file, &self.data));
        }
        result
    }
}
