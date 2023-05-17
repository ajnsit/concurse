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

    let vdom = many_counter(100, href, &marc1, 0);

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

fn many_counter(
    n: u32,
    href: &'static Host,
    marc: &Arc<Mutex<VDom<Node>>>,
    count: i32,
) -> VDom<()> {
    let mut counters = Vec::new();
    for _ in 0..n {
        let marc1 = marc.clone();
        let marc2 = marc.clone();
        let listener = Listener {
            handler: Closure::new(move || {
                take_mut::take(
                    marc1
                        .lock()
                        .expect("Failed to lock machine within a handler")
                        .deref_mut(),
                    |m| step(m, href, many_counter(n, href, &marc2, count + 1)),
                );
            }),
        };
        counters.push(counter(count, listener));
    }
    VDom {
        vdom: VDomNode::Elem(Elem {
            name: "div".to_owned(),
            attrs: HashMap::default(),
            children: Vec::from(counters),
            state: (),
        }),
    }
}

fn counter(count: i32, listener: Listener) -> VDom<()> {
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
