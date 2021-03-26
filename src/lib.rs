extern crate pest;
#[macro_use]
extern crate pest_derive;

mod dictionary_en;
mod ingredient_en;
mod utils;
mod vulgar_fractions;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn parse(str: &str) -> Result<JsValue, JsValue> {
    let info = ingredient_en::parse(str);
    serde_wasm_bindgen::to_value(&info).map_err(|err| err.into())
}
