extern crate parity_wasm;

use parity_wasm::{elements, builder};

fn convert_init(inc : u32, expr : &elements::InitExpr) -> elements::InitExpr {
    use parity_wasm::elements::Opcode::*;
    let mut vec = expr.code().to_vec();
    // println!("Got vec {:?} with len  {}", vec, vec.len());
    assert!(vec.len() == 2);
    if let I32Const(x) = vec[0] {
        vec[0] = I32Const(inc as i32 + x);
    }
    elements::InitExpr::new(vec)
}

fn convert_op(inc : u32, expr : &elements::Opcode) -> elements::Opcode {
    use parity_wasm::elements::Opcode::*;
    match expr {
    	&F32Load(flag, offset) => F32Load(flag, offset+inc),
    	&F64Load(flag, offset) => F64Load(flag, offset+inc),
        
    	&I32Load(flag, offset) => I32Load(flag, offset+inc),
    	&I32Load8S(flag, offset) => I32Load8S(flag, offset+inc),
    	&I32Load16S(flag, offset) => I32Load16S(flag, offset+inc),
    	&I32Load8U(flag, offset) => I32Load8U(flag, offset+inc),
    	&I32Load16U(flag, offset) => I32Load16U(flag, offset+inc),
    	&I64Load(flag, offset) => I64Load(flag, offset+inc),
    	&I64Load8S(flag, offset) => I64Load8S(flag, offset+inc),
    	&I64Load16S(flag, offset) => I64Load16S(flag, offset+inc),
    	&I64Load32S(flag, offset) => I64Load32S(flag, offset+inc),
    	&I64Load8U(flag, offset) => I64Load8U(flag, offset+inc),
    	&I64Load16U(flag, offset) => I64Load16U(flag, offset+inc),
    	&I64Load32U(flag, offset) => I64Load32U(flag, offset+inc),
        
    	&F32Store(flag, offset) => F32Store(flag, offset+inc),
    	&F64Store(flag, offset) => F64Store(flag, offset+inc),
    	&I32Store(flag, offset) => I32Store(flag, offset+inc),
    	&I32Store8(flag, offset) => I32Store8(flag, offset+inc),
    	&I32Store16(flag, offset) => I32Store16(flag, offset+inc),
    	&I64Store(flag, offset) => I64Store(flag, offset+inc),
    	&I64Store8(flag, offset) => I64Store8(flag, offset+inc),
    	&I64Store16(flag, offset) => I64Store16(flag, offset+inc),
    	&I64Store32(flag, offset) => I64Store32(flag, offset+inc),
        a => a.clone()
    }
}

// 
fn main() {
    let inc : u32 = 1024;
    let mut module = parity_wasm::deserialize_file("input.wasm").unwrap();
    assert!(module.code_section().is_some());
    {
        let code_section = module.code_section().unwrap(); // Part of the module with functions code

        let data_section = module.data_section().unwrap();

        println!("Function count in wasm file: {}", code_section.bodies().len());
        println!("Segment count in wasm file: {}", data_section.entries().len());
    }

    {
        
        for data_entry in module.data_section_mut().unwrap().entries_mut() {
            *data_entry = elements::DataSegment::new(0, convert_init(inc, data_entry.offset()), data_entry.value().to_vec())
        }
        
    }
    
    {
        for ref mut f in module.code_section_mut().unwrap().bodies_mut() {
            for op in f.code_mut().elements_mut() {
               *op = convert_op(inc, op);
            }
        }
        
    }

    parity_wasm::serialize_to_file("output.wasm", module).expect("Module serialization to succeed");
}

fn main2() {
    let inc : u32 = 1024;
    let mut module = parity_wasm::deserialize_file("input.wasm").unwrap();
    assert!(module.code_section().is_some());
    {
        let code_section = module.code_section().unwrap(); // Part of the module with functions code

        let data_section = module.data_section().unwrap();

        println!("Function count in wasm file: {}", code_section.bodies().len());
        println!("Segment count in wasm file: {}", data_section.entries().len());
    }

    {
        
        for data_entry in module.data_section_mut().unwrap().entries_mut() {
            *data_entry = elements::DataSegment::new(0, convert_init(inc, data_entry.offset()), data_entry.value().to_vec())
        }
        
    }
    
    {
        for ref mut f in module.code_section_mut().unwrap().bodies_mut() {
            for op in f.code_mut().elements_mut() {
               *op = convert_op(inc, op);
            }
        }
        
    }

    parity_wasm::serialize_to_file("output.wasm", module).expect("Module serialization to succeed");
}

