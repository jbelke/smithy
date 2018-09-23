use jsx_types::{*, diff};

use std::mem::transmute;
use web_sys::{
  Element,
  Document,
  Window,
  HtmlElement,
  HtmlTemplateElement,
  Node,
  DocumentFragment,
};

pub fn get_document() -> Document { Window::document().unwrap() }

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

pub fn get_path_from(root_element: &HtmlElement, target_element: &HtmlElement) -> Vec<usize> {
  // TODO this method can be cleaned up a bit, especially around the return vec![] lines
  let root_node: &Node = unsafe {
    transmute::<&HtmlElement, &Node>(root_element)
  };
  let target_node: &Node = unsafe {
    transmute::<&HtmlElement, &Node>(target_element)
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

  panic!("");
}

fn convert_node_reference_to_owned(node: &Node) -> Node {
  // N.B. this method smells and should be gotten rid of and never ever used.
  let parent = node.parent_node().unwrap();
  let index_to_root = find_child_index(&parent, node);
  parent.child_nodes().get(index_to_root as u32).unwrap()
}

pub fn find_node_by_path(root_node: &Node, path: &[usize]) -> Option<Node> {
  path.iter().fold(Some(convert_node_reference_to_owned(root_node)), |node_opt, path_segment| {
    node_opt.and_then(|node| node.child_nodes().get(*path_segment as u32))
  })
}

pub fn find_token_by_path<'a, 'b: 'a>(
  html_token: &'a mut HtmlToken<'b>,
  path: &[usize]
) -> Option<&'a mut HtmlToken<'b>> {
  match path.split_first() {
    None => Some(html_token),
    Some((child_index, rest)) => {
      match html_token {
        HtmlToken::DomElement(d) => {
          match d.children.get_mut(*child_index) {
            Some(child) => find_token_by_path(child, rest),
            None => None
          }
        },
        _ => None,
      }
    },
  }
}

pub fn apply_diff(root_el: &HtmlElement, diff: diff::Diff) {
  let root_node = unsafe {
    transmute::<&HtmlElement, &Node>(root_el)
  };
  for (path, op) in diff {
    match op {
      diff::DiffOperation::Replace(replace_operation) => {
        let (last_segment, path_to_parent) = path.split_last().unwrap();
        let parent = find_node_by_path(root_node, path_to_parent).unwrap();
        let original_child = parent.child_nodes().get(*last_segment as u32).unwrap();
        let new_node = get_node_from_inner_html(&replace_operation.new_inner_html);
        let _ = parent.replace_child(&new_node, &original_child);
      },
      diff::DiffOperation::Insert(insert_operation) => {
        let (last_segment, path_to_parent) = path.split_last().unwrap();
        let parent = find_node_by_path(root_node, path_to_parent).unwrap();
        let new_node = get_node_from_inner_html(&insert_operation.new_inner_html);
        let child_opt = parent.child_nodes().get(*last_segment as u32);
        // N.B. this is because parent.insert_before(&new_node, child_opt) gives me
        // note: expected type `std::option::Option<&web_sys::Node>`
        // found type `&std::option::Option<web_sys::Node>`
        let _ = match child_opt {
          Some(child) => parent.insert_before(&new_node, Some(&child)),
          None => parent.insert_before(&new_node, None)
        };
      },
      diff::DiffOperation::Delete(_) => {
        let (last_segment, path_to_parent) = path.split_last().unwrap();
        let parent = find_node_by_path(root_node, path_to_parent).unwrap();
        let original_child = parent.child_nodes().get(*last_segment as u32).unwrap();
        let _ = parent.remove_child(&original_child);
      },
      diff::DiffOperation::UpdateAttributes(update_attributes_operation) => {
        let node = find_node_by_path(root_node, &path).unwrap();
        let element = unsafe {
          transmute::<Node, Element>(node)
        };
        for (attribute, value) in update_attributes_operation.new_attributes {
          let _ = element.set_attribute(&attribute, &value);
        }
      },
    };
  }
}

fn get_node_from_inner_html(inner_html: &str) -> Node {
  let new_el = get_document().create_element("template").unwrap();
  new_el.set_inner_html(inner_html);
  let new_template = unsafe {
    transmute::<Element, HtmlTemplateElement>(new_el)
  };
  let content_document_fragment = new_template.content();
  let content_node = unsafe {
    transmute::<DocumentFragment, Node>(content_document_fragment)
  };
  content_node.first_child().unwrap()
}
