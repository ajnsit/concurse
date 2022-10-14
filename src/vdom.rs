use std::{collections::HashMap};

use crate::host::{Host, Node};

type Attrs<'a> = HashMap<&'a str, &'a str>;

#[derive(Debug)]
pub(crate) enum VDom<'a, Children, State>
  { Text {text: &'a str, state: State}
  , Elem {name: &'a str, attrs: Attrs<'a>, children: Children, state: State}
  }

pub(crate) struct VDomTree<'a>
  { vdom: VDom<'a, Vec<VDomTree<'a>>, ()>
  }

pub(crate) struct VDomMachine<'a>
  { vdom: VDom<'a, Vec<VDomMachine<'a>>, Node>
  }

impl<'a> VDomMachine<'a> {
  pub(crate) fn get_node(vdom: &'a VDom<'a, Vec<VDomMachine<'a>>, Node>) -> &Node {
    match vdom {
      VDom::Text { state, .. } => state,
      VDom::Elem { state, .. } => state,
    }
  }

  pub(crate) fn build_vdom(host: &Host, input: VDom<'a, Vec<VDomTree<'a>>, ()>) -> VDom<'a, Vec<VDomMachine<'a>>, Node> {
    match input {
        VDom::Text { text, .. } => {
          let node = host.create_text_node(text);
          VDom::Text{text, state: node}
        },
        VDom::Elem { name, attrs, children: children1, .. } => {
          let node = host.create_element(name);
          attrs.iter().for_each(|(key,val)| node.set_attribute(key, val) );
          let children =
            children1.into_iter().map(|child| {
              let child_vdom = VDomMachine::build_vdom(host, child.vdom);
              let child_node = VDomMachine::get_node(&child_vdom);
              // TODO: Insert at the correct index
              node.insert_child_ix(0, child_node);
              VDomMachine { vdom: child_vdom }
          }).collect();
          VDom::Elem { name, attrs, children, state: node }
        },
    }
  }

  pub(crate) fn rebuild_dom(&mut self, host: &Host, input: VDom<'a, Vec<VDomTree<'a>>, ()>) {
    self.vdom = Self::build_vdom(host, input);
  }

  pub(crate) fn halt_and_rebuild(&mut self, host: &Host, input: VDom<'a, Vec<VDomTree<'a>>, ()>) {
    self.halt();
    self.rebuild_dom(host, input);
  }

  pub(crate) fn step(&mut self, host: &Host, input: VDom<'a, Vec<VDomTree<'a>>, ()>) {
    match input {
      VDom::Text { text: tnew, .. } => {
        match &mut self.vdom {
          VDom::Text { text: told, state: node } =>
            if tnew != *told {
              node.set_text_content(tnew);
              *told = tnew;
            },
          VDom::Elem { .. } => {
            self.rebuild_dom(host, input);
          },
        }
      },
      VDom::Elem { name: name_new, attrs: attrs_new, children: children_new, .. } => {
        let mut vdom_temp = VDomMachine {vdom: VDom::Text { text: "DELME", state: Node::dummyNode() } };
        std::mem::swap(self, &mut vdom_temp);
        match vdom_temp.vdom {
          VDom::Elem { name: name_old, attrs: attrs_old, children: children_old, state: mut node } => {
            if name_new == name_old {
              update_attrs(&mut node, &attrs_old, &attrs_new);
              if children_old.len() != 0 || children_new.len() != 0 {
                let new_children = update_children(host, &mut node, children_old, children_new);
                vdom_temp.vdom = VDom::Elem { name: name_old, attrs: attrs_new, children: new_children, state: node };
                std::mem::swap(self, &mut vdom_temp);
              }
            } else {
              self.rebuild_dom(host, VDom::Elem { name: name_old, attrs: attrs_new, children: children_new, state: () });
            }
          },
          VDom::Text { text, .. } => {
            self.rebuild_dom(host, VDom::Text { text, state: () });
          },
        }
      },
    }
  }


  pub(crate) fn halt(&mut self) {
    match &mut self.vdom {
      VDom::Text { state: node , .. } => {
        let parent = node.parent_node();
        parent.remove_child(&node);
      },
      VDom::Elem { children, state: node , .. } => {
        let parent = node.parent_node();
        parent.remove_child(&node);
        children.iter_mut().for_each(|x| { x.halt(); });
        // attrs.halt();
      },
    }
  }
}

fn update_attrs(node: &mut Node, attrs_old: &Attrs, attrs_new: &Attrs) {
  // TODO: Do a more efficient diff and update attrs on the node?
  attrs_new.into_iter().for_each(|(key,val)| {
    match attrs_old.get(key) {
      None => node.set_attribute(key, val),
      Some(val_old) => if val != val_old { node.set_attribute(key, val) },
    }
  });
  attrs_old.into_iter().for_each(|(key, _)| {
    match attrs_new.get(key) {
        None => node.remove_attribute(key),
        Some(_) => (),
    }
  });
}

fn update_children<'a>(host: &Host, parent: &mut Node, children_old: Vec<VDomMachine<'a>>, children_new: Vec<VDomTree<'a>>) -> Vec<VDomMachine<'a>> {
  // TODO: Currently only unkeyed is supported as a POC
  let mut ret = Vec::new();
  let mut children_old_iter = children_old.into_iter();
  let mut children_new_iter = children_new.into_iter();
  loop {
    match children_old_iter.next() {
      Some(mut child_old) => {
        match children_new_iter.next() {
          Some(child_new) => {
            child_old.step(host, child_new.vdom);
            ret.push(child_old);
          }
          None => {
            child_old.halt();
          },
        }
      }
      None => {
        match children_new_iter.next() {
          Some(child_new) => {
            let child_vdom = VDomMachine::build_vdom(host, child_new.vdom);
            let child_node = VDomMachine::get_node(&child_vdom);
            // TODO: Insert at the correct index
            parent.insert_child_ix(0, child_node);
            ret.push(VDomMachine { vdom: child_vdom });
          }
          None => break,
        }
      }
    }
  }
  ret
}