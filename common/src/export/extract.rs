use crate::export::encrypt::file_decode;
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[derive(Serialize, Deserialize)]
struct ExtractMedia {
    id: String,
    data: Vec<u8>,
}

//单个文件提取变量名
#[wasm_bindgen]
pub fn extract_one_file_variable_names(data: Uint8Array, is_decode: bool) -> Vec<String> {
    let mut file = data.to_vec();
    if is_decode {
        file = file_decode(file);
    }
    if let Ok(mut office) = crate::office::zip::new(file) {
        if let Some(vec) = office.extract_variable_names() {
            return vec;
        }
    }
    vec![]
}

//多文件批量提取变量名
#[wasm_bindgen]
pub fn extract_variable_names(files: Vec<Uint8Array>, encode_files: Vec<Uint8Array>) -> Vec<String> {
    let mut tasks = vec![];
    for file in files {
        tasks.push(extract_one_file_variable_names(file, false))
    }
    for file in encode_files {
        tasks.push(extract_one_file_variable_names(file, true))
    }

    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in tasks.into_iter().flatten() {
        if seen.get(&item).is_none() {
            seen.insert(item.clone());
            result.push(item);
        }
    }

    result
}

//单个文件提取媒体文件
#[wasm_bindgen]
pub fn extract_one_file_medias(data: &Uint8Array, is_decode: bool) -> JsValue {
    let mut file = data.to_vec();
    if is_decode {
        file = file_decode(file);
    }
    let map = _extract_one_file_medias(file);
    let mut list = vec![];
    for (id, data) in map {
        list.push(ExtractMedia {
            id,
            data,
        });
    }
    serde_wasm_bindgen::to_value(&list).unwrap()
}

//多文件批量提取媒体文件
#[wasm_bindgen]
pub fn extract_medias(files: Vec<Uint8Array>, encode_files: Vec<Uint8Array>) -> JsValue {
    let mut tasks = vec![];
    for file in files {
        let file = file.to_vec();
        tasks.push(_extract_one_file_medias(file))
    }

    for file in encode_files {
        let mut file = file.to_vec();
        file = file_decode(file);
        tasks.push(_extract_one_file_medias(file))
    }

    let mut set = HashSet::new();
    let mut list = vec![];
    for x in tasks {
        for (id, data) in x {
            if set.get(&id).is_some() {
                continue;
            }
            set.insert(id.clone());
            list.push(ExtractMedia {
                id,
                data,
            });
        }
    }
    serde_wasm_bindgen::to_value(&list).unwrap()
}

//单个文件提取媒体文件
fn _extract_one_file_medias(data: Vec<u8>) -> HashMap<String, Vec<u8>> {
    let mut map = HashMap::new();
    if let Ok(mut office) = crate::office::zip::new(data) {
        if let Some(media_map) = office.get_medias() {
            for (k, (_, data)) in media_map {
                map.insert(k, data.to_vec());
            }
        }
    }
    map
}