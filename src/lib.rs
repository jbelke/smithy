#![feature(wasm_custom_section, wasm_import_module, proc_macro, nll)]

extern crate serde_json;
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate jsx_types;
use jsx_types::{*, events};

use std::cell::RefCell;

pub mod js_fns;

type ShouldUpdate = bool;

thread_local! {
  static ROOT_COMPONENT: RefCell<Option<Box<Component<'static, ()>>>> = RefCell::new(None);
  static LAST_RENDERED_TOKEN: RefCell<Option<jsx_types::bare::BareHtmlToken>> = RefCell::new(None);
}

pub fn mount(div_id: &str, component: Box<dyn for<'a> Component<'a, ()>>) {
  js_fns::initialize(div_id, Interface {});

  let component = unsafe {
    let component_static: Box<dyn Component<'static, ()>> = std::mem::transmute(component);
    component_static
  };
  ROOT_COMPONENT.with(|rc| {
    *rc.borrow_mut() = Some(component);
  });
}

#[wasm_bindgen]
pub struct Interface {}

#[wasm_bindgen]
impl Interface {
  pub fn get_diff(&self) -> String {
    let diff = LAST_RENDERED_TOKEN.with(|rc| {
      let token_opt = rc.replace(None);

      match token_opt {
        Some(old_token) => {
          let new_token = self.render_as_bare_token();
          // N.B. we need to replace the component like:
          let diff = new_token.get_diff_with(&old_token);
          rc.replace(Some(new_token));
          diff
        },
        None => {
          // N.B. this is a weird place to replace the value of rc.
          // TODO replace LAST_RENDERED_TOKEN in a render method...
          rc.replace(Some(self.render_as_bare_token()));
          jsx_types::diff::DiffOperation::initial_diff(&self.get_inner_html())
        }
      }
    });
    
    serde_json::to_string(&diff).unwrap() 
  }

  fn render_as_bare_token(&self) -> jsx_types::bare::BareHtmlToken {
    ROOT_COMPONENT.with(|rc| {
      let component = rc.replace(None).expect("ROOT_COMPONENT is missing");
      let mut component: std::boxed::Box<(dyn for<'a> jsx_types::Component<'a, ()> + 'static)> = unsafe {
        std::mem::transmute(component)
      };
      let (ret, component) = {
        let ret = component.render( ()).as_bare_token();
        (ret, component)
      };
      rc.replace(Some(component));
      ret
    })
  }

  fn get_inner_html(&self) -> String {
    let mut inner_html: String = "".to_string();
    ROOT_COMPONENT.with(|rc| {
      let component = rc.replace(None).expect("ROOT_COMPONENT is missing");
      let mut component: std::boxed::Box<(dyn for<'a> jsx_types::Component<'a, ()> + 'static)> = unsafe {
        std::mem::transmute(component)
      };
      let (inner, component) = {
        let inner_html = component.render( ()).as_inner_html();
        (inner_html, component)
      };
      inner_html = inner;
      rc.replace(Some(component));
    });
    inner_html
  }

