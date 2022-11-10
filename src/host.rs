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
pub(crate) struct Listener;

impl Node {
    pub(crate) fn set_text_content(&mut self, str: &str) {
        todo!()
    }
    pub(crate) fn append_child(&self, child: &Node) {
        todo!()
    }
    pub(crate) fn insert_child_before(&self, existing: &Node, child: &Node) {
        todo!()
    }
    pub(crate) fn remove_child(&self, child: &Node) {
        todo!()
    }
    pub(crate) fn parent_node(&self) -> Option<Node> {
        todo!()
    }
    pub(crate) fn set_attribute(&self, key: &str, val: &str) {
        todo!()
    }
    pub(crate) fn remove_attribute(&self, key: &str) {
        todo!()
    }
    pub(crate) fn has_attribute(&self, key: &str) -> bool {
        todo!()
    }
    pub(crate) fn make_event_listener(&self, handler: Box<dyn Fn()>) -> Listener {
        todo!()
    }
    pub(crate) fn add_event_listener(&self, event_name: &str, handler: Listener) {
        todo!()
    }
    pub(crate) fn remove_event_listener(&self, event_name: &str, handler: Listener) {
        todo!()
    }

    // TODO: THIS IS JUST FOR NOW. DELME.
    pub(crate) fn dummy_node() -> Node {
        todo!()
    }
}
