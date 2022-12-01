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

pub(crate) struct Elem<Children, State, Attributes> {
    pub(crate) name: String,
    pub(crate) attrs: Attributes,
    pub(crate) children: Children,
    pub(crate) state: State,
}

pub(crate) struct Text<State> {
    pub(crate) text: String,
    pub(crate) state: State,
}

pub(crate) enum VDomNode<Children, State, Attributes> {
    Text(Text<State>),
    Elem(Elem<Children, State, Attributes>),
}

pub(crate) struct VDom<State> {
    pub(crate) vdom: VDomNode<Vec<VDom<State>>, State, Attrs>,
}

pub(crate) fn build(host: &Host, input: VDom<()>) -> VDomNode<Vec<VDom<Node>>, Node, Attrs> {
    match input.vdom {
        VDomNode::Text(Text { text, .. }) => VDomNode::Text(Text {
            text: text.clone(),
            state: host.create_text_node(&text),
        }),
        VDomNode::Elem(Elem {
            name,
            attrs,
            children: children1,
            ..
        }) => {
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
                    let child = VDom {
                        vdom: build(host, vdom),
                    };
                    node.append_child(child.vdom.node());
                    child
                })
                .collect();
            // Return the machine
            VDomNode::Elem(Elem {
                name,
                attrs: attrs_new,
                children,
                state: node,
            })
        }
    }
}

impl VDomNode<Vec<VDom<Node>>, Node, Attrs> {
    pub(crate) fn node(&self) -> &Node {
        match self {
            VDomNode::Text(Text { state, .. }) => state,
            VDomNode::Elem(Elem { state, .. }) => state,
        }
    }
}

pub(crate) fn install(m: &VDom<Node>, host: &Host) {
    host.install(m.vdom.node());
}

pub(crate) fn halt(m: VDom<Node>) -> Option<Node> {
    match m.vdom {
        VDomNode::Text(Text { state: node, .. }) => {
            if let Some(parent) = node.parent_node() {
                parent.remove_child(&node);
                Some(parent)
            } else {
                None
            }
        }
        VDomNode::Elem(Elem {
            children,
            state: node,
            ..
        }) => {
            let ret = if let Some(parent) = node.parent_node() {
                parent.remove_child(&node);
                Some(parent)
            } else {
                None
            };
            children.into_iter().for_each(|x| {
                halt(x);
            });
            // TODO: Cleanup attrs
            // attrs.halt();
            ret
        }
    }
}

pub(crate) fn halt_and_build(m: VDom<Node>, host: &Host, input: VDom<()>) -> VDom<Node> {
    let parent = halt(m);
    let vdom = build(host, input);
    parent.map(|p| {
        // TODO: Insert in the same place as prev node
        p.append_child(vdom.node());
    });
    VDom { vdom }
}

pub(crate) fn step(v: VDom<Node>, host: &Host, input: VDom<()>) -> VDom<Node> {
    // let m = v.borrow_mut();
    match input.vdom {
        VDomNode::Text(Text { text: tnew, .. }) => match v.vdom {
            VDomNode::Text(Text {
                text: told,
                state: mut node,
            }) => {
                if tnew != *told {
                    node.set_text_content(&tnew);
                }
                VDom {
                    vdom: VDomNode::Text(Text {
                        text: tnew,
                        state: node,
                    }),
                }
            }
            VDomNode::Elem(Elem { .. }) => halt_and_build(
                v,
                host,
                VDom {
                    vdom: VDomNode::Text(Text {
                        text: tnew,
                        state: (),
                    }),
                },
            ),
        },
        VDomNode::Elem(Elem {
            name,
            attrs,
            children: children_new,
            ..
        }) => match v.vdom {
            VDomNode::Elem(Elem {
                name: name_old,
                attrs: attrs_old,
                children: children_old,
                state: node,
            }) => {
                let mut n = node;
                if name == name_old {
                    update_node_attrs(&mut n, &attrs_old, &attrs);
                    let children = update_children(host, &mut n, children_old, children_new);
                    let vdom = VDomNode::Elem(Elem {
                        name,
                        attrs,
                        children,
                        state: n,
                    });
                    VDom { vdom }
                } else {
                    let v1 = VDom {
                        vdom: VDomNode::Elem(Elem {
                            name: name_old,
                            attrs: attrs_old,
                            children: children_old,
                            state: n,
                        }),
                    };
                    halt_and_build(
                        v1,
                        host,
                        VDom {
                            vdom: VDomNode::Elem(Elem {
                                name,
                                attrs,
                                children: children_new,
                                state: (),
                            }),
                        },
                    )
                }
            }
            VDomNode::Text(Text { .. }) => halt_and_build(
                v,
                host,
                VDom {
                    vdom: VDomNode::Elem(Elem {
                        name,
                        attrs,
                        children: children_new,
                        state: (),
                    }),
                },
            ),
        },
    }
}

fn update_node_attrs(node: &mut Node, attrs_old: &Attrs, attrs_new: &Attrs) {
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
            Both(child_old, vdom) => Some(step(child_old, host, vdom)),
            Left(child_old) => {
                halt(child_old);
                None
            }
            Right(vdom) => {
                let child = build(host, vdom);
                // TODO: Insert at the correct index
                parent.append_child(child.node());
                Some(VDom { vdom: child })
            }
        })
        .collect()
}
