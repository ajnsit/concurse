use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use wasm_bindgen::{prelude::Closure, JsValue};

use crate::{
    alert,
    host::{Host, Listener, Node},
    vdom::{build, Attr, VDom, VDomNode},
};

pub(crate) fn test(name: &str) -> Result<(), JsValue> {
    let host = Host::mk_host();

    let mut machine = VDom {
        vdom: build(
            &host,
            VDom {
                vdom: VDomNode::Text {
                    text: format!("Initialising {}...", name),
                    state: (),
                },
            },
        ),
    };

    machine.install(&host);

    // Sync step test
    machine.step(
        &host,
        VDom {
            vdom: VDomNode::Elem {
                name: "button".to_owned(),
                attrs: HashMap::default(),
                children: Vec::from([(VDom {
                    vdom: VDomNode::Text {
                        text: format!("CLICKED {}, from Rust!", name),
                        state: (),
                    },
                })]),
                state: (),
            },
        },
    );

    let arc: Arc<Mutex<VDom<Node>>> = Arc::from(Mutex::from(machine));
    let arc2 = arc.clone();
    let host2 = Host::mk_host();

    // Async step test
    arc.as_ref()
        .lock()
        .expect("Failed to lock machine for stepping")
        .step(
            &host,
            VDom {
                vdom: VDomNode::Elem {
                    name: "button".to_owned(),
                    attrs: HashMap::from([(
                        "click".to_owned(),
                        Attr::EventHandler(Listener {
                            handler: Closure::once(move || {
                                arc2.as_ref()
                                    .lock()
                                    .expect("Failed to lock machine for stepping")
                                    .step(
                                        &host2,
                                        VDom {
                                            vdom: VDomNode::Text {
                                                text: "Clicked!".to_owned(),
                                                state: (),
                                            },
                                        },
                                    );
                                alert("CLICKED!");
                            }),
                        }),
                    )]),
                    children: Vec::from([(VDom {
                        vdom: VDomNode::Text {
                            text: format!("Hello to {}, from Rust!", name),
                            state: (),
                        },
                    })]),
                    state: (),
                },
            },
        );

    Ok(())
}
