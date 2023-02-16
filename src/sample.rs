use std::{
    collections::HashMap,
    ops::{DerefMut, Generator, GeneratorState},
    pin::Pin,
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

    // let vdom = many_counter(1000, href, marc1, 0);
    let vdom = runner(href, marc1, counter_gen());

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

fn many_counter(n: u32, href: &'static Host, marc: Arc<Mutex<VDom<Node>>>, count: i32) -> VDom<()> {
    let mut counters = Vec::new();
    for _ in 0..n {
        let marc1 = marc.clone();
        let marc2 = marc.clone();
        let listener = Listener {
            handler: Closure::once(move || {
                take_mut::take(
                    marc1
                        .as_ref()
                        .lock()
                        .expect("Failed to lock machine within a handler")
                        .deref_mut(),
                    |m| step(m, href, many_counter(n, href, marc2, count + 1)),
                );
                log!("CLICKED!");
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

// Sync

fn runner<
    V: FnMut(Listener) -> VDom<()>,
    W: 'static + Unpin + Generator<Yield = V, Return = ()>,
>(
    href: &'static Host,
    marc: Arc<Mutex<VDom<Node>>>,
    mut w: W,
) -> VDom<()> {
    let finished_text: VDom<()> = VDom {
        vdom: VDomNode::Text(Text {
            text: "FINISHED".to_owned(),
            state: (),
        }),
    };
    match Pin::new(&mut w).resume(()) {
        GeneratorState::Complete(_) => return finished_text,
        GeneratorState::Yielded(mut vinit) => {
            let mut listener_ref = Option::None;
            let listener = Listener {
                handler: Closure::once(move || {
                    match Pin::new(&mut w).resume(()) {
                        GeneratorState::Yielded(mut v) => match listener_ref {
                            Some(l) => {
                                log!("GOT LISTENER");
                                take_mut::take(
                                    marc.as_ref()
                                        .lock()
                                        .expect("Failed to lock machine within a handler")
                                        .deref_mut(),
                                    |m| step(m, href, v(l)),
                                )
                            }
                            None => log!("ERROR: Listener not found"),
                        },
                        GeneratorState::Complete(_) => (),
                    };
                }),
            };
            listener_ref = Option::Some(listener);
            log!("PUT LISTENER");
            match listener_ref {
                Some(l) => vinit(l),
                None => finished_text,
            }
        }
    }
}

fn counter_gen(
) -> impl 'static + Unpin + Generator<Yield = impl FnMut(Listener) -> VDom<()>, Return = ()> {
    let ui = || {
        log!("INIT");
        let mut count = 20;
        loop {
            log!("COUNT: {}", count);
            yield move |l| {
                log!("INSIDE COUNT: {}", count);
                counter(count, l)
            };
            count += 1;
        }
    };
    ui
}

fn hello() -> impl 'static + Unpin + Generator<Yield = impl FnMut(Listener) -> VDom<()>, Return = ()>
{
    let ui = || loop {
        yield move |l| button("Say Hello".to_owned(), l);
        // yield move |l| display("Hello!".to_owned());
    };
    ui
}

fn display(s: String) -> VDom<()> {
    VDom {
        vdom: VDomNode::Text(Text { text: s, state: () }),
    }
}

fn button(s: String, listener: Listener) -> VDom<()> {
    VDom {
        vdom: VDomNode::Elem(Elem {
            name: "button".to_owned(),
            attrs: HashMap::from([("click".to_owned(), Attr::EventHandler(listener))]),
            children: Vec::from([(VDom {
                vdom: VDomNode::Text(Text { text: s, state: () }),
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
