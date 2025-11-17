use crate::codegen::*;
use cranelift::prelude::*;
use cranelift_codegen::ir::UserFuncName;
use cranelift_codegen::isa::CallConv;
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub struct X86Backend {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    module: ObjectModule,
    func_map: HashMap<String, FuncId>,
}

impl X86Backend {
    pub fn new() -> Self {
        let isa_builder = cranelift_codegen::isa::lookup(target_lexicon::Triple::host())
            .expect("Failed to look up target ISA");
        let isa = isa_builder
            .finish(settings::Flags::new(settings::builder()))
            .expect("Failed to create ISA");

        let builder = ObjectBuilder::new(
            isa,
            "flux_module",
            cranelift_module::default_libcall_names(),
        )
        .expect("Failed to create object builder");

        let module = ObjectModule::new(builder);

        X86Backend {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
            func_map: HashMap::new(),
        }
    }

    pub fn compile_to_object(mut self, ir_module: &IRModule, output_path: &Path) -> Result<(), String> {
        // First pass: declare all functions
        for func in &ir_module.functions {
            self.declare_function(func)?;
        }

        // Second pass: define all functions
        for func in &ir_module.functions {
            self.define_function(func)?;
        }

        // Generate object file
        let product = self.module.finish();
        let bytes = product.emit().map_err(|e| format!("Failed to emit object file: {}", e))?;

        let mut file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format!("Failed to write object file: {}", e))?;

