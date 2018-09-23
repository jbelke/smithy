#![feature(nll)]

extern crate serde_json;
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate jsx_types;
extern crate web_sys;
extern crate js_sys;

use jsx_types::*;

use std::cell::RefCell;
use std::mem::transmute;

mod with_inner_value;
use with_inner_value::*;
mod dom;
use dom::{
  get_document,
  get_path_from,
  find_token_by_path,
  apply_diff,
};

use web_sys::{
  Element,
  Document,
  MouseEvent,
  HtmlElement,
  EventTarget,
  Event,
};

pub mod js_fns;

type ComponentAlt = Box<dyn for<'a> Component<'a, ()>>;

thread_local! {
  static LAST_RENDERED_TOKEN: RefCell<Option<jsx_types::bare::BareHtmlToken>> = RefCell::new(None);
  static ROOT_COMPONENT: RefCell<Option<ComponentAlt>> = RefCell::new(None);
  static ROOT_ELEMENT: RefCell<Option<HtmlElement>> = RefCell::new(None);
}

fn mount_to_element(el: &Element, mut component: ComponentAlt) {
  {
    let token = component.render(());
    el.set_inner_html(&token.as_inner_html());
    LAST_RENDERED_TOKEN.store(token.as_bare_token());
  }
  ROOT_COMPONENT.store(component);
}

fn store_root_element(el: Element) {
  let html_el = unsafe {
    transmute::<Element, HtmlElement>(el)
  };
  ROOT_ELEMENT.store(html_el);
}

fn attach_listeners(el: &Element) {
  let html_el = unsafe {
    transmute::<&Element, &js_fns::HTMLElement>(el)
  };

  let on_click_cb = Closure::new(|e: MouseEvent| {
    let event = unsafe {
      transmute::<MouseEvent, Event>(e)
    };
    if let Some(t) = event.target() {
      let target_html_el: HtmlElement = unsafe {
        transmute::<EventTarget, HtmlElement>(t)
      };
      ROOT_ELEMENT.with_inner_value(|root_element| {
        let path = get_path_from(root_element, &target_html_el);

        let diff_opt = ROOT_COMPONENT.with_inner_value(|root_component| {
          let should_rerender = {
            let mut top_level_token: HtmlToken = root_component.render(());
            match find_token_by_path(&mut top_level_token, &path) {
              Some(target_token) => {
                if let HtmlToken::DomElement(d) = target_token {
                  if let Some(ref mut on_click) = d.event_handlers.on_click {
                    let mouse_event = unsafe {
                      transmute::<&Event, &MouseEvent>(&event)
                    };
                    on_click(&mouse_event);
                    true
                  } else { false }
                } else { false }
              },
              None => { false },
            }
          };
          if should_rerender {
            // we've updated things due to the call to on_click!
            let new_token = root_component.render(()).as_bare_token();
            let diff = LAST_RENDERED_TOKEN.with_inner_value(|old_token| {
              let diff = new_token.get_diff_with(old_token);
              diff
            });
            LAST_RENDERED_TOKEN.store(new_token);
            Some(diff)
          } else { None }
        });

        if let Some(diff) = diff_opt {
          apply_diff(root_element, diff);
        }
      });
    }
  });
  html_el.set_onclick(&on_click_cb);
  on_click_cb.forget();
}

pub fn mount(div_id: &str, component: ComponentAlt) {
  let doc: Document = get_document();
  let el_opt: Option<Element> = doc.get_element_by_id(div_id);
  
  match el_opt {
    Some(el) => {
      mount_to_element(&el, component);
      attach_listeners(&el);
      store_root_element(el);
    },
    None => {
      js_fns::log_error(&format!("Smithy failed to mount! Element with id {} not found.", div_id));
    },
  };
}
