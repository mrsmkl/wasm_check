extern crate parity_wasm;

use parity_wasm::{elements, builder};



fn convert_init(expr : &elements::InitExpr) -> elements::InitExpr {
    let vec = expr.code().to_vec();
    elements::InitExpr::new(vec)
}

// 
fn main() {
    let mut module = parity_wasm::deserialize_file("input.wasm").unwrap();
    assert!(module.code_section().is_some());
    {
        let code_section = module.code_section().unwrap(); // Part of the module with functions code

        let data_section = module.data_section().unwrap();

        println!("Function count in wasm file: {}", code_section.bodies().len());
        println!("Segment count in wasm file: {}", data_section.entries().len());
    }
    
    {
        // let mut data_section = module.data_section_mut().unwrap();
        // let mut entries = data_section.entries_mut();
        
        for data_entry in module.data_section_mut().unwrap().entries_mut() {
            *data_entry = elements::DataSegment::new(0, convert_init(data_entry.offset()), data_entry.value().to_vec())
        }
        
    }
    
    parity_wasm::serialize_to_file("output.wasm", module).expect("Module serialization to succeed");
}

