extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

use web_sys::MouseEvent;

#[wasm_bindgen]
extern {
  #[wasm_bindgen]
  pub fn alert(s: &str);

  #[wasm_bindgen(js_namespace = console, js_name=log)]
  pub fn log(msg: &str);

  #[wasm_bindgen(js_namespace = console, js_name=error)]
  pub fn log_error(msg: &str);

  #[wasm_bindgen(js_name = setInterval)]
  pub fn set_interval(cb: &Closure<FnMut()>, delay: u32) -> f64;

  pub type HTMLElement;
  #[wasm_bindgen(method, setter)]
  pub fn set_onclick(this: &HTMLElement, cb: &Closure<FnMut(MouseEvent)>);
}
