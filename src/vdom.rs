use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use std::collections::HashMap;

use crate::host::{Host, Listener, Node};

pub(crate) enum Attr<L> {
    StringAttr(String),
    EventHandler(L),
}

pub(crate) type HashmapAttrs<A> = HashMap<String, A>;
pub(crate) type Attrs = HashmapAttrs<Attr<Listener>>;

pub(crate) enum VDomNode<Children, State, Attributes> {
    Text {
        text: String,
        state: State,
    },
    Elem {
        name: String,
        attrs: Attributes,
        children: Children,
        state: State,
    },
}

pub(crate) struct VDom<State> {
    pub(crate) vdom: VDomNode<Vec<VDom<State>>, State, Attrs>,
}

pub(crate) fn build(host: &Host, input: VDom<()>) -> VDom<Node> {
    VDom {
        vdom: match input.vdom {
            VDomNode::Text { text, .. } => VDomNode::Text {
                text: text.clone(),
                state: host.create_text_node(&text),
            },
            VDomNode::Elem {
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
                        Attr::EventHandler(handler) => {
                            // let listener = Listener {
                            //     handler: Box::new(|| handler(runner)),
                            // };
                            node.add_event_listener(&key, &handler);
                            (key, Attr::EventHandler(handler))
                        }
                    })
                    .collect();
                // Attach children
                let children = children1
                    .into_iter()
                    .map(|vdom| {
                        let child = build(host, vdom);
                        node.append_child(child.node());
                        child
                    })
                    .collect();
                // Return the machine
                VDomNode::Elem {
                    name,
                    attrs: attrs_new,
                    children,
                    state: node,
                }
            }
        },
    }
}

impl VDom<Node> {
    pub(crate) fn install(&self, host: &Host) {
        host.install(self.node());
    }

    pub(crate) fn node(&self) -> &Node {
        match &self.vdom {
            VDomNode::Text { state, .. } => state,
            VDomNode::Elem { state, .. } => state,
        }
    }

    pub(crate) fn step(mut self, host: &Host, input: VDom<()>) -> VDom<Node> {
        match input.vdom {
            VDomNode::Text { text: tnew, .. } => match self.vdom {
                VDomNode::Text {
                    text: told,
                    state: mut node,
                } => {
                    if tnew != *told {
                        node.set_text_content(&tnew);
                    }
                    VDom {
                        vdom: VDomNode::Text {
                            text: tnew.clone(),
                            state: node,
                        },
                    }
                }
                VDomNode::Elem { .. } => {
                    self.halt();
                    build(
                        host,
                        VDom {
                            vdom: VDomNode::Text {
                                text: tnew,
                                state: (),
                            },
                        },
                    )
                }
            },
            VDomNode::Elem {
                name: name_new,
                attrs: attrs_new,
                children: children_new,
                ..
            } => match self.vdom {
                VDomNode::Elem {
                    name: name_old,
                    attrs: attrs_old,
                    children: children_old,
                    state: mut node,
                } => {
                    if name_new == *name_old {
                        // TODO: Update attrs
                        update_attrs(&mut node, &attrs_old, &attrs_new);
                        VDom {
                            vdom: VDomNode::Elem {
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
                        let mut newself = VDom {
                            vdom: VDomNode::Elem {
                                name: name_old,
                                attrs: attrs_old,
                                children: children_old,
                                state: node,
                            },
                        };
                        newself.halt();
                        build(
                            host,
                            VDom {
                                vdom: VDomNode::Elem {
                                    name: name_new,
                                    attrs: attrs_new,
                                    children: children_new,
                                    state: (),
                                },
                            },
                        )
                    }
                }
                VDomNode::Text { text, state } => {
                    let mut newself = VDom {
                        vdom: VDomNode::Text { text, state },
                    };
                    newself.halt();
                    build(
                        host,
                        VDom {
                            vdom: VDomNode::Elem {
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
            VDomNode::Text { state: node, .. } => {
                if let Some(parent) = node.parent_node() {
                    parent.remove_child(&node)
                }
            }
            VDomNode::Elem {
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
        .iter()
        .for_each(|(key, val)| match attrs_old.get(key) {
            None => match val {
                Attr::StringAttr(s) => node.set_attribute(key, s),
                Attr::EventHandler(handler) => {
                    node.add_event_listener(&key, &handler);
                }
            },
            Some(val_old) => match (val, val_old) {
                (Attr::StringAttr(val), Attr::StringAttr(_)) => node.set_attribute(&key, &val),
                (Attr::StringAttr(val), Attr::EventHandler(listener)) => {
                    node.remove_event_listener(&key, listener);
                    node.set_attribute(&key, val);
                }
                (Attr::EventHandler(handler), Attr::StringAttr(_)) => {
                    node.remove_attribute(&key);
                    node.add_event_listener(&key, &handler);
                }
                (Attr::EventHandler(handler), Attr::EventHandler(listener_old)) => {
                    node.remove_event_listener(&key, listener_old);
                    node.add_event_listener(&key, &handler);
                }
            },
        });
}

fn update_children(
    host: &Host,
    parent: &mut Node,
    children_old: Vec<VDom<Node>>,
    vdoms: Vec<VDom<()>>,
) -> Vec<VDom<Node>> {
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
                let child = build(host, vdom);
                // TODO: Insert at the correct index
                parent.append_child(child.node());
                Some(child)
            }
        })
        .collect()
}
