#![feature(wasm_custom_section, wasm_import_module, proc_macro)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate jsx_types;
use jsx_types::*;

use std::cell::RefCell;

pub mod js_fns;
mod app_state;
use self::app_state::*;

extern crate serde_json;

thread_local! {
  static APP_STATE: RefCell<Option<AppState>> = RefCell::new(None);
}

pub fn mount(div_id: &str, component: Box<Component>) {
  let app_state: AppState = AppState {
    top_level_component: component,
  };

  js_fns::initialize(div_id, Interface {});

  APP_STATE.with(|rc| {
    let token = app_state.top_level_component.render();

    js_fns::render(&token.as_inner_html());
    js_fns::log(&token.as_inner_html());

    *rc.borrow_mut() = Some(app_state);
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

    APP_STATE.with(|rc| {
      let mut app_state = rc.replace(None).expect("app_state is missing");

      let mut token = app_state.top_level_component.render();
      let mut matched_token = match_token(&token, &path);

      if let Some(HtmlToken::DomElement(d)) = matched_token {
        d.event_handlers.get(&event_name).map(|handler| {
          js_fns::log("found handler");
          // handler(Event {});
        });
      }

      rc.replace(Some(app_state));
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