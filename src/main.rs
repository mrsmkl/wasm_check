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

fn simple_init(inc : u32) -> elements::InitExpr {
    use parity_wasm::elements::Opcode::*;
    elements::InitExpr::new([I32Const(inc as i32)].to_vec())
}

fn int_binary(i : u32) -> Vec<u8> {
    let b1 = i as u8;
    let b2 = (i >> 8) as u8;
    let b3 = (i >> 16) as u8;
    let b4 = (i >> 24) as u8;
    [b1, b2, b3, b4].to_vec()
}

fn count_func_imports(b : &elements::Module) -> u32 {
    if let Some(s) = b.import_section() {
        let mut k = 0;
        for im in s.entries().iter() {
            if let &elements::External::Function(_x) = im.external() { k = k + 1 }
        };
        k
    }
    else { 0 }
}

fn remap_export<F>(e : &elements::ExportEntry, f_remap : &F) -> elements::ExportEntry
where F: Fn (u32) -> u32 {
    match e.internal() {
       &elements::Internal::Function(t) => elements::ExportEntry::new(e.field().clone().to_string(), elements::Internal::Function(f_remap(t))),
       _ => e.clone()
    }
}

fn remap_opcode<F, F2>(e : &elements::Opcode, f_remap : &F, ft_remap : &F2) -> elements::Opcode
where F: Fn (u32) -> u32, F2: Fn (u32) -> u32 {
    use parity_wasm::elements::Opcode::*;
    match e {
        &Call(v) => Call(f_remap(v)),
        &CallIndirect(v, h) => CallIndirect(ft_remap(v), h),
        a => a.clone()
    }
}

fn remap_body<F, F2>(e : &elements::FuncBody, f_remap : &F, ft_remap : &F2) -> elements::FuncBody
where F: Fn (u32) -> u32, F2: Fn (u32) -> u32 {
    let ops = e.code().elements().iter().map(|a| remap_opcode(a, f_remap, ft_remap)).collect();
    elements::FuncBody::new(e.locals().to_vec().clone(), elements::Opcodes::new(ops))
}

fn merge(a : &elements::Module, b : &elements::Module, offset : u32) -> elements::Module {
    let builder = builder::module().with_module(a.clone());
    let builder = if let Some(gs) = b.global_section() {
       gs.entries().iter().fold(builder, |builder, g| { builder.with_global(g.clone()) })
    }
    else { builder };
    let builder = if let Some(gs) = b.type_section() {
       gs.types().iter().fold(builder, |builder, g| { builder.with_type(g.clone()) })
    }
    else { builder };
    // shift these signatures
    let a_ft_len = if let Some(ts) = a.type_section() {
       ts.types().len() as u32
    }
    else { 0 };
    let builder = if let Some(gs) = b.function_section() {
       gs.entries().iter().fold(builder, |builder, g| { builder.with_func_sig(elements::Func::new(g.type_ref() + a_ft_len)) })
    }
    else { builder };
    let builder = if let Some(gs) = b.data_section() {
       gs.entries().iter().fold(builder, |builder, g| { builder.with_data(g.clone()) })
    }
    else { builder };
    let builder = builder.with_data(elements::DataSegment::new(0, simple_init(256*4), int_binary(offset)));
    
    let a_num_funcs = if let Some(s) = b.code_section() {
       s.bodies().len() as u32
    }
    else { 0 };
    let a_func_len = count_func_imports(a) + a_num_funcs;

    let builder = if let Some(gs) = b.export_section() {
       gs.entries().iter().fold(builder, |builder, g| { builder.with_export(remap_export(g, &|x| { a_func_len + x } )) })
    }
    else { builder };

    let builder = if let Some(gs) = b.code_section() {
       gs.bodies().iter().fold(builder, |builder, g| { builder.with_func_body(remap_body(g, &|x| { a_func_len + x } , &|x| { a_ft_len + x } )) })
    }
    else { builder };

    builder.build()
}

fn convert_type(vt : &elements::ValueType) -> elements::ValueType {
    use elements::ValueType::*;
    match vt {
        &I32 => I32,
        &F32 => I32,
        &I64 => I64,
        &F64 => I64
    }
}

// function type
fn convert_ftype(ft : &elements::FunctionType) -> elements::FunctionType {
    let ret =
       if let Some(t) = ft.return_type() { Some(convert_type(&t)) }
       else { None };
    let params = ft.params().iter().map(&|a| convert_type(a)).collect();
    elements::FunctionType::new(params, ret)
}

fn convert_gtype(ft : &elements::GlobalType) -> elements::GlobalType {
    elements::GlobalType::new(convert_type(&ft.content_type()), ft.is_mutable())
}

fn convert_block_type(ft : &elements::BlockType) -> elements::BlockType {
    use elements::BlockType::*;
    match ft {
        &Value(ref t) => Value(convert_type(t)),
        &NoResult => NoResult
    }
}

