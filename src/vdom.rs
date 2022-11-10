use std::collections::HashMap;

use crate::host::{Host, Node};

type Attrs = HashMap<String, String>;

#[derive(Debug)]
pub(crate) enum VDom<Children, State> {
    Text {
        text: String,
        state: State,
    },
    Elem {
        name: String,
        attrs: Attrs,
        children: Children,
        state: State,
    },
}

pub(crate) struct VDomTree {
    pub(crate) vdom: VDom<Vec<VDomTree>, ()>,
}

pub(crate) struct VDomMachine {
    pub(crate) vdom: VDom<Vec<VDomMachine>, Node>,
}

impl VDomMachine {
    pub(crate) fn install(&self, host: &Host) {}

    pub(crate) fn get_node(vdom: &VDom<Vec<VDomMachine>, Node>) -> &Node {
        match vdom {
            VDom::Text { state, .. } => state,
            VDom::Elem { state, .. } => state,
        }
    }

    pub(crate) fn build_vdom_machine(host: &Host, input: &VDomTree) -> VDomMachine {
        let vdom = VDomMachine::build_vdom(host, &input.vdom);
        VDomMachine { vdom }
    }

    pub(crate) fn build_vdom(
        host: &Host,
        input: &VDom<Vec<VDomTree>, ()>,
    ) -> VDom<Vec<VDomMachine>, Node> {
        match input {
            VDom::Text { text, .. } => {
                let node = host.create_text_node(text);
                VDom::Text {
                    text: text.clone(),
                    state: node,
                }
            }
            VDom::Elem {
                name,
                attrs,
                children: children1,
                ..
            } => {
                let node = host.create_element(name);
                attrs
                    .iter()
                    .for_each(|(key, val)| node.set_attribute(key, val));
                let children = children1
                    .into_iter()
                    .map(|child| {
                        let child_vdom = VDomMachine::build_vdom(host, &child.vdom);
                        let child_node = VDomMachine::get_node(&child_vdom);
                        node.append_child(child_node);
                        VDomMachine { vdom: child_vdom }
                    })
                    .collect();
                VDom::Elem {
                    name: name.clone(),
                    attrs: attrs.clone(),
                    children,
                    state: node,
                }
            }
        }
    }

    pub(crate) fn rebuild_dom(&mut self, host: &Host, input: &VDom<Vec<VDomTree>, ()>) {
        self.vdom = Self::build_vdom(host, input);
    }

    pub(crate) fn halt_and_rebuild(&mut self, host: &Host, input: &VDom<Vec<VDomTree>, ()>) {
        self.halt();
        self.rebuild_dom(host, input);
    }

    pub(crate) fn step(&mut self, host: &Host, input: &VDom<Vec<VDomTree>, ()>) {
        match input {
            VDom::Text { text: tnew, .. } => match &mut self.vdom {
                VDom::Text {
                    text: told,
                    state: node,
                } => {
                    if tnew != told {
                        node.set_text_content(tnew);
                        *told = tnew.clone();
                    }
                }
                VDom::Elem { .. } => {
                    self.rebuild_dom(host, input);
                }
            },
            VDom::Elem {
                name: name_new,
                attrs: attrs_new,
                children: children_new,
                ..
            } => {
                match std::mem::replace(
                    &mut self.vdom,
                    VDom::Text {
                        text: String::new(),
                        state: Node::dummy_node(),
                    },
                ) {
                    VDom::Elem {
                        name: name_old,
                        attrs: attrs_old,
                        children: mut children_old,
                        state: mut node,
                    } => {
                        if *name_new == name_old {
                            update_attrs(&mut node, &attrs_old, &attrs_new);
                            if children_old.len() != 0 || children_new.len() != 0 {
                                update_children(host, &mut node, &mut children_old, children_new);
                                self.vdom = VDom::Elem {
                                    name: name_old,
                                    attrs: attrs_new.clone(),
                                    children: children_old,
                                    state: node,
                                };
                            }
                        } else {
                            self.halt_and_rebuild(host, input);
                        }
                    }
                    VDom::Text { .. } => {
                        self.halt_and_rebuild(host, input);
                    }
                }
            }
        }
    }

    pub(crate) fn halt(&mut self) {
        match &mut self.vdom {
            VDom::Text { state: node, .. } => {
                if let Some(parent) = node.parent_node() {
                    parent.remove_child(&node)
                }
            }
            VDom::Elem {
                children,
                state: node,
                ..
            } => {
                if let Some(parent) = node.parent_node() {
                    parent.remove_child(&node)
                }
                children.iter_mut().for_each(|x| {
                    x.halt();
                });
                // attrs.halt();
            }
        }
    }
}

fn update_attrs(node: &mut Node, attrs_old: &Attrs, attrs_new: &Attrs) {
    // TODO: Do a more efficient diff and update attrs on the node?
    attrs_new
        .into_iter()
        .for_each(|(key, val)| match attrs_old.get(key) {
            None => node.set_attribute(key, val),
            Some(val_old) => {
                if val != val_old {
                    node.set_attribute(key, val)
                }
            }
        });
    attrs_old
        .into_iter()
        .for_each(|(key, _)| match attrs_new.get(key) {
            None => node.remove_attribute(key),
            Some(_) => (),
        });
}

fn update_children(
    host: &Host,
    parent: &mut Node,
    children_old: &mut Vec<VDomMachine>,
    children_new: &Vec<VDomTree>,
) {
    children_old
        .iter_mut()
        .zip(children_new)
        .for_each(|(child_old, child_new)| {
            child_old.step(host, &child_new.vdom);
        });

    let to_be_removed = children_old.drain(children_new.len()..);
    to_be_removed.for_each(|mut child| {
        child.halt();
    });

    children_new
        .iter()
        .skip(children_old.len())
        .for_each(|child_new| {
            let child_vdom = VDomMachine::build_vdom(host, &child_new.vdom);
            let child_node = VDomMachine::get_node(&child_vdom);
            // TODO: Insert at the correct index
            parent.append_child(child_node);
            children_old.push(VDomMachine { vdom: child_vdom });
        });
}
