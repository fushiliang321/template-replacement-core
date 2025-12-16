use crate::export::common::{batch_uint8array_to_replace_file, replace_execute, replace_execute_multiple_params, replace_execute_multiple_params_to_zip, replace_execute_to_zip, uint8array_to_replace_file, BatchReplaceParams, ReplaceParams, Variables, VariablesTrait};
use crate::export::encrypt::file_decode;
use crate::office::zip::Error::NotSupported;
use crate::replace::index::{File, Replace};
use futures::future::join_all;
use js_sys::Uint8Array;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

//单个文件替换
#[wasm_bindgen]
pub async fn replace_item(
    variables: JsValue,
    medias: Vec<Uint8Array>,
    file: Uint8Array,
    is_decode: bool,
) -> Vec<u8> {
    let variables: Variables = from_value(variables).unwrap();
    let mut data = file.to_vec();
    if is_decode {
        data = file_decode(data);
    }
    match crate::office::zip::new(data).await {
        Ok(office) => {
            let execute_results = Replace::new(vec![File::Zip(office)], variables.to_data(&medias)).execute().await;
            let res = execute_results.get(0).unwrap();
            res.to_vec()
        }
        Err(err) => match err {
            NotSupported(data) => {
                data
            }
            _ => {
                if is_decode {
                    let data = file.to_vec();
                    file_decode(data)
                } else {
                    file.to_vec()
                }
            }
        }
    }
}

//文件替换，需要提前添加文件
#[wasm_bindgen]
pub async fn replace(params: JsValue, medias: Vec<Uint8Array>) -> Vec<Uint8Array> {
    let params_data: ReplaceParams = from_value(params).unwrap();
    let files = params_data.get_files();
    let variables = params_data.get_variables(&medias);
    replace_execute(variables, files).await
}

//批量文件替换
#[wasm_bindgen]
pub async fn replace_batch(
    params: JsValue,
    medias: Vec<Uint8Array>, //媒体文件
    files: Vec<Uint8Array>, //模板文件
    is_decode: bool,
) -> Vec<Uint8Array> {
    let variables: Variables = from_value(params).unwrap();
    let mut tasks = vec![];

    for file in files {
        tasks.push(uint8array_to_replace_file(file, is_decode));
    }
    let res = join_all(tasks).await;

    replace_execute(variables.to_data(&medias), res).await
}

//文件替换（多套参数），需要提前添加文件
#[wasm_bindgen]
pub async fn replace_multiple_params(params: JsValue, medias: Vec<Uint8Array>) -> Vec<Uint8Array> {
    let params_data: BatchReplaceParams = from_value(params).unwrap();
    let files = params_data.get_files();
    replace_execute_multiple_params(params_data.variables, &medias, files).await
}

//批量文件替换（多套参数）
#[wasm_bindgen]
pub async fn replace_batch_multiple_params(
    params: JsValue,
    medias: Vec<Uint8Array>, //媒体文件
    files: Vec<Uint8Array>, //模板文件
    is_decode: bool,
) -> Vec<Uint8Array> {
    let params_data: Vec<Variables> = from_value(params).unwrap();
    let files = batch_uint8array_to_replace_file(files, is_decode).await;
    replace_execute_multiple_params(params_data, &medias, files).await
}

//批量文件替换并压缩
#[wasm_bindgen]
pub async fn replace_batch_to_zip(
    params: JsValue,
    medias: Vec<Uint8Array>, //媒体文件
    files: Vec<Uint8Array>, //模板文件
    file_names: Vec<String>, //模板文件名称
    is_decode: bool,
) -> Vec<u8> {
    let variables: Variables = from_value(params).unwrap();
    let files = batch_uint8array_to_replace_file(files, is_decode).await;
    replace_execute_to_zip(variables.to_data(&medias), files, file_names).await
}

//批量文件替换并压缩（多套参数）
#[wasm_bindgen]
pub async fn replace_batch_multiple_params_to_zip(
    params: JsValue,
    medias: Vec<Uint8Array>, //媒体文件
    files: Vec<Uint8Array>, //模板文件数据
    file_names: Vec<String>, //模板文件名称
    is_decode: bool,
) -> Vec<u8> {
    let params_data: Vec<Variables> = from_value(params).unwrap();
    let files = batch_uint8array_to_replace_file(files, is_decode).await;
    replace_execute_multiple_params_to_zip(params_data, &medias, files, file_names).await
}
