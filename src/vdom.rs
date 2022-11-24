use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use std::collections::HashMap;

use crate::host::{Host, Listener, Node};

#[derive(Clone)]
pub(crate) enum Attr<L> {
    StringAttr(String),
    EventHandler(L),
}

pub(crate) type HashmapAttrs<A> = HashMap<String, A>;
pub(crate) type Attrs = HashmapAttrs<Attr<Listener>>;

#[derive(Clone)]
pub struct Text<State> {
    text: String,
    state: State,
}

impl<State> Text<State> {
    pub fn new(text: String, state: State) -> Self {
        Text { text, state }
    }

    pub fn update(&mut self, new_text_struct: Text<State>) {
        self.text = new_text_struct.text;
        self.state = new_text_struct.state;
    }
}

#[derive(Clone)]
pub struct Elem<Attributes, Children, State> {
    name: String,
    attrs: Attributes,
    children: Children,
    state: State,
}

impl<Attributes, Children, State> Elem<Attributes, Children, State> {
    pub fn new(name: String, attrs: Attributes, children: Children, state: State) -> Self {
        Elem {
            name,
            attrs,
            children,
            state,
        }
    }

    pub fn update(&mut self, new_elem_struct: Elem<Attributes, Children, State>) {
        self.name = new_elem_struct.name;
        self.attrs = new_elem_struct.attrs;
        self.children = new_elem_struct.children;
        self.state = new_elem_struct.state;
    }
}

pub(crate) enum VDomNode<Children, State, Attributes> {
    Text(Text<State>),
    Elem(Elem<Attributes, Children, State>),
}

pub(crate) struct VDom<State> {
    pub(crate) vdom: VDomNode<Vec<VDom<State>>, State, Attrs>,
}

pub(crate) fn build(host: &Host, input: VDom<()>) -> VDomNode<Vec<VDom<Node>>, Node, Attrs> {
    match input.vdom {
        VDomNode::Text(text) => {
            VDomNode::new_text(text.text.clone(), host.create_text_node(&text.text))
        }
        VDomNode::Elem(elem) => {
            // Attach attributes
            let node = host.create_element(&elem.name);
            let attrs_new = elem
                .attrs
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
            let children = elem
                .children
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
            VDomNode::new_elem(elem.name, attrs_new, children, node)
        }
    }
}

impl VDomNode<Vec<VDom<Node>>, Node, Attrs> {
    pub fn new_text(text: String, state: Node) -> Self {
        VDomNode::Text(Text::new(text, state))
    }

    pub fn new_elem(name: String, attrs: Attrs, children: Vec<VDom<Node>>, state: Node) -> Self {
        VDomNode::Elem(Elem::new(name, attrs, children, state))
    }
    pub(crate) fn node(&self) -> &Node {
        match self {
            VDomNode::Text(text) => &text.state,
            VDomNode::Elem(elem) => &elem.state,
        }
    }
}

impl VDom<Node> {
    pub fn new(vdom: VDomNode<Vec<VDom<Node>>, Node, Attrs>) -> Self {
        VDom { vdom }
    }

    pub(crate) fn install(&self, host: &Host) {
        host.install(self.vdom.node());
    }

    pub(crate) fn halt(&mut self) -> Option<Node> {
        match self.vdom {
            VDomNode::Text(text) => {
                let node = text.state;
                if let Some(parent) = node.parent_node() {
                    parent.remove_child(&node);
                    Some(parent)
                } else {
                    None
                }
            }
            VDomNode::Elem(elem) => {
                let node = elem.state;
                let children = elem.children;
                let ret = if let Some(parent) = node.parent_node() {
                    parent.remove_child(&node);
                    Some(parent)
                } else {
                    None
                };
                children.iter_mut().for_each(|x| {
                    x.halt();
                });
                // TODO: Cleanup attrs
                // attrs.halt();
                ret
            }
        }
    }

    pub(crate) fn halt_and_build(&mut self, host: &Host, input: VDom<()>) {
        let parent = self.halt();
        parent.map(|p| {
            self.vdom = build(host, input);
            // TODO: Insert in the same place as prev node
            p.append_child(self.vdom.node());
        });
    }

    pub(crate) fn step(&mut self, host: &Host, input: VDom<()>) {
        match input.vdom {
            VDomNode::Text(mut new) => match self.vdom {
                VDomNode::Text(mut old) => {
                    let mut node = old.state;
                    if new.text != old.text {
                        node.set_text_content(&new.text);
                        old.update(Text::new(new.text.clone(), old.state));
                    }
                }
                VDomNode::Elem(elem) => {
                    let node = VDomNode::Text(Text {
                        text: new.text.clone(),
                        state: (),
                    });
                    self.halt_and_build(host, VDom { vdom: node });
                }
            },
            VDomNode::Elem(mut new) => match self.vdom {
                VDomNode::Elem(mut old) => {
                    let mut node = old.state;
                    let attrs_old = old.attrs;
                    let attrs_new = new.attrs;
                    if new.name == old.name {
                        // TODO: Update attrs
                        update_attrs(&mut node, &attrs_old, &attrs_new);
                        old.update(Elem::new(old.name, attrs_new, old.children, old.state));
                        update_children(host, &mut node, &mut old.children, new.children);
                    } else {
                        let node = VDomNode::Elem(Elem {
                            name: new.name,
                            attrs: attrs_new,
                            children: new.children,
                            state: (),
                        });
                        self.halt_and_build(host, VDom { vdom: node });
                    }
                }
                VDomNode::Text(text) => {
                    let node = VDomNode::Elem(Elem {
                        name: new.name,
                        attrs: new.attrs,
                        children: new.children,
                        state: (),
                    });
                    self.halt_and_build(host, VDom { vdom: node });
                }
            },
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
    children: &mut Vec<VDom<Node>>,
    vdoms: Vec<VDom<()>>,
) {
    let mut cs = std::mem::take(children);
    cs = update_children1(host, parent, cs, vdoms);
    std::mem::swap(&mut cs, children);
}

fn update_children1(
    host: &Host,
    parent: &mut Node,
    children_old: Vec<VDom<Node>>,
    vdoms: Vec<VDom<()>>,
) -> Vec<VDom<Node>> {
    children_old
        .into_iter()
        .zip_longest(vdoms)
        .filter_map(|zip| match zip {
            Both(mut child_old, vdom) => {
                child_old.step(host, vdom);
                Some(child_old)
            }
            Left(mut child_old) => {
                child_old.halt();
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
