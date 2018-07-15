#![feature(wasm_custom_section, wasm_import_module, proc_macro)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate jsx_types;
use jsx_types::*;

use std::cell::RefCell;

pub mod js_fns;

extern crate serde_json;

thread_local! {
  static ROOT_COMPONENT: RefCell<Option<Box<Component>>> = RefCell::new(None);
}

pub fn mount(div_id: &str, mut component: Box<dyn Component + 'static>) {
  js_fns::initialize(div_id, Interface {});

  ROOT_COMPONENT.with(|rc| {
    let token = component.render();

    js_fns::render(&token.as_inner_html());
    js_fns::log(&token.as_inner_html());

    *rc.borrow_mut() = Some(component);
  });
}

#[wasm_bindgen]
pub struct Interface {}

#[wasm_bindgen]
impl Interface {
  pub fn handle_event(&self, e: &str, path: &str) {
    let path: Vec<usize> = serde_json::from_str(path).unwrap();
    js_fns::log(e);

    let event_name: EventName = e.parse().unwrap();

    ROOT_COMPONENT.with(|rc| {
      let mut root_component = rc.replace(None).expect("app_state is missing");

      let token = root_component.render();
      let matched_token = match_token(&token, &path);

      if let Some(HtmlToken::DomElement(d)) = matched_token {
        d.event_handlers.get(&event_name).map(|_handler| {
          js_fns::log("found handler");
          // handler(Event {});
        });
      }

      rc.replace(Some(root_component));
    });
  }
}

fn match_token<'a>(html_token: &'a HtmlToken, path: &[usize]) -> Option<&'a HtmlToken> {
  match path.split_first() {
    None => Some(&html_token),
    Some((child_index, rest)) => {
      match html_token {
        HtmlToken::DomElement(d) => {
          d.children
            .iter()
            .filter(|token| {
              match token {
                HtmlToken::DomElement(_) => true,
                _ => false
              }
            })
            .collect::<Vec<&HtmlToken>>()
            .get(*child_index)
            .and_then(|child_token| match_token(child_token, rest))
        },
        _ => None,
      }
    },
  }
}