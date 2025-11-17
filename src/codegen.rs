use crate::ast::*;
use crate::types::TypeEnv;
use std::collections::HashMap;

/// Common Intermediate Representation
#[derive(Debug, Clone)]
pub struct IRModule {
    pub functions: Vec<IRFunction>,
    pub globals: Vec<IRGlobal>,
}

#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<IRParam>,
    pub return_type: IRType,
    pub body: Vec<IRInstruction>,
    pub local_count: usize,
}

#[derive(Debug, Clone)]
pub struct IRParam {
    pub name: String,
    pub ty: IRType,
}

#[derive(Debug, Clone)]
pub struct IRGlobal {
    pub name: String,
    pub ty: IRType,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IRType {
    I32,
    I64,
    Bool,
    Void,
    Ptr,
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    // Local variable operations
    LocalGet(usize),
    LocalSet(usize, Box<IRInstruction>),

    // Constants
    Const(i32),

    // Arithmetic
    Add(Box<IRInstruction>, Box<IRInstruction>),
    Sub(Box<IRInstruction>, Box<IRInstruction>),
    Mul(Box<IRInstruction>, Box<IRInstruction>),
    Div(Box<IRInstruction>, Box<IRInstruction>),
    Mod(Box<IRInstruction>, Box<IRInstruction>),
    Neg(Box<IRInstruction>),

    // Comparison
    Eq(Box<IRInstruction>, Box<IRInstruction>),
    Ne(Box<IRInstruction>, Box<IRInstruction>),
    Lt(Box<IRInstruction>, Box<IRInstruction>),
    Le(Box<IRInstruction>, Box<IRInstruction>),
    Gt(Box<IRInstruction>, Box<IRInstruction>),
    Ge(Box<IRInstruction>, Box<IRInstruction>),

    // Logical
    And(Box<IRInstruction>, Box<IRInstruction>),
    Or(Box<IRInstruction>, Box<IRInstruction>),
    Not(Box<IRInstruction>),

    // Control flow
    If(Box<IRInstruction>, Vec<IRInstruction>, Vec<IRInstruction>),
    Block(Vec<IRInstruction>),
    Return(Box<IRInstruction>),

    // Function calls
    Call(String, Vec<IRInstruction>),

    // Memory operations
    Load(Box<IRInstruction>),
    Store(Box<IRInstruction>, Box<IRInstruction>),

    // Array operations
    ArrayNew(usize),
    ArrayGet(Box<IRInstruction>, Box<IRInstruction>),
    ArraySet(Box<IRInstruction>, Box<IRInstruction>, Box<IRInstruction>),
    ArrayLen(Box<IRInstruction>),

    // Special
    Nop,
}

pub struct CodeGenerator {
    local_map: HashMap<String, usize>,
    local_count: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            local_map: HashMap::new(),
            local_count: 0,
        }
    }

    pub fn generate(&mut self, module: &Module, env: &TypeEnv) -> IRModule {
        let mut functions = Vec::new();

        for item in &module.items {
            if let Item::FunctionDef(func) = item {
                functions.push(self.generate_function(func));
            }
        }

        IRModule {
            functions,
            globals: Vec::new(),
        }
    }

    fn generate_function(&mut self, func: &FunctionDef) -> IRFunction {
        self.local_map.clear();
        self.local_count = 0;

        // Add parameters as locals
        for param in &func.params {
            self.local_map.insert(param.clone(), self.local_count);
            self.local_count += 1;
        }

        let body = self.generate_expr(&func.body);

        IRFunction {
            name: func.name.clone(),
            params: func
                .params
                .iter()
                .map(|p| IRParam {
                    name: p.clone(),
                    ty: IRType::I32, // Simplified
                })
                .collect(),
            return_type: self.convert_type(&func.type_sig.return_type),
            body: vec![IRInstruction::Return(Box::new(body))],
            local_count: self.local_count,
        }
    }

    fn generate_expr(&mut self, expr: &Expr) -> IRInstruction {
        match expr {
            Expr::Literal(Literal::Int(n)) => IRInstruction::Const(*n),
            Expr::Literal(Literal::Bool(true)) => IRInstruction::Const(1),
            Expr::Literal(Literal::Bool(false)) => IRInstruction::Const(0),
            Expr::Literal(Literal::Unit) => IRInstruction::Const(0),
            Expr::Literal(Literal::String(_)) => IRInstruction::Const(0), // Simplified

            Expr::Var(name) => {
                if let Some(&local_idx) = self.local_map.get(name) {
                    IRInstruction::LocalGet(local_idx)
                } else {
                    IRInstruction::Const(0)
                }
            }

            Expr::BinOp(op, left, right) => {
                let left_ir = self.generate_expr(left);
                let right_ir = self.generate_expr(right);
                match op {
                    BinOp::Add => IRInstruction::Add(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Sub => IRInstruction::Sub(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Mul => IRInstruction::Mul(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Div => IRInstruction::Div(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Mod => IRInstruction::Mod(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Eq => IRInstruction::Eq(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Ne => IRInstruction::Ne(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Lt => IRInstruction::Lt(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Le => IRInstruction::Le(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Gt => IRInstruction::Gt(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Ge => IRInstruction::Ge(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::And => IRInstruction::And(Box::new(left_ir), Box::new(right_ir)),
                    BinOp::Or => IRInstruction::Or(Box::new(left_ir), Box::new(right_ir)),
                }
            }

            Expr::UnOp(op, operand) => {
                let operand_ir = self.generate_expr(operand);
                match op {
                    UnOp::Neg => IRInstruction::Neg(Box::new(operand_ir)),
                    UnOp::Not => IRInstruction::Not(Box::new(operand_ir)),
                }
            }

            Expr::If(cond, then_branch, else_branch) => {
                let cond_ir = self.generate_expr(cond);
                let then_ir = vec![self.generate_expr(then_branch)];
                let else_ir = vec![self.generate_expr(else_branch)];
                IRInstruction::If(Box::new(cond_ir), then_ir, else_ir)
            }

            Expr::Let(var, value, body) => {
                let value_ir = self.generate_expr(value);
                let local_idx = self.local_count;
                self.local_map.insert(var.clone(), local_idx);
                self.local_count += 1;

                let set_ir = IRInstruction::LocalSet(local_idx, Box::new(value_ir));
                let body_ir = self.generate_expr(body);

                IRInstruction::Block(vec![set_ir, body_ir])
            }

            Expr::Call(name, args) => {
                let args_ir: Vec<IRInstruction> = args.iter().map(|arg| self.generate_expr(arg)).collect();
                IRInstruction::Call(name.clone(), args_ir)
            }

            Expr::Match(scrutinee, arms) => {
                // Simplified: just use the first arm's body
                if !arms.is_empty() {
                    self.generate_expr(&arms[0].body)
                } else {
                    IRInstruction::Const(0)
                }
            }

            Expr::Unsafe(inner) => self.generate_expr(inner),

            Expr::ParFor(start, end, body) => {
                // Generate a call to the runtime par_for
                let start_ir = self.generate_expr(start);
                let end_ir = self.generate_expr(end);
                IRInstruction::Call(
                    "par_for".to_string(),
                    vec![start_ir, end_ir],
                )
            }

            Expr::ParMap(func, array) => {
                let array_ir = self.generate_expr(array);
                IRInstruction::Call("par_map".to_string(), vec![array_ir])
            }

            Expr::ParMapInplace(func, src, dst) => {
                let src_ir = self.generate_expr(src);
                let dst_ir = self.generate_expr(dst);
                IRInstruction::Call("par_map_inplace".to_string(), vec![src_ir, dst_ir])
            }

            Expr::Async(inner) => {
                let inner_ir = self.generate_expr(inner);
                IRInstruction::Call("async".to_string(), vec![inner_ir])
            }

            Expr::Await(task) => {
                let task_ir = self.generate_expr(task);
                IRInstruction::Call("await".to_string(), vec![task_ir])
            }

            Expr::ActorCreate(actor_type, initial_state) => {
                let state_ir = self.generate_expr(initial_state);
                IRInstruction::Call("actor_create".to_string(), vec![state_ir])
            }

            Expr::ActorSend(actor, message, args) => {
                let actor_ir = self.generate_expr(actor);
                IRInstruction::Call("actor_send".to_string(), vec![actor_ir])
            }

            Expr::NewArray(space, arena, size) => {
                let size_ir = self.generate_expr(size);
                match size_ir {
                    IRInstruction::Const(n) => IRInstruction::ArrayNew(n as usize),
                    _ => IRInstruction::ArrayNew(0),
                }
            }

            Expr::ArrayGet(array, index) => {
                let array_ir = self.generate_expr(array);
                let index_ir = self.generate_expr(index);
                IRInstruction::ArrayGet(Box::new(array_ir), Box::new(index_ir))
            }

            Expr::ArraySet(array, index, value) => {
                let array_ir = self.generate_expr(array);
                let index_ir = self.generate_expr(index);
                let value_ir = self.generate_expr(value);
                IRInstruction::ArraySet(Box::new(array_ir), Box::new(index_ir), Box::new(value_ir))
            }

            Expr::ArrayLen(array) => {
                let array_ir = self.generate_expr(array);
                IRInstruction::ArrayLen(Box::new(array_ir))
            }

            Expr::GpuKernel(inner) => {
                let inner_ir = self.generate_expr(inner);
                IRInstruction::Call("gpu_kernel".to_string(), vec![inner_ir])
            }

            Expr::CpuToGpu(data) => {
                let data_ir = self.generate_expr(data);
                IRInstruction::Call("cpu_to_gpu".to_string(), vec![data_ir])
            }

            Expr::GpuToCpu(data) => {
                let data_ir = self.generate_expr(data);
                IRInstruction::Call("gpu_to_cpu".to_string(), vec![data_ir])
            }

            Expr::Log(inner) => {
                let inner_ir = self.generate_expr(inner);
                IRInstruction::Call("log".to_string(), vec![inner_ir])
            }

            Expr::Assert(inner) => {
                let inner_ir = self.generate_expr(inner);
                IRInstruction::Call("assert".to_string(), vec![inner_ir])
            }

            Expr::Print(inner) => {
                let inner_ir = self.generate_expr(inner);
                IRInstruction::Call("print".to_string(), vec![inner_ir])
            }

            Expr::RawThreadSpawn(func) => {
                let func_ir = self.generate_expr(func);
                IRInstruction::Call("raw_thread_spawn".to_string(), vec![func_ir])
            }

            Expr::AtomicLoad(addr) => {
                let addr_ir = self.generate_expr(addr);
                IRInstruction::Call("atomic_load".to_string(), vec![addr_ir])
            }

            Expr::AtomicStore(addr, value) => {
                let addr_ir = self.generate_expr(addr);
                let value_ir = self.generate_expr(value);
                IRInstruction::Call("atomic_store".to_string(), vec![addr_ir, value_ir])
            }
        }
    }

    fn convert_type(&self, ty: &Type) -> IRType {
        match ty {
            Type::I32 => IRType::I32,
            Type::Bool => IRType::Bool,
            Type::Unit => IRType::Void,
            _ => IRType::I32, // Simplified
        }
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::typecheck::TypeChecker;

    #[test]
    fn test_codegen_simple() {
        let input = "inc: i32 -> i32\ninc x = x + 1";
        let mut parser = Parser::new(input);
        let module = parser.parse_module().unwrap();

        let mut checker = TypeChecker::new();
        let env = checker.check_module(&module).unwrap();

        let mut codegen = CodeGenerator::new();
        let ir = codegen.generate(&module, &env);

        assert_eq!(ir.functions.len(), 1);
        assert_eq!(ir.functions[0].name, "inc");
    }
}