  pub fn handle_click(&self, e: &str, path: &str) -> ShouldUpdate {
    let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
    let event: events::MouseEvent = serde_json::from_str(e).expect("Invalid event data");
    let mut should_update = false;

    ROOT_COMPONENT.with(|rc| {
      let component = rc.replace(None).expect("ROOT_COMPONENT is missing");
      let mut component: std::boxed::Box<(dyn for<'a> jsx_types::Component<'a, ()> + 'static)> = unsafe {
        std::mem::transmute(component)
      };

      {
        let token = &mut component.render( ());
        let token_opt = match_token(token, &path);
        if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
          let event_handlers = &mut d.event_handlers;
          if let Some(ref mut handler) = event_handlers.on_click {
            should_update = true;
            handler(&event);
          }
        }
      }
      rc.replace(Some(component));
    });
    return should_update;
  }

  pub fn handle_mouseover(&self, e: &str, path: &str) -> ShouldUpdate {
    let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
    let event: events::MouseEvent = serde_json::from_str(e).expect("Invalid event data");
    let mut should_update = false;

    ROOT_COMPONENT.with(|rc| {
      let component = rc.replace(None).expect("ROOT_COMPONENT is missing");
      let mut component: std::boxed::Box<(dyn for<'a> jsx_types::Component<'a, ()> + 'static)> = unsafe {
        std::mem::transmute(component)
      };

      {
        let token = &mut component.render( ());
        let token_opt = match_token(token, &path);
        if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
          let event_handlers = &mut d.event_handlers;
          if let Some(ref mut handler) = event_handlers.on_mouse_over {
            should_update = true;
            handler(&event);
          }
        }
      }
      rc.replace(Some(component));
    });
    return should_update;
  }

  pub fn handle_mouseout(&self, e: &str, path: &str) -> ShouldUpdate {
    let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
    let event: events::MouseEvent = serde_json::from_str(e).expect("Invalid event data");
    let mut should_update = false;

    ROOT_COMPONENT.with(|rc| {
      let component = rc.replace(None).expect("ROOT_COMPONENT is missing");
      let mut component: std::boxed::Box<(dyn for<'a> jsx_types::Component<'a, ()> + 'static)> = unsafe {
        std::mem::transmute(component)
      };

      {
        let token = &mut component.render( ());
        let token_opt = match_token(token, &path);
        if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
          let event_handlers = &mut d.event_handlers;
          if let Some(ref mut handler) = event_handlers.on_mouse_out {
            should_update = true;
            handler(&event);
          }
        }
      }
      rc.replace(Some(component));
    }); 
    return should_update;
  }

  pub fn handle_input(&self, e: &str, path: &str) -> ShouldUpdate {
    let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
    let event: events::InputEvent = serde_json::from_str(e).expect("Invalid event data");
    let mut should_update = false;

    ROOT_COMPONENT.with(|rc| {
      let component = rc.replace(None).expect("ROOT_COMPONENT is missing");
      let mut component: std::boxed::Box<(dyn for<'a> jsx_types::Component<'a, ()> + 'static)> = unsafe {
        std::mem::transmute(component)
      };

      {
        let token = &mut component.render( ());
        let token_opt = match_token(token, &path);
        if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
          let event_handlers = &mut d.event_handlers;
          if let Some(ref mut handler) = event_handlers.on_input {
            should_update = true;
            handler(&event);
          }
        }
      }
      rc.replace(Some(component));
    });
    return should_update;
  }

  pub fn handle_keydown(&self, e: &str, path: &str) -> ShouldUpdate {
    let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
    let event: events::KeyboardEvent = serde_json::from_str(e).expect("Invalid event data");
    let mut should_update = false;

    ROOT_COMPONENT.with(|rc| {
      let component = rc.replace(None).expect("ROOT_COMPONENT is missing");
      let mut component: std::boxed::Box<(dyn for<'a> jsx_types::Component<'a, ()> + 'static)> = unsafe {
        std::mem::transmute(component)
      };

      {
        let token = &mut component.render( ());
        let token_opt = match_token(token, &path);
        if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
          let event_handlers = &mut d.event_handlers;
          if let Some(ref mut handler) = event_handlers.on_keydown {
            should_update = true;
            handler(&event);
          }
        }
      }
      rc.replace(Some(component));
    });
    return should_update;
  }
 
}



fn match_token<'a, 'b: 'a>(html_token: &'a mut HtmlToken<'b>, path: &[usize]) -> Option<&'a mut HtmlToken<'b>> {
  match path.split_first() {
    None => Some(html_token),
    Some((child_index, rest)) => {
      match html_token {
        HtmlToken::DomElement(d) => {
          match d.children.get_mut(*child_index) {
            Some(child) => match_token(child, rest),
            None => None
          }
        },
        _ => None,
      }
    },
  }
}