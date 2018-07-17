extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

use super::Interface;

use jsx_types::*;

#[wasm_bindgen]
extern {
  #[wasm_bindgen]
  pub fn alert(s: &str);

  #[wasm_bindgen(js_namespace = console, js_name=log)]
  pub fn log(msg: &str);

  #[wasm_bindgen(module = "./lib")]
  pub fn initialize(id: &str, interface: Interface);

  #[wasm_bindgen(module = "./lib")]
  pub fn render(inner_html: &str);
}
