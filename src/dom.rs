use std::collections::HashMap;

use wasm_bindgen::{prelude::Closure, JsValue};

use crate::{
    host::{Host, Listener, Node},
    log,
    vdom::{build, Attr, VDom, VDomNode},
};

pub(crate) fn test(name: &str) -> Result<(), JsValue> {
    let host = Host::mk_host();

    // Create and install a dummy machine
    let machine = VDom {
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

    // Get 'static references by putting stuff in the heap
    let mref = Box::<VDom<Node>>::leak(Box::new(machine));
    let href = Box::<Host>::leak(Box::new(host));

    // Run
    let m1 = mref as *mut VDom<Node>;
    let vdom = counter(href, unsafe { &mut *m1 }, 0);
    mref.step(href, vdom);

    Ok(())
}

fn counter(href: &Host, mref: &mut VDom<Node>, count: i32) -> VDom<()> {
    let m1 = mref as *mut VDom<Node>;
    let mref1 = unsafe { &mut *m1 };
    let h1 = href as *const Host;
    let href1 = unsafe { &*h1 };
    VDom {
        vdom: VDomNode::Elem {
            name: "div".to_owned(),
            attrs: HashMap::default(),
            children: Vec::from([(VDom {
                vdom: VDomNode::Elem {
                    name: "button".to_owned(),
                    attrs: HashMap::from([(
                        "click".to_owned(),
                        Attr::EventHandler(Listener {
                            handler: Closure::once(move || {
                                let vdom = counter(href1, mref1, count + 1);
                                mref1.step(href1, vdom);
                                log!("CLICKED!");
                            }),
                        }),
                    )]),
                    children: Vec::from([(VDom {
                        vdom: VDomNode::Text {
                            text: format!("Count: {}", count),
                            state: (),
                        },
                    })]),
                    state: (),
                },
            })]),
            state: (),
        },
    }
}
