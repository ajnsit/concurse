use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use std::collections::HashMap;

use crate::host::{Host, Listener, Node};

pub(crate) enum Attr<F> {
    StringAttr(String),
    EventHandler(F),
}

type Attrs<A> = HashMap<String, Attr<A>>;

type Handler<A> = Box<dyn Fn(A)>;
// type TreeAttrs<A> = Attrs<Handler<A>>;
type MachineAttrs<A> = Attrs<Listener<A>>;

pub(crate) enum VDom<Children, State, A> {
    Text {
        text: String,
        state: State,
    },
    Elem {
        name: String,
        attrs: Attrs<A>,
        children: Children,
        state: State,
    },
}

pub(crate) struct VDomTree<A> {
    pub(crate) vdom: VDom<Vec<VDomTree<A>>, (), Listener<A>>,
}

pub(crate) struct VDomMachine<A> {
    pub(crate) vdom: VDom<Vec<VDomMachine<A>>, Node, Listener<A>>,
}

struct VDomMachineOps;
impl VDomMachineOps {
    pub(crate) fn build<B>(host: &Host, input: VDomTree<B>) -> VDomMachine<B> {
        VDomMachine {
            vdom: match input.vdom {
                VDom::Text { text, .. } => VDom::Text {
                    text: text.clone(),
                    state: host.create_text_node(&text),
                },
                VDom::Elem {
                    name,
                    attrs,
                    children: children1,
                    ..
                } => {
                    // Attach attributes
                    let node = host.create_element(&name);
                    let attrs_new = attrs
                        .into_iter()
                        .map(|(key, val)| match val {
                            Attr::StringAttr(v) => {
                                node.set_attribute(&key, &v);
                                (key, Attr::StringAttr(v))
                            }
                            Attr::EventHandler(listener) => {
                                // let listener = Host::make_event_listener(Box::new(handler));
                                node.add_event_listener(&key, &listener);
                                (key, Attr::EventHandler(listener))
                            }
                        })
                        .collect();
                    // Attach children
                    let children = children1
                        .into_iter()
                        .map(|vdom| {
                            let child: VDomMachine<B> = VDomMachineOps::build(host, vdom);
                            node.append_child(child.node());
                            child
                        })
                        .collect();
                    // Return the machine
                    VDom::Elem {
                        name,
                        attrs: attrs_new,
                        children,
                        state: node,
                    }
                }
            },
        }
    }
}

impl<A> VDomMachine<A> {
    pub(crate) fn install(&self, host: &Host) {
        host.install(self.node());
    }

    pub(crate) fn node(&self) -> &Node {
        match &self.vdom {
            VDom::Text { state, .. } => state,
            VDom::Elem { state, .. } => state,
        }
    }

    pub(crate) fn step<B>(mut self, host: &Host, input: VDomTree<B>) -> VDomMachine<B> {
        match input.vdom {
            VDom::Text { text: tnew, .. } => match self.vdom {
                VDom::Text {
                    text: told,
                    state: mut node,
                } => {
                    if tnew != *told {
                        node.set_text_content(&tnew);
                    }
                    VDomMachine {
                        vdom: VDom::Text {
                            text: tnew.clone(),
                            state: node,
                        },
                    }
                }
                VDom::Elem { .. } => {
                    self.halt();
                    VDomMachineOps::build(
                        host,
                        VDomTree {
                            vdom: VDom::Text {
                                text: tnew,
                                state: (),
                            },
                        },
                    )
                }
            },
            VDom::Elem {
                name: name_new,
                attrs: attrs_new,
                children: children_new,
                ..
            } => match self.vdom {
                VDom::Elem {
                    name: name_old,
                    attrs: attrs_old,
                    children: children_old,
                    state: mut node,
                } => {
                    if name_new == *name_old {
                        // TODO: Update attrs
                        update_attrs(&mut node, &attrs_old, &attrs_new);
                        VDomMachine {
                            vdom: VDom::Elem {
                                name: name_old.clone(),
                                attrs: attrs_new,
                                children: update_children(
                                    host,
                                    &mut node,
                                    children_old,
                                    children_new,
                                ),
                                state: node,
                            },
                        }
                    } else {
                        let mut newself = VDomMachine {
                            vdom: VDom::Elem {
                                name: name_old,
                                attrs: attrs_old,
                                children: children_old,
                                state: node,
                            },
                        };
                        newself.halt();
                        VDomMachineOps::build(
                            host,
                            VDomTree {
                                vdom: VDom::Elem {
                                    name: name_new,
                                    attrs: attrs_new,
                                    children: children_new,
                                    state: (),
                                },
                            },
                        )
                    }
                }
                VDom::Text { text, state } => {
                    let mut newself: VDomMachine<A> = VDomMachine {
                        vdom: VDom::Text { text, state },
                    };
                    newself.halt();
                    VDomMachineOps::build(
                        host,
                        VDomTree {
                            vdom: VDom::Elem {
                                name: name_new,
                                attrs: attrs_new,
                                children: children_new,
                                state: (),
                            },
                        },
                    )
                }
            },
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

fn update_attrs<A, B>(node: &mut Node, attrs_old: &MachineAttrs<A>, attrs_new: &MachineAttrs<B>) {
    // TODO: Do a more efficient diff and update attrs on the node?
    attrs_new
        .iter()
        .for_each(|(key, val)| match attrs_old.get(key) {
            None => match val {
                Attr::StringAttr(s) => node.set_attribute(key, s),
                Attr::EventHandler(listener) => node.add_event_listener(&key, &listener),
            },
            Some(val_old) => match (val, val_old) {
                (Attr::StringAttr(val), Attr::StringAttr(_)) => node.set_attribute(&key, &val),
                (Attr::StringAttr(val), Attr::EventHandler(listener)) => {
                    node.remove_event_listener(&key, listener);
                    node.set_attribute(&key, val);
                }
                (Attr::EventHandler(listener), Attr::StringAttr(_)) => {
                    node.remove_attribute(&key);
                    node.add_event_listener(&key, &listener);
                }
                (Attr::EventHandler(listener), Attr::EventHandler(listener_old)) => {
                    node.remove_event_listener(&key, listener_old);
                    node.add_event_listener(&key, &listener);
                }
            },
        });
}

fn update_children<A, B>(
    host: &Host,
    parent: &mut Node,
    children_old: Vec<VDomMachine<A>>,
    vdoms: Vec<VDomTree<B>>,
) -> Vec<VDomMachine<B>> {
    children_old
        .into_iter()
        .zip_longest(vdoms)
        .filter_map(|zip| match zip {
            Both(child_old, vdom) => Some(child_old.step(host, vdom)),
            Left(mut child_old) => {
                child_old.halt();
                None
            }
            Right(vdom) => {
                let child = VDomMachineOps::build(host, vdom);
                // TODO: Insert at the correct index
                parent.append_child(child.node());
                Some(child)
            }
        })
        .collect()
}
