use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Document, Element, HtmlElement, Window};

pub(crate) struct Host {
    pub(crate) window: Window,
    pub(crate) document: Document,
    pub(crate) body: HtmlElement,
}

impl Host {
    pub(crate) fn mk_host() -> Host {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("document should have a body");
        Host {
            window,
            document,
            body,
        }
    }
    pub(crate) fn create_text_node(&self, str: &str) -> Node {
        let element = self
            .document
            .create_element("span")
            .expect("should be able to create an element");
        element.set_text_content(Some(str));
        Node { element }
    }
    pub(crate) fn create_element(&self, name: &str) -> Node {
        let element = self
            .document
            .create_element(name)
            .expect("should be able to create an element");
        Node { element }
    }

    pub(crate) fn install(&self, node: &Node) {
        self.body.append_child(&node.element);
    }
}

pub struct Node {
    element: Element,
}

#[derive(Clone)]
pub(crate) struct Listener {
    // handler: fn(),
    pub(crate) handler: Closure<dyn FnMut()>,
}

impl Node {
    pub(crate) fn set_text_content(&mut self, str: &str) {
        self.element.set_text_content(Some(str));
    }
    pub(crate) fn append_child(&self, child: &Node) {
        self.element.append_child(&child.element);
    }
    pub(crate) fn insert_child_before(&self, existing: &Node, child: &Node) {
        self.element
            .insert_before(&existing.element, Some(&child.element));
    }
    pub(crate) fn remove_child(&self, child: &Node) {
        self.element.remove_child(&child.element);
    }
    pub(crate) fn parent_node(&self) -> Option<Node> {
        let parent = self.element.parent_element();
        parent.map(|element| Node { element })
    }
    pub(crate) fn set_attribute(&self, key: &str, val: &str) {
        self.element.set_attribute(key, val);
    }
    pub(crate) fn remove_attribute(&self, key: &str) {
        self.element.remove_attribute(key);
    }
    pub(crate) fn has_attribute(&self, key: &str) -> bool {
        self.element.has_attribute(key)
    }

    // pub(crate) fn make_event_listener(&self, handler: Box<dyn Fn()>) -> Listener {
    //     Listener {
    //         listener: Closure::<dyn FnMut()>::new(move || {
    //             handler();
    //         }),
    //     }
    // }

    pub(crate) fn add_event_listener(&self, event_name: &str, handler: &Listener) {
        // pub(crate) fn add_event_listener(&self, event_name: &str, handler: Listener) {
        self.element
            .add_event_listener_with_callback(event_name, handler.handler.as_ref().unchecked_ref())
            .expect("Could not add event listener");
    }

    pub(crate) fn remove_event_listener(&self, event_name: &str, handler: &Listener) {
        // pub(crate) fn remove_event_listener(&self, event_name: &str, handler: Listener) {
        self.element
            .remove_event_listener_with_callback(
                event_name,
                handler.handler.as_ref().unchecked_ref(),
            )
            .expect("Could not remove event listener");
    }
}
