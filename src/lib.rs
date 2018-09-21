#![feature(nll)]

extern crate serde_json;
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate jsx_types;
extern crate web_sys;
extern crate js_sys;

use jsx_types::{*, events, bare};
use bare::BareHtmlToken;

use std::cell::RefCell;

mod util;
use util::*;

use web_sys::{Element, Document, Window, MouseEvent, HtmlElement, EventTarget, Event, Node};

pub mod js_fns;

type ShouldUpdate = bool;
type ComponentAlt = Box<dyn for<'a> Component<'a, ()>>;

thread_local! {
  // for diff's only
  static LAST_RENDERED_TOKEN: RefCell<Option<jsx_types::bare::BareHtmlToken>> = RefCell::new(None);
  static ROOT_COMPONENT: RefCell<Option<ComponentAlt>> = RefCell::new(None);
  static ROOT_ELEMENT: RefCell<Option<HtmlElement>> = RefCell::new(None);
}

fn get_document() -> Document { Window::document().unwrap() }

fn mount_to_element(el: &Element, mut component: ComponentAlt) {
  {
    let token = component.render(());
    js_fns::log(&format!("bout to render {:?}", token));
    el.set_inner_html(&token.as_inner_html());
  }
  ROOT_COMPONENT.store(component);
}

fn store_root_element(el: Element) {
  let html_el = unsafe {
    std::mem::transmute::<Element, HtmlElement>(el)
  };
  ROOT_ELEMENT.store(html_el);
}

fn find_child_index(parent: &Node, child: &Node) -> usize {
  let node_list = parent.child_nodes();
  for i in 0..node_list.length() {
    if let Some(node) = node_list.get(i) {
      if node.is_same_node(Some(&child)) {
        return i as usize;
      }
    }
  }
  panic!("child not found in parent");
}