// opcode
fn convert_opcode(op : &elements::Opcode) -> elements::Opcode {
    use elements::Opcode::*;
    match op {
        &Block(ref bt) => Block(convert_block_type(bt)),
   	    &Loop(ref bt) => Loop(convert_block_type(bt)),
	    &If(ref bt) => If(convert_block_type(bt)),
        a => a.clone()
    }
}

fn convert_local(l : &elements::Local) -> elements::Local {
    elements::Local::new(l.count(), convert_type(&l.value_type()))
}

// function body
// !!! probably will need to work more on initializing floating point values
// how to find the number of parameters?
fn convert_body(params : u32, bd : &elements::FuncBody) -> elements::FuncBody {
    use elements::ValueType::*;
    use elements::Opcode::*;
    let mut v = Vec::new();
    let mut acc = params;
    for i in bd.locals().iter() {
        match &i.value_type() {
            &F32 =>
              for n in 0..i.count()-1 {
                  v.push(I32Const(0.0_f32.to_bits() as i32));
                  v.push(SetLocal(n+acc))
              },
            &F64 =>
              for n in 0..i.count()-1 {
                  v.push(I64Const(0.0_f64.to_bits() as i64));
                  v.push(SetLocal(n+acc))
              },
            _ => {}
        };
        acc += i.count()
    };
    v.append(&mut bd.code().elements().iter().map(|a| convert_opcode(a)).collect());
    elements::FuncBody::new(bd.locals().iter().map(|l| convert_local(l)).collect(), elements::Opcodes::new(v))
}

fn test_clear(a : &elements::Section) -> bool {
    match a {
        &elements::Section::Code(_) => false,
        &elements::Section::Type(_) => false,
        _ => true
    }
}

fn clear_module(a : &elements::Module) -> elements::Module {
    elements::Module::new(a.sections().iter().filter(|a| test_clear(a)).map(|a| a.clone()).collect())
}

fn func_params(a : &elements::Module, num : usize) -> u32 {
    let f = a.function_section().unwrap().entries()[num].type_ref();
    let elements::Type::Function(ref ft) = a.type_section().unwrap().types()[f as usize];
    ft.params().len() as u32
}

