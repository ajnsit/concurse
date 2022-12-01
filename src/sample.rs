use std::{
    collections::HashMap,
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use wasm_bindgen::{prelude::Closure, JsValue};

use crate::{
    host::{Host, Listener, Node},
    log,
    vdom::{build, install, step, Attr, Elem, Text, VDom, VDomNode},
};

pub(crate) fn test(name: &str) -> Result<(), JsValue> {
    let host = Host::mk_host();

    // Create and install a dummy machine
    let machine = VDom {
        vdom: build(
            &host,
            VDom {
                vdom: VDomNode::Text(Text {
                    text: format!("Initialising {}...", name),
                    state: (),
                }),
            },
        ),
    };
    install(&machine, &host);

    // Get 'static references by putting stuff in the heap
    let href = Box::<Host>::leak(Box::new(host));

    // Arc so we can share machines
    let marc1 = Arc::from(Mutex::from(machine));
    let marc2 = marc1.clone();

    let vdom = counter(href, marc1, 0);

    take_mut::take(
        marc2
            .as_ref()
            .lock()
            .expect("Failed to lock machine for stepping")
            .deref_mut(),
        |m| step(m, href, vdom),
    );

    Ok(())
}

fn counter(href: &'static Host, marc1: Arc<Mutex<VDom<Node>>>, count: i32) -> VDom<()> {
    let marc2 = marc1.clone();
    let marc3 = marc1.clone();
    let listener = Listener {
        handler: Closure::once(move || {
            take_mut::take(
                marc2
                    .as_ref()
                    .lock()
                    .expect("Failed to lock machine within a handler")
                    .deref_mut(),
                |m| step(m, href, counter(href, marc3, count + 1)),
            );
            log!("CLICKED!");
        }),
    };
    VDom {
        vdom: VDomNode::Elem(Elem {
            name: "div".to_owned(),
            attrs: HashMap::default(),
            children: Vec::from([VDom {
                vdom: VDomNode::Elem(Elem {
                    name: "button".to_owned(),
                    attrs: HashMap::from([("click".to_owned(), Attr::EventHandler(listener))]),
                    children: Vec::from([(VDom {
                        vdom: VDomNode::Text(Text {
                            text: format!("Count: {}", count),
                            state: (),
                        }),
                    })]),
                    state: (),
                }),
            }]),
            state: (),
        }),
    }
}

fn annotate<F>(f: F) -> F
where
    F: 'static + FnOnce(),
{
    f
}
