extern crate parity_wasm;

fn main() {
    let module = parity_wasm::deserialize_file("./input.wasm").unwrap();
    assert!(module.code_section().is_some());

    let code_section = module.code_section().unwrap(); // Part of the module with functions code

    println!("Function count in wasm file: {}", code_section.bodies().len());
}
