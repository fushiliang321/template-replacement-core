#[macro_use]
extern crate console_log;
pub mod extract;
pub mod office;
pub mod replace;
mod authorization;
pub mod encrypt;
pub mod export;

use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::*;

static VERSION: OnceCell<&str> = OnceCell::new();

// 获取版本号
pub(crate) fn version() -> &'static str {
    VERSION.get_or_init(|| {
        env!("CARGO_PKG_VERSION")
    })
}

#[wasm_bindgen(start)]
pub fn main() {
    // 初始化代码
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logger");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
