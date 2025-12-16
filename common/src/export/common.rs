use crate::export::common::VariableValue::{Image, Text};
use crate::export::encrypt::file_decode;
use crate::office::zip::Error::NotSupported;
use crate::replace::data::{encode, Data, Value};
use crate::replace::image::{generate_id, Extent, TextWrapType};
use crate::replace::index::{File, Replace};
use crate::version;
use flate2::Crc;
use futures::future::join_all;
use js_sys::Uint8Array;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::sync::Mutex;
use wasm_bindgen::prelude::wasm_bindgen;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

lazy_static! {
    static ref INDEX: Mutex<u32> = Mutex::new(index_init());
    static ref FILES: Mutex<HashMap<u32, File>> = Mutex::new(Default::default());
    static ref MEDIA_FILES: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(Default::default());
}


#[derive(Serialize, Deserialize)]
struct WpExtent {
    cy: f32,
    cx: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Media {
    id: String,
    index: usize,
    suffix: String,
    text_wrap: TextWrapType,
    wp_extent: WpExtent,
}

#[derive(Serialize, Deserialize)]
pub enum VariableValue {
    Text(String),
    Image(Media),
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

//替换参数
#[derive(Serialize, Deserialize)]
pub struct Variables {
    pub text: HashMap<String, VariableValue>, //文本参数
    pub media: HashMap<String, VariableValue>, //媒体参数
}

//替换参数
#[derive(Serialize, Deserialize)]
pub(crate) struct ReplaceParams<T = Variables> {
    pub(crate) files: Vec<u32>, //文件数据
    pub(crate) variables: T, //参数数据
    pub(crate) file_names: Option<Vec<String>>, //模板文件名称
}

//批量替换参数
pub(crate) type BatchReplaceParams = ReplaceParams<Vec<Variables>>;

impl<T> ReplaceParams<T> {
    pub fn get_files(&self) -> Vec<File> {
        let mut file_map = FILES.lock().unwrap();
        let mut files = Vec::new();
        for file_id in &self.files {
            let file = file_map.remove(file_id).unwrap_or_else(|| {
                //如果没有获取到文件就创建一个空的结果数据
                File::Result(Vec::new())
            });
            files.push(file);
        }
        files
    }
}

impl<T: VariablesTrait> ReplaceParams<T> {
    pub(crate) fn get_variables(&self, medias: &Vec<Uint8Array>) -> Data {
        self.variables.to_data(medias)
    }
}

pub trait VariablesTrait {
    fn to_data(&self, medias: &Vec<Uint8Array>) -> Data;
}

impl VariablesTrait for Variables {
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

//添加模板文件
#[wasm_bindgen]
pub async fn add_template(file_data: Uint8Array, is_decode: bool) -> u32 {
    let mut len = file_data.length();
    let file = uint8array_to_replace_file(file_data, is_decode).await;
    let mut index = INDEX.lock().unwrap();
    let mut files = FILES.lock().unwrap();
    if len > 0 {
        len = len % 100;
    }
    //非固定长度递增，避免不同文件签名的结果被碰撞
    *index += len + 1;
    files.insert(*index, file);
    *index
}

//添加媒体文件
#[wasm_bindgen]
pub fn add_media(file: Uint8Array) -> String {
    let file = file.to_vec();
    let id = generate_id(&file);
    let mut media_files = MEDIA_FILES.lock().unwrap();
    media_files.insert(id.clone(), file);
    id
}

//文件uint8array转模板文件对象
pub(crate) async fn uint8array_to_replace_file(file: Uint8Array, is_decode: bool) -> File {
    let mut data = file.to_vec();
    if is_decode {
        data = file_decode(data);
    }
    match crate::office::zip::new(data).await {
        Ok(zip) => File::Zip(zip),
        Err(err) => match err {
            NotSupported(data) => {
                File::Result(data)
            }
            _ => {
                let mut data = file.to_vec();
                if is_decode {
                    data = file_decode(data);
                }
                File::Result(data)
            }
        }
    }
}

//批量文件uint8array转模板文件对象
pub(crate) async fn batch_uint8array_to_replace_file(files: Vec<Uint8Array>, is_decode: bool) -> Vec<File> {
    let mut tasks = vec![];
    for file in files {
        tasks.push(uint8array_to_replace_file(file, is_decode));
    }
    join_all(tasks).await
}

//多套参数批量替换
pub(crate) async fn replace_execute_multiple_params(variables: Vec<Variables>, medias: &Vec<Uint8Array>, files: Vec<File>) -> Vec<Uint8Array> {
    let mut replace_task = Replace::new(files, Data {
        text: None,
        media: None,
    });
    let mut result = vec![];
    for variable in variables {
        let variable_data = variable.to_data(&medias);
        replace_task.set_data(variable_data);
        let execute_results = replace_task.execute().await;
        for execute_result in execute_results {
            let execute_result_vec = execute_result.iter().as_slice();
            let uint8array = Uint8Array::from(execute_result_vec);
            result.push(uint8array);
        }
    }
    result
}

//单套参数替换
pub(crate) async fn replace_execute(variables: Data, files: Vec<File>) -> Vec<Uint8Array> {
    let execute_results = Replace::new(files, variables).execute().await;
    let mut result = vec![];
    for execute_result in execute_results {
        let execute_result_vec = execute_result.iter().as_slice();
        let uint8array = Uint8Array::from(execute_result_vec);
        result.push(uint8array);
    }
    result
}

//单套参数替换并生成zip
pub(crate) async fn replace_execute_to_zip(variables: Data, files: Vec<File>, file_names: Vec<String>) -> Vec<u8> {
    let execute_results = Replace::new(files, variables).execute().await;

    let mut buffer = Vec::new();
    let mut writer = ZipWriter::new(Cursor::new(&mut buffer));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for (i, execute_result) in execute_results.iter().enumerate() {
        if let Some(name) = file_names.get(i) {
            let execute_result_vec = execute_result.iter().as_slice();
            writer.start_file(format!("{}", name), options).unwrap();
            writer.write_all(execute_result_vec).unwrap();
        }
    }
    writer.finish().unwrap();
    buffer
}

//多套参数替换并生成zip
pub async fn replace_execute_multiple_params_to_zip(variables: Vec<Variables>, medias: &Vec<Uint8Array>, files: Vec<File>, file_names: Vec<String>) -> Vec<u8> {
    let mut replace_task = Replace::new(files, Data {
        text: None,
        media: None,
    });

    let mut buffer = Vec::new();
    let mut writer = ZipWriter::new(Cursor::new(&mut buffer));
    let options =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for (i, variable) in variables.iter().enumerate() {
        let variable_data = variable.to_data(&medias);
        replace_task.set_data(variable_data);
        let execute_results = replace_task.execute().await;

        for (ii, execute_result) in execute_results.iter().enumerate() {
            match file_names.get(ii) {
                Some(name) => {
                    let execute_result_vec = execute_result.iter().as_slice();
                    writer.start_file(format!("{}/{}", i, name), options).unwrap();
                    writer.write_all(execute_result_vec).unwrap();
                }
                None => {}
            }
        }
    }

    writer.finish().unwrap();
    buffer
}

//媒体文件数据转图片对象
fn media_to_image(value: &Media, medias: &Vec<Uint8Array>) -> Option<Value> {
    if !value.id.is_empty() {
        if let Some(file) = MEDIA_FILES.lock().unwrap().get(&value.id) {
            let wp_extent = Extent {
                cx: value.wp_extent.cx,
                cy: value.wp_extent.cy,
            };
            return Some(Value::Image(crate::replace::image::new(
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
        return Some(Value::Image(crate::replace::image::new(
            id,
            file.into_boxed_slice(),
            value.suffix.clone(),
            value.text_wrap.clone(),
            wp_extent,
        )));
    }
    None
}

//初始化INDEX的值
fn index_init() -> u32 {
    //根据版本信息生成随机数，避免不同版本的文件签名的结果被碰撞
    let v = version();
    let mut crc = Crc::new();
    crc.update(v.as_ref());
    crc.sum() % 100
}