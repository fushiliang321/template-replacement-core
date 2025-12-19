use crate::authorization::verify::{decode, encode, verify};
use crate::export::common::{replace_execute, replace_execute_multiple_params, BatchReplaceParams, ReplaceParams};
use crate::version;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[derive(Serialize, Deserialize)]
pub(crate) struct AddReplaceParamsResult {
    pub(crate) version: String,
    pub(crate) data: String,
}

//批量替换并验证参数
#[wasm_bindgen]
pub fn replace_batch(verify_code: &str, params_data: &str) -> Vec<Uint8Array> {
    if let Some(params) = params_decode::<ReplaceParams>(verify_code, params_data) {
        let files = params.get_files();
        let variables = params.get_variables(&vec![]);
        return replace_execute(variables, files);
    }
    vec![]
}

//批量替换并验证参数（多套参数）
#[wasm_bindgen]
pub fn replace_batch_multiple_params(verify_code: &str, params_data: &str) -> Vec<Uint8Array> {
    if let Some(params) = params_decode::<BatchReplaceParams>(verify_code, params_data) {
        let files = params.get_files();
        return replace_execute_multiple_params(params.variables, &vec![], files);
    }
    vec![]
}

//编码替换参数
#[wasm_bindgen]
pub fn replace_params_encode(params: JsValue) -> JsValue {
    params_encode::<ReplaceParams>(params)
}

//编码替换参数（多套参数）
#[wasm_bindgen]
pub fn replace_params_encode_multiple_params(params: JsValue) -> JsValue {
    params_encode::<BatchReplaceParams>(params)
}

//参数编码
fn params_encode<T: serde::de::DeserializeOwned + serde::Serialize>(params: JsValue) -> JsValue {
    let params_data = from_value::<T>(params).unwrap();
    let encoded = encode(&params_data);
    let version = version();
    serde_wasm_bindgen::to_value(&AddReplaceParamsResult {
        version: version.to_string(),
        data: BASE64_STANDARD.encode(encoded),
    }).unwrap()
}

//参数解码
fn params_decode<T: serde::de::DeserializeOwned + serde::Serialize>(verify_code: &str, params_data: &str) -> Option<T> {
    if !verify(verify_code, params_data) {
        return None;
    }
    let data = BASE64_STANDARD.decode(params_data).unwrap();
    let params = decode::<T>(data.as_slice()).unwrap();
    Some(params)
}