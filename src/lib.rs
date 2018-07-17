#![feature(wasm_custom_section, wasm_import_module, proc_macro, nll)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate jsx_types;
use jsx_types::*;

use std::cell::RefCell;

pub mod js_fns;

extern crate serde_json;

thread_local! {
  static ROOT_COMPONENT: RefCell<Option<Box<Component<'static>>>> = RefCell::new(None);
}

pub fn mount(div_id: &str, mut component: Box<dyn Component>) {
  js_fns::initialize(div_id, Interface {});

  //   let token = component.render();

  //   js_fns::render(&token.as_inner_html());
  //   js_fns::log(&token.as_inner_html());

  //   *rc.borrow_mut() = Some(component);
  // });
}

#[wasm_bindgen]
pub struct Interface {}

#[wasm_bindgen]
impl Interface {
  pub fn handle_event(&self, e: &str, path: &str) {
    let path: Vec<usize> = serde_json::from_str(path).unwrap();
    js_fns::log(e);

    let event_name: EventName = e.parse().unwrap();

    // ROOT_COMPONENT.with(|rc| {
    //   let mut root_component = rc.replace(None).expect("ROOT_COMPONENT is missing");

    //   let mut token = root_component.render();
    //   let matched_token = match_token(&mut token, &path);

    //   if let Some(HtmlToken::DomElement(ref mut d)) = matched_token {
    //     d.event_handlers.get_mut(&event_name).map(|handler| {
    //       // handler: &std::boxed::Box<dyn std::ops::FnBox(jsx_types::Event)>
    //       js_fns::log("found handler");
    //       handler(Event {});
    //     });
    //   }

    //   rc.replace(Some(root_component));
    // });
  }
}

fn match_token<'a>(html_token: &'a mut HtmlToken<'a>, path: &[usize]) -> Option<&'a mut HtmlToken<'a>> {
  match path.split_first() {
    None => Some(html_token),
    Some((child_index, rest)) => {
      match html_token {
        HtmlToken::DomElement(d) => {
          // TODO figure out why filtering is not working!!!! WAAH
          match d.children.get_mut(*child_index) {
            Some(child) => match_token(child, rest),
            None => None,
          }
          // d.children
            // .get(*child_index)
            // .and_then(|mut child_token| match_token(&mut child_token, rest))
            // .iter()
            // .filter(|token| {
            //   match token {
            //     HtmlToken::DomElement(_) => true,
            //     _ => false
            //   }
            // })
            // .collect::<Vec<&HtmlToken>>()
            // .remove(*child_index);

          // match_token(&mut child, rest)
            // .and_then(|child_token| match_token(&mut child_token, rest))
        },
        _ => None,
      }
    },
  }
}