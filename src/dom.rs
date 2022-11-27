use std::collections::HashMap;

use wasm_bindgen::{prelude::Closure, JsValue};

use crate::{
    host::{Host, Listener, Node},
    log,
    vdom::{build, Attr, Elem, Text, VDom, VDomNode},
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
    machine.install(&host);

    // Get 'static references by putting stuff in the heap
    // If the references are not static then the unsafe stuff below to replicate refs will cause runtime errors
    let mref = Box::<VDom<Node>>::leak(Box::new(machine));
    let href = Box::<Host>::leak(Box::new(host));
    let m1 = mref as *mut VDom<Node>;
    let mref1: &'static mut VDom<Node> = unsafe { &mut *m1 };

    let vdom = counter(href, mref1, 0);
    mref.step(href, vdom);

    Ok(())
}

fn counter(href: &'static Host, mref: &'static mut VDom<Node>, count: i32) -> VDom<()> {
    let m1 = mref as *mut VDom<Node>;
    let mref1: &'static mut VDom<Node> = unsafe { &mut *m1 };
    VDom {
        vdom: VDomNode::Elem(Elem {
            name: "div".to_owned(),
            attrs: HashMap::default(),
            children: Vec::from([(VDom {
                vdom: VDomNode::Elem(Elem {
                    name: "button".to_owned(),
                    attrs: HashMap::from([(
                        "click".to_owned(),
                        Attr::EventHandler(Listener {
                            handler: Closure::once(annotate(move || {
                                let vdom = counter(href, mref, count + 1);
                                mref1.step(href, vdom);
                                log!("CLICKED!");
                            })),
                        }),
                    )]),
                    children: Vec::from([(VDom {
                        vdom: VDomNode::Text(Text {
                            text: format!("Count: {}", count),
                            state: (),
                        }),
                    })]),
                    state: (),
                }),
            })]),
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
