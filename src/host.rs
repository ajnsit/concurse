pub(crate) struct Host;

impl Host {
  pub(crate) fn create_text_node(&self, str: &str) -> Node {todo!();}
  pub(crate) fn create_element(&self, name: &str) -> Node {todo!();}
}

pub(crate) struct Node;
pub(crate) struct Evt;

impl Node {
  pub(crate) fn set_text_content(&self, str: &str) {todo!();}
  pub(crate) fn insert_child_ix(&self, index: i32, child: &Node) {todo!();}
  pub(crate) fn remove_child(&self, child: &Node) {todo!();}
  pub(crate) fn parent_node(&self) -> &Node {todo!();}
  pub(crate) fn set_attribute(&self, key: &str, val: &str) {todo!();}
  pub(crate) fn remove_attribute(&self, key: &str) {todo!();}
  pub(crate) fn has_attribute(&self, key: &str) -> bool {todo!();}
  pub(crate) fn add_event_listener(&self, event_name: &str, handler: Box<dyn Fn(Evt)>) {todo!();}
  pub(crate) fn remove_event_listener(&self, event_name: &str, handler: Box<dyn Fn(Evt)>) {todo!();}

  // TODO: THIS IS JUST FOR NOW. DELME.
  pub(crate) fn dummyNode() -> Node {todo!();}
}
