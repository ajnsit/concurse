pub(crate) struct Host {}

impl Host {
    pub(crate) fn mk_host() -> Host {
        // let app = App::default().with_scheme(Scheme::Gtk);
        Host {}
    }
    pub(crate) fn create_text_node(&self, str: &str) -> Node {
        Node::Text()
    }
    pub(crate) fn create_element(&self, name: &str) -> Node {
        match name {
            // "button" => Node::Button(Button::default()),
            // "output" => Node::Output(Output::default()),
            _ => panic!("Invalid element type"),
        }
    }
}

pub enum Node {
    Text(),
    Button(),
    Output(),
}

pub(crate) struct Evt;

impl Node {
    pub(crate) fn set_text_content(&mut self, str: &str) {
        // match self {
        //     Node::Text(_) => *self = Node::Text(str),
        //     Node::Button(b) => b.set_label(str.as_str()),
        //     Node::Output(o) => o.set_label(str.as_str()),
        // }
    }
    pub(crate) fn insert_child_ix(&self, index: i32, child: &Node) {
        // match self {
        //     Node::Text(_) => {}
        //     Node::Button(b) => b.,
        //     Node::Output(_) => todo!(),
        // }
    }
    pub(crate) fn remove_child(&self, _child: &Node) {
        todo!()
    }
    pub(crate) fn parent_node(&self) -> &Node {
        todo!()
    }
    pub(crate) fn set_attribute(&self, _key: &str, _val: &str) {
        todo!()
    }
    pub(crate) fn remove_attribute(&self, _key: &str) {
        todo!()
    }
    pub(crate) fn has_attribute(&self, _key: &str) -> bool {
        todo!()
    }
    pub(crate) fn add_event_listener(&self, _event_name: &str, _handler: Box<dyn Fn(Evt)>) {
        todo!()
    }
    pub(crate) fn remove_event_listener(&self, _event_name: &str, _handler: Box<dyn Fn(Evt)>) {
        todo!()
    }

    // TODO: THIS IS JUST FOR NOW. DELME.
    pub(crate) fn dummy_node() -> Node {
        todo!()
    }
}
