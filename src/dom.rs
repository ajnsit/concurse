use std::collections::HashMap;

use wasm_bindgen::{prelude::Closure, JsValue};

use crate::{
    alert,
    host::{Host, Listener},
    vdom::{build, Attr, VDom, VDomNode},
};

pub(crate) fn test(name: &str) -> Result<(), JsValue> {
    let host = Host::mk_host();

    let attrs1: HashMap<String, Attr<Listener>> = HashMap::from([(
        "click".to_owned(),
        Attr::EventHandler(Listener {
            handler: Closure::new(move || {
                alert("CLICKED!");
            }),
        }),
    )]);

    let vdom_text = VDom {
        vdom: VDomNode::Text {
            text: format!("Hello to {}, from Rust!", name),
            state: (),
        },
    };

    let mut children = Vec::new();
    children.push(vdom_text);

    let vdom1 = VDom {
        vdom: VDomNode::Elem {
            name: "button".to_owned(),
            attrs: attrs1,
            children,
            state: (),
        },
    };

    let node = build(&host, vdom1);
    node.install(&host);

    Ok(())
}
