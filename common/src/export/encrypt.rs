use js_sys::Uint8Array;
use wasm_bindgen::prelude::wasm_bindgen;

//文件加密
#[wasm_bindgen]
pub fn file_encrypt(file: Uint8Array) -> Vec<u8> {
    let data = file.to_vec();
    file_encode(data)
}

//文件批量加密
#[wasm_bindgen]
pub fn files_encrypt(files: Vec<Uint8Array>) -> Vec<Uint8Array> {
    let mut result = vec![];
    for file in files {
        let data = file.to_vec();
        let uint8array = Uint8Array::from(file_encode(data).as_slice());
        result.push(uint8array);
    }
    result
}

//文件加密
pub fn file_encode(mut data: Vec<u8>) -> Vec<u8> {
    let input = data.as_mut_slice();
    crate::encrypt::encrypt::encode(input);
    input.to_vec()
}

//文件解密
pub fn file_decode(mut data: Vec<u8>) -> Vec<u8> {
    let input = data.as_mut_slice();
    crate::encrypt::encrypt::decode(input);
    input.to_vec()
}