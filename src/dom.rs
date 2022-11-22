use std::collections::HashMap;

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

use crate::{
    alert,
    host::{Host, Listener},
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

    machine.step(
        &host,
        VDom {
            vdom: VDomNode::Elem {
                name: "button".to_owned(),
                attrs: HashMap::from([(
                    "click".to_owned(),
                    Attr::EventHandler(Listener {
                        handler: Closure::new(move || {
                            machine.step(
                                &host,
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

    // At runtime:
    // The problem is that the vdomnode is already dropped before I can click on the button
    // Dropping the node drops the closure, which drops the event handler

    Ok(())
}

pub(crate) fn simple_test() -> Result<(), JsValue> {
    let host = Host::mk_host();

    let val = host.document.create_element("button")?;
    let closure = Closure::<dyn Fn()>::new(|| {
        alert("CLICKED!");
    });
    val.set_text_content(Some("Hello from Rust!"));
    val.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());

    /*
    The instance of `Closure` that we created will invalidate its
    corresponding JS callback whenever it is dropped, so if we were to
    normally return from `setup_clock` then our registered closure will
    raise an exception when invoked.

    Normally we'd store the handle to later get dropped at an appropriate
    time but for now we want it to be a global handler so we use the
    `forget` method to drop it without invalidating the closure. Note that
    this is leaking memory in Rust, so this should be done judiciously!
    */
    closure.forget();

    host.body.append_child(&val)?;

    Ok(())
}
