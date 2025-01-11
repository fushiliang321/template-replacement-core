#[macro_use]
extern crate console_log;
mod extract;
pub mod office;
mod replace;
mod authorization;
use crate::office::excel::new as new_excel;
use crate::office::word::new as new_word;
use crate::office::zip::Zip;
use crate::replace::data::{encode, Data, Value};
use crate::replace::image::{generate_id, TextWrapType};
use crate::replace::image::{new as new_image, Extent};
use crate::replace::index::Replace;
use crate::VariableValue::Image;
use crate::VariableValue::Text;
use js_sys::Uint8Array;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

pub const VERSION: &str = "1.0.0";

#[derive(Serialize, Deserialize)]
struct WpExtent {
    cy: f32,
    cx: f32,
}

#[derive(Serialize, Deserialize)]
struct Media {
    id: String,
    index: usize,
    suffix: String,
    text_wrap: TextWrapType,
    wp_extent: WpExtent,
}

#[derive(Serialize, Deserialize)]
enum VariableValue {
    Text(String),
    Image(Media),
}

fn media_to_image(value: &Media, medias: &Vec<Uint8Array>) -> Option<Value> {
    if !value.id.is_empty() {
        if let Some(file) = MEDIA_FILES.lock().unwrap().get(&value.id) {
            let wp_extent = Extent {
                cx: value.wp_extent.cx,
                cy: value.wp_extent.cy,
            };
            return Some(Value::Image(new_image(
                value.id.clone(),
                file.clone().into_boxed_slice(),
                value.suffix.clone(),
                value.text_wrap.clone(),
                wp_extent,
            )));
        }
    }
    if let Some(media_file) = medias.get(value.index) {
        let file = media_file.to_vec();
        let id = generate_id(&file);
        let wp_extent = Extent {
            cx: value.wp_extent.cx,
            cy: value.wp_extent.cy,
        };
        Some(Value::Image(new_image(
            id,
            file.into_boxed_slice(),
            value.suffix.clone(),
            value.text_wrap.clone(),
            wp_extent,
        )))
    } else {
        None
    }
}

impl VariableValue {
    fn to_value(&self, medias: &Vec<Uint8Array>) -> Option<Value> {
        match self {
            Text(value) => Some(Value::Text(encode(value))),
            Image(value) => media_to_image(value, medias)
        }
    }
    fn to_image(&self, medias: &Vec<Uint8Array>) -> Option<Value> {
        if let Image(value) = self {
            return media_to_image(value, medias);
        }
        None
    }
}

#[derive(Serialize, Deserialize)]
struct Variables {
    text: HashMap<String, VariableValue>,
    media: HashMap<String, VariableValue>,
}

#[derive(Serialize, Deserialize)]
struct ReplaceParams {
    files: Vec<u32>,
    variables: Variables,
}

#[derive(Serialize, Deserialize)]
struct AddReplaceParamsResult {
    version: String,
    data: String,
}

lazy_static! {
    static ref INDEX: Mutex<u32> = Mutex::new(0);
    static ref FILES: Mutex<HashMap<u32, Zip>> = Mutex::new(Default::default());
    static ref MEDIA_FILES: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(Default::default());
}

const TYPE_WORD: u8 = 0;
const TYPE_EXCEL: u8 = 1;

async fn new_office(file: Vec<u8>, _type: u8) -> Result<Zip, Error> {
    match _type {
        TYPE_WORD => new_word(file).await,
        TYPE_EXCEL => new_excel(file).await,
        _ => Err(Error::new(ErrorKind::Other, "文件类型错误")),
    }
}

impl Variables {
    fn to_data(&self, medias: &Vec<Uint8Array>) -> Data {
        let mut data = Data {
            text: None,
            media: None,
        };

        if !self.text.is_empty() {
            let mut text_data = HashMap::new();
            for (key, value) in &self.text {
                if let Some(value) = value.to_value(medias) {
                    text_data.insert(key.clone(), value);
                }
            }
            data.text = Some(text_data);
        }
        if !self.media.is_empty() {
            let mut media_data = HashMap::new();
            for (key, value) in &self.media {
                if let Some(value) = value.to_image(medias) {
                    media_data.insert(key.clone(), value);
                }
            }
            data.media = Some(media_data);
        }

        data
    }
}

impl ReplaceParams {
    async fn get_files(&self) -> Vec<Zip> {
        let mut files: Vec<Zip> = Vec::new();
        let mut file_map = FILES.lock().unwrap();
        for (_, file_id) in self.files.iter().enumerate() {
            if let Some(file) = file_map.remove(file_id) {
                files.push(file);
            }
        }
        files
    }

    fn get_variables(&self, medias: &Vec<Uint8Array>) -> Data {
        self.variables.to_data(medias)
    }
}

