#![feature(wasm_custom_section, wasm_import_module, proc_macro, nll)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate jsx_types;
use jsx_types::{*, events};

use std::cell::RefCell;

pub mod js_fns;

extern crate serde_json;

thread_local! {
  static ROOT_COMPONENT: RefCell<Option<Box<Component<'static>>>> = RefCell::new(None);
}

pub fn mount(div_id: &str, component: Box<dyn for<'a> Component<'a>>) {
  js_fns::initialize(div_id, Interface {});

  let component = unsafe {
    let component_static: Box<dyn Component<'static>> = std::mem::transmute(component);
    component_static
  };
  ROOT_COMPONENT.with(|rc| {
    *rc.borrow_mut() = Some(component);
  });
}

fn get_inner_html_from_component(mut component: Box<dyn for<'a> Component<'a>>) -> (String, Box<dyn for<'a> Component<'a>>) {
  let inner_html = component.render().as_inner_html();
  (inner_html, component)
}

#[wasm_bindgen]
pub struct Interface {}

#[wasm_bindgen]
impl Interface {
  pub fn get_inner_html(&self) -> String {
    let mut inner_html: String = "".to_string();
    ROOT_COMPONENT.with(|rc| {
      let component = rc.replace(None).expect("ROOT_COMPONENT is missing");
      let component: std::boxed::Box<(dyn for<'a> jsx_types::Component<'a> + 'static)> = unsafe {
        std::mem::transmute(component)
      };
      let (inner, component) = get_inner_html_from_component(component);
      inner_html = inner;
      rc.replace(Some(component));
    });
    inner_html
  }

  pub fn handle_click(&self, e: &str, path: &str) {
    let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
    let event: events::OnClickEvent = serde_json::from_str(e).expect("Invalid event data");

    ROOT_COMPONENT.with(|rc| {
      let component = rc.replace(None).expect("ROOT_COMPONENT is missing");
      let mut component: std::boxed::Box<(dyn for<'a> jsx_types::Component<'a> + 'static)> = unsafe {
        std::mem::transmute(component)
      };

      {
        let token = &mut component.render();
        let token_opt = match_token(token, &path);
        if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
          let event_handlers = &mut d.event_handlers;
          if let Some(ref mut handler) = event_handlers.on_click {
            handler(&event);
          }
        }
      }
      rc.replace(Some(component));
    });
  }
}

fn match_token<'a, 'b: 'a>(html_token: &'a mut HtmlToken<'b>, path: &[usize]) -> Option<&'a mut HtmlToken<'b>> {
  match path.split_first() {
    None => Some(html_token),
    Some((child_index, rest)) => {
      match html_token {
        HtmlToken::DomElement(d) => {
          // N.B. I could not get a more straightforward filter to work :(
          let mut count: i32 = -1;
          let real_index_opt = d.children.iter()
            .position(|child| {
              match child {
                HtmlToken::DomElement(_) => {
                  count += 1;
                  &(count as usize) == child_index
                },
                _ => false
              }
            });

          real_index_opt
            .and_then(move |index| {
              match d.children.get_mut(index) {
                Some(child) => match_token(child, rest),
                None => None,
              }
            })
        },
        _ => None,
      }
    },
  }
}