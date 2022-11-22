// Runner for vdom

// Runner takes in an initial vdom
// This VDOM doesn't have raw handlers, but they take some attr a

use crate::{
    host::Listener,
    vdom::{Attr, HashmapAttrs, VDomNode},
};

pub(crate) struct Handler<A> {
    handle: fn(&Runner<A>),
}

pub(crate) struct Runner<A> {
    send: fn(A),
}

pub(crate) struct VDomUI<A> {
    pub(crate) vdom: VDomNode<Vec<VDomUI<A>>, (), HashmapAttrs<Attr<Handler<A>>>>,
}

// pub(crate) fn mkListener<A>(h: &'static Handler<A>, r: &'static Runner<A>) -> Listener {
//     Listener {
//         handler: &(|| (h.handle)(r)),
//     }
// }