fn match_token<'a, 'b: 'a>(
  html_token: &'a mut HtmlToken<'b>,
  path: &[usize]
) -> Option<&'a mut HtmlToken<'b>> {
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

fn get_path_from(root_element: &HtmlElement, target_element: &HtmlElement) -> Vec<usize> {
  // TODO this method can be cleaned up a bit, especially around the return vec![] lines
  js_fns::log(&format!(
    "get path from root =\n{}\ntarget =\n{}",
    root_element.inner_text(),
    target_element.inner_text()
  ));
  let root_node: &Node = unsafe {
    std::mem::transmute::<&HtmlElement, &Node>(root_element)
  };
  let target_node: &Node = unsafe {
    std::mem::transmute::<&HtmlElement, &Node>(target_element)
  };

  // special case, we clicked on the <div id="app" /> node,
  // but that shouldn't be possible unless that node has some padding
  if target_node.is_same_node(Some(&root_node)) {
    return vec![];
  }

  let mut current_node = target_node.parent_node().unwrap();

  if current_node.is_same_node(Some(&root_node)) {
    return vec![];
  }

  let mut path = vec![find_child_index(&current_node, target_node)];
  while let Some(parent) = current_node.parent_node() {
    if parent.is_same_node(Some(&root_node)) {
      return path;
    }
    path.push(find_child_index(&parent, &current_node));
    current_node = parent;
  }

  js_fns::log(&format!("did not get a successful path, got path {:?}", path));
  panic!("");
}

fn attach_listeners(el: &Element) {
  let html_el = unsafe {
    std::mem::transmute::<&Element, &js_fns::HTMLElement>(el)
  };

  let on_click_cb = Closure::new(|e: MouseEvent| {
    let event = unsafe {
      std::mem::transmute::<MouseEvent, Event>(e)
    };
    if let Some(t) = event.target() {
      let target_html_el: HtmlElement = unsafe {
        std::mem::transmute::<EventTarget, HtmlElement>(t)
      };
      ROOT_ELEMENT.with_inner_value(|root_element| {
        let path = get_path_from(root_element, &target_html_el);
        js_fns::log(&format!("on click cb: path = {:?}", path));

        ROOT_COMPONENT.with_inner_value(|root_component| {
          let mut top_level_token: HtmlToken = root_component.render(());
          match match_token(&mut top_level_token, &path) {
            Some(target_token) => {
              js_fns::log(&format!("found target {}", target_token.as_inner_html()));

              if let HtmlToken::DomElement(d) = target_token {
                if let Some(ref mut on_click) = d.event_handlers.on_click {
                  let mouse_event = unsafe {
                    std::mem::transmute::<&Event, &MouseEvent>(&event)
                  };
                  js_fns::log("bout to click it");
                  on_click(&mouse_event);
                  // TODO re-render
                }
              }
            },
            None => { js_fns::log("DID NOT FIND you fail at life"); },
          }
        });
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

// #[wasm_bindgen]
// pub struct Interface {}

// #[wasm_bindgen]
// impl Interface {
//   pub fn get_diff(&self) -> String {
//     let diff = LAST_RENDERED_TOKEN.with(|rc| {
//       let token_opt = rc.replace(None);

//       match token_opt {
//         Some(old_token) => {
//           let new_token = self.render_as_bare_token();
//           // N.B. we need to replace the component like:
//           let diff = new_token.get_diff_with(&old_token);
//           rc.replace(Some(new_token));
//           diff
//         },
//         None => {
//           // N.B. this is a weird place to replace the value of rc.
//           // TODO replace LAST_RENDERED_TOKEN in a render method...
//           rc.replace(Some(self.render_as_bare_token()));
//           jsx_types::diff::DiffOperation::initial_diff(&self.get_inner_html())
//         }
//       }
//     });
    
//     serde_json::to_string(&diff).unwrap() 
//   }

//   fn render_as_bare_token(&self) -> BareHtmlToken {
//     ROOT_COMPONENT.with(|rc| {
//       let mut component = rc.replace(None).expect("ROOT_COMPONENT is missing");
//       let (ret, component) = {
//         let ret = component.render( ()).as_bare_token();
//         (ret, component)
//       };
//       rc.replace(Some(component));
//       ret
//     })
//   }

//   fn get_inner_html(&self) -> String {
//     let mut inner_html: String = "".to_string();
//     ROOT_COMPONENT.with(|rc| {
//       let mut component = rc.replace(None).expect("ROOT_COMPONENT is missing");
//       let (inner, component) = {
//         let inner_html = component.render(()).as_inner_html();
//         (inner_html, component)
//       };
//       inner_html = inner;
//       rc.replace(Some(component));
//     });
//     inner_html
//   }

//   pub fn handle_click(&self, e: &str, path: &str) -> ShouldUpdate {
//     let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
//     let event: events::MouseEvent = serde_json::from_str(e).expect("Invalid event data");
//     let mut should_update = false;

//     ROOT_COMPONENT.with(|rc| {
//       let mut component = rc.replace(None).expect("ROOT_COMPONENT is missing");

//       {
//         let token = &mut component.render(());
//         let token_opt = match_token(token, &path);
//         if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
//           let event_handlers = &mut d.event_handlers;
//           if let Some(ref mut handler) = event_handlers.on_click {
//             should_update = true;
//             handler(&event);
//           }
//         }
//       }
//       rc.replace(Some(component));
//     });
//     return should_update;
//   }

//   pub fn handle_mouseover(&self, e: &str, path: &str) -> ShouldUpdate {
//     let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
//     let event: events::MouseEvent = serde_json::from_str(e).expect("Invalid event data");
//     let mut should_update = false;

//     ROOT_COMPONENT.with(|rc| {
//       let mut component = rc.replace(None).expect("ROOT_COMPONENT is missing");
//       {
//         let token = &mut component.render( ());
//         let token_opt = match_token(token, &path);
//         if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
//           let event_handlers = &mut d.event_handlers;
//           if let Some(ref mut handler) = event_handlers.on_mouse_over {
//             should_update = true;
//             handler(&event);
//           }
//         }
//       }
//       rc.replace(Some(component));
//     });
//     return should_update;
//   }

//   pub fn handle_mouseout(&self, e: &str, path: &str) -> ShouldUpdate {
//     let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
//     let event: events::MouseEvent = serde_json::from_str(e).expect("Invalid event data");
//     let mut should_update = false;

//     ROOT_COMPONENT.with(|rc| {
//       let mut component = rc.replace(None).expect("ROOT_COMPONENT is missing");

//       {
//         let token = &mut component.render( ());
//         let token_opt = match_token(token, &path);
//         if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
//           let event_handlers = &mut d.event_handlers;
//           if let Some(ref mut handler) = event_handlers.on_mouse_out {
//             should_update = true;
//             handler(&event);
//           }
//         }
//       }
//       rc.replace(Some(component));
//     }); 
//     return should_update;
//   }

//   pub fn handle_input(&self, e: &str, path: &str) -> ShouldUpdate {
//     let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
//     let event: events::InputEvent = serde_json::from_str(e).expect("Invalid event data");
//     let mut should_update = false;

//     ROOT_COMPONENT.with(|rc| {
//       let mut component = rc.replace(None).expect("ROOT_COMPONENT is missing");

//       {
//         let token = &mut component.render( ());
//         let token_opt = match_token(token, &path);
//         if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
//           let event_handlers = &mut d.event_handlers;
//           if let Some(ref mut handler) = event_handlers.on_input {
//             should_update = true;
//             handler(&event);
//           }
//         }
//       }
//       rc.replace(Some(component));
//     });
//     return should_update;
//   }

//   pub fn handle_keydown(&self, e: &str, path: &str) -> ShouldUpdate {
//     let path: Vec<usize> = serde_json::from_str(path).expect("Invalid path");
//     let event: events::KeyboardEvent = serde_json::from_str(e).expect("Invalid event data");
//     let mut should_update = false;

//     ROOT_COMPONENT.with(|rc| {
//       let mut component = rc.replace(None).expect("ROOT_COMPONENT is missing");

//       {
//         let token = &mut component.render( ());
//         let token_opt = match_token(token, &path);
//         if let Some(HtmlToken::DomElement(ref mut d)) = token_opt {
//           let event_handlers = &mut d.event_handlers;
//           if let Some(ref mut handler) = event_handlers.on_keydown {
//             should_update = true;
//             handler(&event);
//           }
//         }
//       }
//       rc.replace(Some(component));
//     });
//     return should_update;
//   }
 
// }
