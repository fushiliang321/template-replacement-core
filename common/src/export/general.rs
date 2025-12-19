use crate::export::common::{batch_uint8array_to_replace_file, replace_execute, replace_execute_multiple_params, BatchReplaceParams, ReplaceParams, Variables, VariablesTrait};
use crate::export::encrypt::file_decode;
use crate::replace::index::{File, Replace};
use js_sys::Uint8Array;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

//单个文件替换
#[wasm_bindgen]
pub fn replace_item(
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
    match crate::office::zip::new(data) {
        Ok(office) => {
            let execute_results = Replace::new(vec![File::Zip(office)], variables.to_data(&medias)).execute();
            let res = execute_results.get(0).unwrap();
            res.to_vec()
        }
        Err(_) => {
            let mut data = file.to_vec();
            if is_decode {
                data = file_decode(data);
            }
            data
        }
    }
}

//文件替换，需要提前添加文件
#[wasm_bindgen]
pub fn replace(params: JsValue, medias: Vec<Uint8Array>) -> Vec<Uint8Array> {
    let params_data: ReplaceParams = from_value(params).unwrap();
    let files = params_data.get_files();
    let variables = params_data.get_variables(&medias);
    replace_execute(variables, files)
}

//批量文件替换
#[wasm_bindgen]
pub fn replace_batch(
    params: JsValue,
    medias: Vec<Uint8Array>, //媒体文件
    files: Vec<Uint8Array>, //模板文件
    encode_files: Vec<Uint8Array>, //加密的模板文件
) -> Vec<Uint8Array> {
    let variables: Variables = from_value(params).unwrap();
    let files = batch_uint8array_to_replace_file(files, encode_files);
    replace_execute(variables.to_data(&medias), files)
}

//文件替换（多套参数），需要提前添加文件
#[wasm_bindgen]
pub fn replace_multiple_params(params: JsValue, medias: Vec<Uint8Array>) -> Vec<Uint8Array> {
    let params_data: BatchReplaceParams = from_value(params).unwrap();
    let files = params_data.get_files();
    replace_execute_multiple_params(params_data.variables, &medias, files)
}

//批量文件替换（多套参数）
#[wasm_bindgen]
pub fn replace_batch_multiple_params(
    params: JsValue,
    medias: Vec<Uint8Array>, //媒体文件
    files: Vec<Uint8Array>, //模板文件
    encode_files: Vec<Uint8Array>, //加密的模板文件
) -> Vec<Uint8Array> {
    let params_data: Vec<Variables> = from_value(params).unwrap();
    let files = batch_uint8array_to_replace_file(files, encode_files);
    replace_execute_multiple_params(params_data, &medias, files)
}