async fn replace_execute(variables: Data, files: Vec<Zip>) -> Vec<Uint8Array> {
    let execute_results = Replace::new(files, variables).execute().await;
    let mut result = vec![];
    for execute_result in execute_results.iter() {
        let execute_result_vec = (&**execute_result).to_vec();
        let uint8array = Uint8Array::from(execute_result_vec.as_slice());
        result.push(uint8array);
    }
    result
}

pub mod common {
    use crate::authorization::verify::{decode, verify};
    use crate::replace::image::generate_id;
    use crate::replace::index::Replace;
    use crate::{add, new_office, replace_execute, AddReplaceParamsResult, ReplaceParams, Variables, MEDIA_FILES, TYPE_EXCEL, TYPE_WORD, VERSION};
    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;
    use futures::future::join_all;
    use js_sys::Uint8Array;
    use serde_wasm_bindgen::from_value;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    //单个文件替换
    #[wasm_bindgen]
    #[cfg(not(feature = "param-sign"))]
    pub async fn replace_item(
        variables: JsValue,
        medias: Vec<Uint8Array>,
        file: Uint8Array,
    ) -> Uint8Array {
        let variables: Variables = from_value(variables).unwrap();
        let file = file.to_vec();
        match crate::office::word::new(file).await {
            Ok(office) => {
                let mut execute_results = Replace::new(vec![office], variables.to_data(&medias))
                    .execute()
                    .await;
                let res = execute_results.pop().unwrap();
                Uint8Array::from(res.as_ref())
            }
            Err(_) => Uint8Array::new(&Default::default()),
        }
    }

    //文件替换，需要提前添加文件
    #[wasm_bindgen]
    #[cfg(not(feature = "param-sign"))]
    pub async fn replace(params: JsValue, medias: Vec<Uint8Array>) -> Vec<Uint8Array> {
        let params_data: ReplaceParams = from_value(params).unwrap();
        let files = params_data.get_files().await;
        let variables = params_data.get_variables(&medias);
        replace_execute(variables, files).await
    }

    //批量文件替换
    #[wasm_bindgen]
    #[cfg(not(feature = "param-sign"))]
    pub async fn replace_batch(
        params: JsValue,
        medias: Vec<Uint8Array>,    //媒体文件
        mut files: Vec<Uint8Array>, //模板文件
        split_size: usize,          //分割大小
    ) -> Vec<Uint8Array> {
        let variables: Variables = from_value(params).unwrap();
        let mut tasks = vec![];
        let excel_size = files.len() - split_size; //excel文件数量

        //添加excel文件
        for _ in 0..excel_size {
            if let Some(file) = files.pop() {
                tasks.push(new_office(file.to_vec(), TYPE_EXCEL));
            }
        }

        //添加word文件
        while let Some(file) = files.pop() {
            tasks.push(new_office(file.to_vec(), TYPE_WORD));
        }

        let mut res = join_all(tasks).await;
        let mut files = vec![];
        while let Some(zip) = res.pop() {
            if let Ok(zip) = zip {
                files.push(zip);
            }
        }

        replace_execute(variables.to_data(&medias), files).await
    }

    //批量替换并验证参数
    #[wasm_bindgen]
    #[cfg(feature = "param-sign")]
    pub async fn replace_batch_verify(verify_code: String, params_data: String) -> Vec<Uint8Array> {
        if !verify(&verify_code, &params_data) {
            return vec![];
        }

        let data = BASE64_STANDARD.decode(&params_data).unwrap();
        let params = decode::<ReplaceParams>(data.as_slice()).unwrap();
        let files = params.get_files().await;
        let variables = params.get_variables(&vec![]);
        replace_execute(variables, files).await
    }

    //编码替换参数
    #[wasm_bindgen]
    #[cfg(feature = "param-sign")]
    pub async fn replace_params_encode(params: JsValue) -> JsValue {
        let params_data: ReplaceParams = from_value(params).unwrap();
        let encoded = crate::authorization::verify::encode(&params_data);

        serde_wasm_bindgen::to_value(&AddReplaceParamsResult {
            version: VERSION.to_string(),
            data: BASE64_STANDARD.encode(encoded),
        }).unwrap()
    }

    #[wasm_bindgen]
    pub async fn add_word(file: Uint8Array) -> u32 {
        add(file.to_vec(), TYPE_WORD).await
    }

    #[wasm_bindgen]
    pub async fn add_excel(file: Uint8Array) -> u32 {
        add(file.to_vec(), TYPE_EXCEL).await
    }

    #[wasm_bindgen]
    pub async fn add_media(file: Uint8Array) -> String {
        let file = file.to_vec();
        let id = generate_id(&file);
        let mut media_files = MEDIA_FILES.lock().unwrap();
        media_files.insert(id.clone(), file);
        id
    }
}

async fn add(file: Vec<u8>, _type: u8) -> u32 {
    let file = file.to_vec();
    let mut index = INDEX.lock().unwrap();
    let mut files = FILES.lock().unwrap();
    if let Ok(office) = new_office(file, _type).await {
        *index += 1;
        files.insert(*index, office);
        *index
    } else {
        0
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    // 初始化代码
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
