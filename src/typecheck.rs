use crate::ast::*;
use crate::effect::EffectChecker;
use crate::types::*;
use std::collections::HashMap;

pub struct TypeChecker {
    effect_checker: EffectChecker,
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
}

impl TypeError {
    fn new(msg: String) -> Self {
        TypeError { message: msg }
    }
}

type TypeResult<T> = Result<T, TypeError>;

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            effect_checker: EffectChecker::new(),
        }
    }

    pub fn check_module(&mut self, module: &Module) -> TypeResult<TypeEnv> {
        let mut env = TypeEnv::new();

        // First pass: collect all function signatures and data types
        for item in &module.items {
            match item {
                Item::FunctionDef(func) => {
                    env.add_function(func.name.clone(), func.type_sig.clone());
                }
                Item::DataDef(data) => {
                    env.add_data_type(data.name.clone(), data.clone());
                }
            }
        }

        // Second pass: type check all function bodies
        for item in &module.items {
            if let Item::FunctionDef(func) = item {
                self.check_function(func, &mut env)?;
            }
        }

        Ok(env)
    }

    fn check_function(&mut self, func: &FunctionDef, env: &mut TypeEnv) -> TypeResult<()> {
        let mut local_env = env.push_scope();

        // Add parameters to environment
        for (i, param) in func.params.iter().enumerate() {
            if i < func.type_sig.param_types.len() {
                local_env.add_var(param.clone(), func.type_sig.param_types[i].clone());
            }
        }

        // Check function body
        let body_type = self.infer_expr(&func.body, &local_env, &func.type_sig.effects)?;

        // Verify return type matches
        if !unify(&body_type, &func.type_sig.return_type) {
            return Err(TypeError::new(format!(
                "Function {} body type {:?} does not match declared return type {:?}",
                func.name, body_type, func.type_sig.return_type
            )));
        }

        // Verify effects
        let body_effects = self.effect_checker.infer_expr_effects(&func.body);
        if !func.type_sig.effects.includes(&body_effects) {
            return Err(TypeError::new(format!(
                "Function {} body has effects {:?} which are not included in declared effects {:?}",
                func.name, body_effects, func.type_sig.effects
            )));
        }

        Ok(())
    }

    fn infer_expr(
        &mut self,
        expr: &Expr,
        env: &TypeEnv,
        current_effects: &EffectSet,
    ) -> TypeResult<Type> {
        match expr {
            Expr::Literal(lit) => Ok(self.infer_literal(lit)),

            Expr::Var(name) => env
                .get_var(name)
                .cloned()
                .ok_or_else(|| TypeError::new(format!("Undefined variable: {}", name))),

            Expr::BinOp(op, left, right) => {
                let left_ty = self.infer_expr(left, env, current_effects)?;
                let right_ty = self.infer_expr(right, env, current_effects)?;
                self.infer_binop(op, &left_ty, &right_ty)
            }

            Expr::UnOp(op, operand) => {
                let operand_ty = self.infer_expr(operand, env, current_effects)?;
                self.infer_unop(op, &operand_ty)
            }

            Expr::If(cond, then_branch, else_branch) => {
                let cond_ty = self.infer_expr(cond, env, current_effects)?;
                if !unify(&cond_ty, &Type::Bool) {
                    return Err(TypeError::new(format!(
                        "If condition must be Bool, got {:?}",
                        cond_ty
                    )));
                }

                let then_ty = self.infer_expr(then_branch, env, current_effects)?;
                let else_ty = self.infer_expr(else_branch, env, current_effects)?;

                if !unify(&then_ty, &else_ty) {
                    return Err(TypeError::new(format!(
                        "If branches have incompatible types: {:?} and {:?}",
                        then_ty, else_ty
                    )));
                }

                Ok(then_ty)
            }

            Expr::Let(var, value, body) => {
                let value_ty = self.infer_expr(value, env, current_effects)?;
                let mut new_env = env.push_scope();
                new_env.add_var(var.clone(), value_ty);
                self.infer_expr(body, &new_env, current_effects)
            }

            Expr::Call(name, args) => {
                let func_sig = env
                    .get_function(name)
                    .ok_or_else(|| TypeError::new(format!("Undefined function: {}", name)))?
                    .clone();

                // Check effect compatibility
                if !current_effects.includes(&func_sig.effects) {
                    return Err(TypeError::new(format!(
                        "error: cannot call function `{}` with effect {:?}\n  from context with effect {:?}",
                        name, func_sig.effects, current_effects
                    )));
                }

                // Check argument types
                if args.len() != func_sig.param_types.len() {
                    return Err(TypeError::new(format!(
                        "Function {} expects {} arguments, got {}",
                        name,
                        func_sig.param_types.len(),
                        args.len()
                    )));
                }

                for (i, (arg, param_ty)) in args.iter().zip(&func_sig.param_types).enumerate() {
                    let arg_ty = self.infer_expr(arg, env, current_effects)?;
                    if !unify(&arg_ty, param_ty) {
                        return Err(TypeError::new(format!(
                            "Function {} argument {} has type {:?}, expected {:?}",
                            name, i, arg_ty, param_ty
                        )));
                    }
                }

                Ok((*func_sig.return_type).clone())
            }

            Expr::Match(scrutinee, arms) => {
                let scrutinee_ty = self.infer_expr(scrutinee, env, current_effects)?;

                if arms.is_empty() {
                    return Err(TypeError::new("Match must have at least one arm".to_string()));
                }

                let first_arm_ty = self.infer_expr(&arms[0].body, env, current_effects)?;

                for arm in &arms[1..] {
                    let arm_ty = self.infer_expr(&arm.body, env, current_effects)?;
                    if !unify(&arm_ty, &first_arm_ty) {
                        return Err(TypeError::new(format!(
                            "Match arms have incompatible types: {:?} and {:?}",
                            first_arm_ty, arm_ty
                        )));
                    }
                }

                Ok(first_arm_ty)
            }

            Expr::Unsafe(inner) => {
                // Inside unsafe blocks, we allow arbitrary effects
                let unsafe_effects = EffectSet {
                    purity: Purity::IO,
                    execution: Execution::Cpu,
                    allocation: Allocation::Heap,
                    concurrency: Concurrency::Concurrent,
                    debug: true,
                };
                self.infer_expr(inner, env, &unsafe_effects)
            }

            Expr::ParFor(start, end, body) => {
                let start_ty = self.infer_expr(start, env, current_effects)?;
                let end_ty = self.infer_expr(end, env, current_effects)?;

                if !unify(&start_ty, &Type::I32) || !unify(&end_ty, &Type::I32) {
                    return Err(TypeError::new(
                        "ParFor range must be i32".to_string(),
                    ));
                }

                // Body should be a function i32 -> Unit
                let body_ty = self.infer_expr(body, env, current_effects)?;

                Ok(Type::Unit)
            }

            Expr::ParMap(func, array) => {
                let func_ty = self.infer_expr(func, env, current_effects)?;
                let array_ty = self.infer_expr(array, env, current_effects)?;

                // Simplified - in reality we'd extract element types
                Ok(array_ty)
            }

            Expr::ParMapInplace(func, src, dst) => {
                let func_ty = self.infer_expr(func, env, current_effects)?;
                let src_ty = self.infer_expr(src, env, current_effects)?;
                let dst_ty = self.infer_expr(dst, env, current_effects)?;

                Ok(Type::Unit)
            }

            Expr::Async(inner) => {
                let async_effects = EffectSet {
                    purity: Purity::IO,
                    execution: Execution::Cpu,
                    allocation: Allocation::Heap,
                    concurrency: Concurrency::Concurrent,
                    debug: current_effects.debug,
                };

                let inner_ty = self.infer_expr(inner, env, &async_effects)?;
                Ok(Type::Task(Box::new(inner_ty)))
            }

            Expr::Await(task) => {
                let task_ty = self.infer_expr(task, env, current_effects)?;
                match task_ty {
                    Type::Task(inner) => Ok((*inner).clone()),
                    _ => Err(TypeError::new(format!(
                        "Await expects Task, got {:?}",
                        task_ty
                    ))),
                }
            }

            Expr::ActorCreate(actor_type, initial_state) => {
                let state_ty = self.infer_expr(initial_state, env, current_effects)?;
                Ok(Type::Actor(actor_type.clone()))
            }

            Expr::ActorSend(actor, message, args) => {
                let actor_ty = self.infer_expr(actor, env, current_effects)?;
                Ok(Type::Unit)
            }

            Expr::NewArray(space, arena, size) => {
                let size_ty = self.infer_expr(size, env, current_effects)?;
                if !unify(&size_ty, &Type::I32) {
                    return Err(TypeError::new("Array size must be i32".to_string()));
                }

                // Check allocation effect
                if current_effects.allocation == Allocation::None {
                    return Err(TypeError::new(
                        "error: cannot allocate array in alloc none context".to_string(),
                    ));
                }

                Ok(Type::Array(
                    Box::new(Type::I32),
                    space.clone(),
                    arena.clone(),
                ))
            }

            Expr::ArrayGet(array, index) => {
                let array_ty = self.infer_expr(array, env, current_effects)?;
                let index_ty = self.infer_expr(index, env, current_effects)?;

                if !unify(&index_ty, &Type::I32) {
                    return Err(TypeError::new("Array index must be i32".to_string()));
                }

                match array_ty {
                    Type::Array(elem_ty, _, _) => Ok((*elem_ty).clone()),
                    _ => Err(TypeError::new(format!(
                        "ArrayGet expects Array, got {:?}",
                        array_ty
                    ))),
                }
            }

            Expr::ArraySet(array, index, value) => {
                let array_ty = self.infer_expr(array, env, current_effects)?;
                let index_ty = self.infer_expr(index, env, current_effects)?;
                let value_ty = self.infer_expr(value, env, current_effects)?;

                if !unify(&index_ty, &Type::I32) {
                    return Err(TypeError::new("Array index must be i32".to_string()));
                }

                Ok(Type::Unit)
            }

            Expr::ArrayLen(array) => {
                let array_ty = self.infer_expr(array, env, current_effects)?;
                Ok(Type::I32)
            }

            Expr::GpuKernel(inner) => {
                let gpu_effects = EffectSet {
                    purity: current_effects.purity.clone(),
                    execution: Execution::Gpu,
                    allocation: current_effects.allocation.clone(),
                    concurrency: current_effects.concurrency.clone(),
                    debug: current_effects.debug,
                };

                self.infer_expr(inner, env, &gpu_effects)
            }

            Expr::CpuToGpu(data) | Expr::GpuToCpu(data) => {
                let data_ty = self.infer_expr(data, env, current_effects)?;
                Ok(data_ty)
            }

            Expr::Log(inner) | Expr::Assert(inner) | Expr::Print(inner) => {
                let _inner_ty = self.infer_expr(inner, env, current_effects)?;
                Ok(Type::I32)  // print returns the value it printed
            }

            Expr::RawThreadSpawn(func) => {
                let func_ty = self.infer_expr(func, env, current_effects)?;
                Ok(Type::Unit)
            }

            Expr::AtomicLoad(addr) => {
                let addr_ty = self.infer_expr(addr, env, current_effects)?;
                Ok(Type::I32)
            }

            Expr::AtomicStore(addr, value) => {
                let addr_ty = self.infer_expr(addr, env, current_effects)?;
                let value_ty = self.infer_expr(value, env, current_effects)?;
                Ok(Type::Unit)
            }
        }
    }

    fn infer_literal(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Int(_) => Type::I32,
            Literal::Bool(_) => Type::Bool,
            Literal::Unit => Type::Unit,
            Literal::String(_) => Type::String,
        }
    }

    fn infer_binop(&self, op: &BinOp, left_ty: &Type, right_ty: &Type) -> TypeResult<Type> {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                if unify(left_ty, &Type::I32) && unify(right_ty, &Type::I32) {
                    Ok(Type::I32)
                } else {
                    Err(TypeError::new(format!(
                        "Arithmetic operator requires i32, got {:?} and {:?}",
                        left_ty, right_ty
                    )))
                }
            }
            BinOp::Eq | BinOp::Ne => {
                if unify(left_ty, right_ty) {
                    Ok(Type::Bool)
                } else {
                    Err(TypeError::new(format!(
                        "Equality operator requires same types, got {:?} and {:?}",
                        left_ty, right_ty
                    )))
                }
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if unify(left_ty, &Type::I32) && unify(right_ty, &Type::I32) {
                    Ok(Type::Bool)
                } else {
                    Err(TypeError::new(format!(
                        "Comparison operator requires i32, got {:?} and {:?}",
                        left_ty, right_ty
                    )))
                }
            }
            BinOp::And | BinOp::Or => {
                if unify(left_ty, &Type::Bool) && unify(right_ty, &Type::Bool) {
                    Ok(Type::Bool)
                } else {
                    Err(TypeError::new(format!(
                        "Logical operator requires Bool, got {:?} and {:?}",
                        left_ty, right_ty
                    )))
                }
            }
        }
    }

    fn infer_unop(&self, op: &UnOp, operand_ty: &Type) -> TypeResult<Type> {
        match op {
            UnOp::Neg => {
                if unify(operand_ty, &Type::I32) {
                    Ok(Type::I32)
                } else {
                    Err(TypeError::new(format!(
                        "Negation requires i32, got {:?}",
                        operand_ty
                    )))
                }
            }
            UnOp::Not => {
                if unify(operand_ty, &Type::Bool) {
                    Ok(Type::Bool)
                } else {
                    Err(TypeError::new(format!(
                        "Logical not requires Bool, got {:?}",
                        operand_ty
                    )))
                }
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_simple_function() {
        let input = "inc: i32 -> i32\ninc x = x + 1";
        let mut parser = Parser::new(input);
        let module = parser.parse_module().unwrap();

        let mut checker = TypeChecker::new();
        let result = checker.check_module(&module);
        assert!(result.is_ok());
    }

    #[test]
    fn test_effect_violation() {
        let input = "foo: i32 -> i32 !{pure, cpu, alloc heap}\nfoo x = x + 1\n\nbar: i32 -> i32 !{pure, cpu, alloc none}\nbar x = foo x";
        let mut parser = Parser::new(input);
        let module = parser.parse_module().unwrap();

        let mut checker = TypeChecker::new();
        let result = checker.check_module(&module);
        assert!(result.is_err());
    }
}
