pub mod host;
pub mod log;
pub mod machine;
pub mod sample;
pub mod vdom;

use sample::test;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// #[wasm_bindgen]
// Called by our JS entry point to run the example
#[wasm_bindgen(start)]
pub fn main() {
    test("Mr. Bond");
    // alert(&format!("Hello, {}!", name));
}
