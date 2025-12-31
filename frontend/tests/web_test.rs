use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_basic_math() {
    let a = 1;
    let b = 1;
    assert_eq!(a + b, 2);
}
