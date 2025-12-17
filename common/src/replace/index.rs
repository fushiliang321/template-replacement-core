use crate::extract::index::{out_tag, raw_variables, TEMP_EXCLUDE_REG_EXP, TEMP_FIELD_REG_EXP};
use crate::office::zip::{RelationshipInfo, Zip};
use crate::replace::data::Value::{Image, Text};
use crate::replace::data::{Data, Value};
use crate::replace::thread::Thread;
use futures::future::join_all;
use std::collections::{HashMap, VecDeque};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};


pub enum File {
    Zip(Zip),
    Result(Vec<u8>),
}

pub struct Replace {
    files: VecDeque<File>,
    data: Arc<Data>,
}

// 提取变量
fn extract_variables(content: &str) -> HashMap<&str, String> {
    let mut variables: HashMap<&str, String> = HashMap::new();
    if content.eq("") {
        return variables;
    }
    //提取出原始变量
    let raw_vars = raw_variables(content);
    if raw_vars.is_empty() {
        return variables;
    }
    for raw_var in raw_vars {
        let var_name = out_tag(raw_var);
        variables.insert(raw_var, var_name);
    }
    variables
}

struct ReplaceContentResult {
    content: Box<[u8]>,
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
        .to_vec()
        .into_boxed_slice();

    if !is_change {
        return None;
    }
    Some(ReplaceContentResult { content, medias })
}

pub async fn replace_lock(office_mutex: Arc<Mutex<Zip>>, data: Arc<Data>) -> Box<[u8]> {
    let mut office = office_mutex.lock().unwrap();
    if let Some(files) = office.match_document_contents().await {
        if let Some(text) = &data.text {
            for (file_name, file_data) in files {
                //提取出原始变量
                if let Ok(content) = from_utf8(&file_data) {
                    if let Some(replace_content_result) = replace_content(content, text) {
                        office.write_file(&file_name, replace_content_result.content);
                    }
                }
            }
        }
    }
    office.finish().await
}

fn get_filename(path: &str) -> Option<&str> {
    match path.rfind('/') {
        Some(index) => Some(&path[index + 1..]),
        None => if path.is_empty() { None } else { Some(path) },
    }
}

pub async fn replace(file: &mut File, data: &Data) -> Box<[u8]> {
    match file {
        File::Zip(office) => {
            if let Some(text) = &data.text &&
                !text.is_empty() &&
                let Some(files) = office.match_document_contents().await {
                for (file_name, file_data) in files {
                    //提取出原始变量
                    if let Ok(content) = from_utf8(&file_data) {
                        if let Some(replace_content_result) = replace_content(content, text) {
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
            }

            if let Some(medias) = &data.media &&
                !medias.is_empty() &&
                let Some(office_medias) = office.get_media_names().await {
                for (key, name) in office_medias {
                    if let Some(file) = medias.get(&key) {
                        match file {
                            Text(_) => {}
                            Image(file) => {
                                office.write_media(name, file.file.clone());
                            }
                        }
                    }
                }
            }

            office.finish().await
        }
        File::Result(file) => {
            file.to_vec().into_boxed_slice()
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

    pub fn execute_thread(&mut self, mut concurrency: u8) -> Vec<Box<[u8]>> {
        if self.data.is_empty() {
            return Vec::new();
        }
        if concurrency < 1 {
            concurrency = 1;
        }
        let mut thread = Thread::new(concurrency, Arc::clone(&self.data));
        let mut ids = Vec::new();
        while let Some(office) = self.files.pop_front() {
            let office = Arc::new(Mutex::new(office));
            let id = thread.add(office);
            ids.push(id);
        }
        thread.over(ids)
    }

    async fn vec_to_box(v: &mut Vec<u8>) -> Box<[u8]> {
        v.to_vec().into_boxed_slice()
    }

    pub async fn execute(&mut self) -> Vec<Box<[u8]>> {
        let mut tasks: Vec<_> = vec![];
        for file in self.files.iter_mut() {
            tasks.push(replace(file, &self.data));
        }
        join_all(tasks).await
    }
}
