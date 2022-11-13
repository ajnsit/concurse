use do_notation::{m, Lift};

fn foo() {
    let r = m! {
        x <- Some(1);
        y <- Some(2);
        z <- Some(3);
        return [x, y, z];
    };
    let x = m! {
        x <- V { a: 1 };
        return x;
    };

    assert_eq!(r, Some([1, 2, 3]));
}

struct V<A> {
    a: A,
}

impl<A> V<A> {
    fn and_then<B>(self, f: impl FnOnce(A) -> V<B>) -> V<B> {
        f(self.a)
    }
}

impl<A> Lift<A> for V<A> {
    fn lift(a: A) -> Self {
        V { a }
    }
}