        Ok(())
    }

    pub fn compile_to_executable(
        mut self,
        ir_module: &IRModule,
        output_path: &Path,
    ) -> Result<(), String> {
        // First pass: declare all functions
        for func in &ir_module.functions {
            self.declare_function(func)?;
        }

        // Second pass: define all functions
        for func in &ir_module.functions {
            self.define_function(func)?;
        }

        // Generate object file
        let obj_path = output_path.with_extension("o");
        let product = self.module.finish();
        let bytes = product.emit().map_err(|e| format!("Failed to emit object file: {}", e))?;

        let mut file = File::create(&obj_path)
            .map_err(|e| format!("Failed to create object file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format!("Failed to write object file: {}", e))?;
        drop(file);

        // Link with system linker
        Self::link_object(&obj_path, output_path)?;

        // Clean up object file
        std::fs::remove_file(&obj_path).ok();

        Ok(())
    }

    fn link_object(obj_path: &Path, output_path: &Path) -> Result<(), String> {
        // Try to use system C compiler as linker (works cross-platform)
        let linkers = ["cc", "gcc", "clang"];

        for linker in &linkers {
            let output = Command::new(linker)
                .arg("-o")
                .arg(output_path)
                .arg(obj_path)
                .arg("-lm") // Link math library
                .arg("-lpthread") // Link pthread
                .output();

            if let Ok(result) = output {
                if result.status.success() {
                    return Ok(());
                }
            }
        }

        Err("Failed to link object file. Make sure a C compiler (cc/gcc/clang) is installed.".to_string())
    }

    fn declare_function(&mut self, func: &IRFunction) -> Result<(), String> {
        let mut sig = self.module.make_signature();

        for param in &func.params {
            sig.params.push(AbiParam::new(self.convert_type(&param.ty)));
        }

        sig.returns.push(AbiParam::new(self.convert_type(&func.return_type)));

        let func_id = self
            .module
            .declare_function(&func.name, Linkage::Export, &sig)
            .map_err(|e| format!("Failed to declare function: {}", e))?;

        self.func_map.insert(func.name.clone(), func_id);

        Ok(())
    }

    fn define_function(&mut self, func: &IRFunction) -> Result<(), String> {
        let func_id = *self.func_map.get(&func.name).unwrap();

        self.ctx.func.signature.clear(CallConv::SystemV);
        self.ctx.func.name = UserFuncName::user(0, func_id.as_u32());

        for param in &func.params {
            self.ctx.func.signature.params.push(AbiParam::new(self.convert_type(&param.ty)));
        }

        self.ctx.func.signature.returns.push(AbiParam::new(self.convert_type(&func.return_type)));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let mut locals: Vec<Variable> = Vec::new();
        for i in 0..func.local_count {
            let var = Variable::new(i);
            builder.declare_var(var, types::I32);
            locals.push(var);
        }

        // Set parameter values
        for (i, _param) in func.params.iter().enumerate() {
            let val = builder.block_params(entry_block)[i];
            builder.def_var(locals[i], val);
        }

        // Generate function body
        for instr in &func.body {
            let val = self.compile_instruction(instr, &mut builder, &locals)?;
            if matches!(instr, IRInstruction::Return(_)) {
                builder.ins().return_(&[val]);
            }
        }

        builder.finalize();

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| format!("Failed to define function: {}", e))?;

        self.module.clear_context(&mut self.ctx);

        Ok(())
    }

    fn compile_instruction(
        &self,
        instr: &IRInstruction,
        builder: &mut FunctionBuilder,
        locals: &[Variable],
    ) -> Result<Value, String> {
        match instr {
            IRInstruction::Const(n) => Ok(builder.ins().iconst(types::I32, *n as i64)),

            IRInstruction::LocalGet(idx) => Ok(builder.use_var(locals[*idx])),

            IRInstruction::LocalSet(idx, value) => {
                let val = self.compile_instruction(value, builder, locals)?;
                builder.def_var(locals[*idx], val);
                Ok(val)
            }

            IRInstruction::Add(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                Ok(builder.ins().iadd(lhs, rhs))
            }

            IRInstruction::Sub(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                Ok(builder.ins().isub(lhs, rhs))
            }

            IRInstruction::Mul(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                Ok(builder.ins().imul(lhs, rhs))
            }

            IRInstruction::Div(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                Ok(builder.ins().sdiv(lhs, rhs))
            }

            IRInstruction::Mod(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                Ok(builder.ins().srem(lhs, rhs))
            }

            IRInstruction::Neg(operand) => {
                let val = self.compile_instruction(operand, builder, locals)?;
                Ok(builder.ins().ineg(val))
            }

            IRInstruction::Eq(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                let cmp = builder.ins().icmp(IntCC::Equal, lhs, rhs);
                Ok(builder.ins().uextend(types::I32, cmp))
            }

            IRInstruction::Ne(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                let cmp = builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
                Ok(builder.ins().uextend(types::I32, cmp))
            }

            IRInstruction::Lt(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                let cmp = builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
                Ok(builder.ins().uextend(types::I32, cmp))
            }

            IRInstruction::Le(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
                Ok(builder.ins().uextend(types::I32, cmp))
            }

            IRInstruction::Gt(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                Ok(builder.ins().uextend(types::I32, cmp))
            }

            IRInstruction::Ge(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
                Ok(builder.ins().uextend(types::I32, cmp))
            }

            IRInstruction::And(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                Ok(builder.ins().band(lhs, rhs))
            }

            IRInstruction::Or(left, right) => {
                let lhs = self.compile_instruction(left, builder, locals)?;
                let rhs = self.compile_instruction(right, builder, locals)?;
                Ok(builder.ins().bor(lhs, rhs))
            }

            IRInstruction::Not(operand) => {
                let val = self.compile_instruction(operand, builder, locals)?;
                let zero = builder.ins().iconst(types::I32, 0);
                let cmp = builder.ins().icmp(IntCC::Equal, val, zero);
                Ok(builder.ins().uextend(types::I32, cmp))
            }

            IRInstruction::If(cond, then_block, else_block) => {
                let cond_val = self.compile_instruction(cond, builder, locals)?;
                let zero = builder.ins().iconst(types::I32, 0);
                let cmp = builder.ins().icmp(IntCC::NotEqual, cond_val, zero);

                let then_bb = builder.create_block();
                let else_bb = builder.create_block();
                let merge_bb = builder.create_block();

                builder.append_block_param(merge_bb, types::I32);

                builder.ins().brif(cmp, then_bb, &[], else_bb, &[]);

                builder.switch_to_block(then_bb);
                builder.seal_block(then_bb);
                let mut then_val = builder.ins().iconst(types::I32, 0);
                for instr in then_block {
                    then_val = self.compile_instruction(instr, builder, locals)?;
                }
                builder.ins().jump(merge_bb, &[then_val]);

                builder.switch_to_block(else_bb);
                builder.seal_block(else_bb);
                let mut else_val = builder.ins().iconst(types::I32, 0);
                for instr in else_block {
                    else_val = self.compile_instruction(instr, builder, locals)?;
                }
                builder.ins().jump(merge_bb, &[else_val]);

                builder.switch_to_block(merge_bb);
                builder.seal_block(merge_bb);

                Ok(builder.block_params(merge_bb)[0])
            }

            IRInstruction::Block(instrs) => {
                let mut last_val = builder.ins().iconst(types::I32, 0);
                for instr in instrs {
                    last_val = self.compile_instruction(instr, builder, locals)?;
                }
                Ok(last_val)
            }

            IRInstruction::Return(value) => {
                let val = self.compile_instruction(value, builder, locals)?;
                Ok(val)
            }

            IRInstruction::Call(name, args) => {
                // Look up the function
                if let Some(&callee_id) = self.func_map.get(name) {
                    let mut sig = self.module.make_signature();
                    for _ in args {
                        sig.params.push(AbiParam::new(types::I32));
                    }
                    sig.returns.push(AbiParam::new(types::I32));

                    let callee_ref = self.module.declare_func_in_func(callee_id, builder.func);

                    let arg_vals: Result<Vec<_>, _> = args
                        .iter()
                        .map(|arg| self.compile_instruction(arg, builder, locals))
                        .collect();
                    let arg_vals = arg_vals?;

                    let call = builder.ins().call(callee_ref, &arg_vals);
                    let results = builder.inst_results(call);
                    if !results.is_empty() {
                        Ok(results[0])
                    } else {
                        Ok(builder.ins().iconst(types::I32, 0))
                    }
                } else {
                    // Unknown function - return 0
                    Ok(builder.ins().iconst(types::I32, 0))
                }
            }

            IRInstruction::ArrayNew(size) => {
                Ok(builder.ins().iconst(types::I32, *size as i64))
            }

            IRInstruction::ArrayGet(array, index) => {
                let _array_val = self.compile_instruction(array, builder, locals)?;
                let _index_val = self.compile_instruction(index, builder, locals)?;
                // Simplified - would need proper array implementation
                Ok(builder.ins().iconst(types::I32, 0))
            }

            IRInstruction::ArraySet(array, index, value) => {
                let _array_val = self.compile_instruction(array, builder, locals)?;
                let _index_val = self.compile_instruction(index, builder, locals)?;
                let _value_val = self.compile_instruction(value, builder, locals)?;
                Ok(builder.ins().iconst(types::I32, 0))
            }

            IRInstruction::ArrayLen(array) => {
                let _array_val = self.compile_instruction(array, builder, locals)?;
                Ok(builder.ins().iconst(types::I32, 0))
            }

            IRInstruction::Load(addr) => {
                let _addr_val = self.compile_instruction(addr, builder, locals)?;
                Ok(builder.ins().iconst(types::I32, 0))
            }

            IRInstruction::Store(addr, value) => {
                let _addr_val = self.compile_instruction(addr, builder, locals)?;
                let _value_val = self.compile_instruction(value, builder, locals)?;
                Ok(builder.ins().iconst(types::I32, 0))
            }

            IRInstruction::Nop => Ok(builder.ins().iconst(types::I32, 0)),
        }
    }

    fn convert_type(&self, ty: &IRType) -> types::Type {
        match ty {
            IRType::I32 | IRType::Bool => types::I32,
            IRType::I64 => types::I64,
            IRType::Void => types::I32, // Return dummy i32 for void
            IRType::Ptr => types::I64,
        }
    }
}

impl Default for X86Backend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_compile_to_object() {
        let mut backend = X86Backend::new();

        let func = IRFunction {
            name: "add".to_string(),
            params: vec![
                IRParam {
                    name: "x".to_string(),
                    ty: IRType::I32,
                },
                IRParam {
                    name: "y".to_string(),
                    ty: IRType::I32,
                },
            ],
            return_type: IRType::I32,
            body: vec![IRInstruction::Return(Box::new(IRInstruction::Add(
                Box::new(IRInstruction::LocalGet(0)),
                Box::new(IRInstruction::LocalGet(1)),
            )))],
            local_count: 2,
        };

        let ir_module = IRModule {
            functions: vec![func],
            globals: vec![],
        };

        let temp_dir = std::env::temp_dir();
        let obj_path = temp_dir.join("test_flux.o");

        backend.compile_to_object(&ir_module, &obj_path).unwrap();
        assert!(obj_path.exists());

        std::fs::remove_file(obj_path).ok();
    }
}
