// // This works, but we have to call main manually, which is not ideal
// // Also, if there is a wasm_bindgen(start) in the code, that will *also* get called!
// import {main} from './pkg';
// main();

// So instead, we don't call main manually, and only depend on the wasm_bindgen(start) section

// Why won't a straight import work when using wasm_bindgen(start)?
// import './pkg';

// Why do I need to do a dynamic import when using wasm_bindgen(start)?
import('./pkg');
