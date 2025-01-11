extern crate common;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

//批量替换并验证参数
#[wasm_bindgen]
pub async fn replace_batch_verify(verify_code: String, params_data: String) -> Vec<Uint8Array> {
    common::common::replace_batch_verify(verify_code, params_data).await
}

//编码替换参数
#[wasm_bindgen]
pub async fn replace_params_encode(params: JsValue) -> JsValue {
    common::common::replace_params_encode(params).await
}

#[wasm_bindgen]
pub async fn add_word(file: Uint8Array) -> u32 {
    common::common::add_word(file).await
}

#[wasm_bindgen]
pub async fn add_excel(file: Uint8Array) -> u32 {
    common::common::add_excel(file).await
}

#[wasm_bindgen]
pub async fn add_media(file: Uint8Array) -> String {
    common::common::add_media(file).await
}