// all together
fn convert_module_types(a : &elements::Module) -> elements::Module {
    use elements::Type::*;
    let builder = builder::module().with_module(clear_module(a));
    let builder = if let Some(gs) = a.type_section() {
       gs.types().iter().fold(builder, |builder, &Function(ref g)| { builder.with_type(Function(convert_ftype(g))) })
    }
    else { builder };
    let mut acc = 0;
    let builder = if let Some(gs) = a.code_section() {
       gs.bodies().iter().fold(builder, |builder, g| { let builder = builder.with_func_body(convert_body(func_params(a, acc), g)); acc += 1; builder })
    }
    else { builder };
    builder.build()
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

fn find_function(m : &elements::Module, str : &str) -> u32 {
   m.export_section().unwrap().entries().iter().map(|a| match a.internal() {
      &elements::Internal::Function(v) => if a.field() == str { Some(v) }  else { None },
      _ => None
   }).find(|a| if let &Some(_) = a {true} else {false}).unwrap().unwrap().clone()
}

fn fpu_emu_opcode(m : &elements::Module, a : &elements::Opcode) -> elements::Opcode {
    use parity_wasm::elements::Opcode::*;
    match a {
    	&F32Load(flag, offset) => I32Load(flag, offset),
    	&F64Load(flag, offset) => I64Load(flag, offset),
    	&F32Store(flag, offset) => I32Store(flag, offset),
    	&F64Store(flag, offset) => I64Store(flag, offset),
        
        &F32Add => Call(find_function(m, "f32_add")),
        &F32Div => Call(find_function(m, "f32_div")),
        &F32Mul => Call(find_function(m, "f32_mul")),
        &F32Sub => Call(find_function(m, "f32_sub")),
        &F32Min => Call(find_function(m, "f32_min")),
        &F32Max => Call(find_function(m, "f32_max")),
        &F32Sqrt => Call(find_function(m, "f32_sqrt")),
        &F32Nearest => Call(find_function(m, "f32_nearest")),
        &F32Ceil => Call(find_function(m, "f32_ceil")),
        &F32Floor => Call(find_function(m, "f32_floor")),
        &F32Trunc => Call(find_function(m, "f32_trunc")),
        &F32Neg => Call(find_function(m, "f32_neg")),
        &F32Abs => Call(find_function(m, "f32_abs")),
        &F32Eq => Call(find_function(m, "f32_eq")),
        &F32Le => Call(find_function(m, "f32_le")),
        &F32Lt => Call(find_function(m, "f32_lt")),
        &F32Ne => Call(find_function(m, "f32_ne")),
        &F32Ge => Call(find_function(m, "f32_ge")),
        &F32Gt => Call(find_function(m, "f32_gt")),
        &F32Copysign => Call(find_function(m, "f32_copysign")),
        
        &F64Add => Call(find_function(m, "f64_add")),
        &F64Div => Call(find_function(m, "f64_div")),
        &F64Mul => Call(find_function(m, "f64_mul")),
        &F64Sub => Call(find_function(m, "f64_sub")),
        &F64Min => Call(find_function(m, "f64_min")),
        &F64Max => Call(find_function(m, "f64_max")),
        &F64Sqrt => Call(find_function(m, "f64_sqrt")),
        &F64Nearest => Call(find_function(m, "f64_nearest")),
        &F64Ceil => Call(find_function(m, "f64_ceil")),
        &F64Floor => Call(find_function(m, "f64_floor")),
        &F64Trunc => Call(find_function(m, "f64_trunc")),
        &F64Neg => Call(find_function(m, "f64_neg")),
        &F64Abs => Call(find_function(m, "f64_abs")),
        &F64Eq => Call(find_function(m, "f64_eq")),
        &F64Le => Call(find_function(m, "f64_le")),
        &F64Lt => Call(find_function(m, "f64_lt")),
        &F64Ne => Call(find_function(m, "f64_ne")),
        &F64Ge => Call(find_function(m, "f64_ge")),
        &F64Gt => Call(find_function(m, "f64_gt")),
        &F64Copysign => Call(find_function(m, "f64_copysign")),
        
        &F32ReinterpretI32 => Nop,
        &F64ReinterpretI64 => Nop,
        &I32ReinterpretF32 => Nop,
        &I64ReinterpretF64 => Nop,
        
	    &I32TruncSF32 => Call(find_function(m, "f32_trunc_i32")),
	    &I32TruncUF32 => Call(find_function(m, "f32_trunc_ui32")),
	    &I32TruncSF64 => Call(find_function(m, "f64_trunc_i32")),
	    &I32TruncUF64 => Call(find_function(m, "f64_trunc_ui32")),
  	    &I64TruncSF32 => Call(find_function(m, "f32_trunc_i64")),
	    &I64TruncUF32 => Call(find_function(m, "f32_trunc_ui64")),
	    &I64TruncSF64 => Call(find_function(m, "f64_trunc_i64")),
	    &I64TruncUF64 => Call(find_function(m, "f64_trunc_ui64")),
	    &F32ConvertSI32 => Call(find_function(m, "i32_to_f32")),
	    &F32ConvertUI32 => Call(find_function(m, "ui32_to_f32")),
	    &F32ConvertSI64 => Call(find_function(m, "i64_to_f32")),
	    &F32ConvertUI64 => Call(find_function(m, "ui64_to_f32")),
	    &F32DemoteF64 => Call(find_function(m, "f64_to_f32")),
	    &F64ConvertSI32 => Call(find_function(m, "i32_to_f64")),
	    &F64ConvertUI32 => Call(find_function(m, "ui32_to_f64")),
	    &F64ConvertSI64 => Call(find_function(m, "i64_to_f64")),
	    &F64ConvertUI64 => Call(find_function(m, "ui64_to_f64")),
	    &F64PromoteF32 => Call(find_function(m, "f32_to_f64")),
        
        &F32Const(i) => I32Const(i as i32),
        &F64Const(i) => I64Const(i as i64),
        
        a => a.clone()
    }
}

fn fpu_emu_body(m : &elements::Module, bd : &elements::FuncBody) -> elements::FuncBody {
    elements::FuncBody::new(bd.locals().to_vec(), elements::Opcodes::new(bd.code().elements().iter().map(|a| fpu_emu_opcode(m, a)).collect()))
}

fn fpu_emu_global(m : &elements::Module, a : &elements::GlobalEntry) -> elements::GlobalEntry {
    elements::GlobalEntry::new(convert_gtype(a.global_type()), elements::InitExpr::new(a.init_expr().code().iter().map(|a| fpu_emu_opcode(m, a)).collect()))
}

fn fpu_emu_module(a : &elements::Module) -> elements::Module {
    let m = a;
    elements::Module::new(a.sections().iter().map(|a|
       match a {
           &elements::Section::Code(ref s) => elements::Section::Code(elements::CodeSection::with_bodies(s.bodies().iter().map(|a| fpu_emu_body(m, a)).collect())),
           &elements::Section::Global(ref s) => elements::Section::Global(elements::GlobalSection::with_entries(s.entries().iter().map(|a| fpu_emu_global(m, a)).collect())),
           s => s.clone()
       }).collect())
}


// 
fn shift_offset(inc : u32, module : &mut elements::Module) {
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

}

fn main() {
    let inc : u32 = 10240;
    let mut module = parity_wasm::deserialize_file("input.wasm").unwrap();
    let module2 = parity_wasm::deserialize_file("softfloat.wasm").unwrap();
    shift_offset(10240, &mut module);
    fpu_emu_module(&convert_module_types(&merge(&module, &module2, inc)));
    assert!(module.code_section().is_some());
    {
        let code_section = module.code_section().unwrap(); // Part of the module with functions code

        let data_section = module.data_section().unwrap();

        println!("Function count in wasm file: {}", code_section.bodies().len());
        println!("Segment count in wasm file: {}", data_section.entries().len());
    }

    parity_wasm::serialize_to_file("output.wasm", module).expect("Module serialization to succeed");
}

