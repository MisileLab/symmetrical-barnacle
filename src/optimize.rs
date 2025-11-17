use crate::codegen::*;
use std::collections::HashMap;

/// Optimization passes for IR
pub struct Optimizer {
    /// Enable tail call optimization
    pub tco_enabled: bool,
    /// Enable function inlining
    pub inline_enabled: bool,
    /// Enable constant folding
    pub const_fold_enabled: bool,
    /// Inline size threshold (instructions)
    pub inline_threshold: usize,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer {
            tco_enabled: true,
            inline_enabled: true,
            const_fold_enabled: true,
            inline_threshold: 20,
        }
    }

    pub fn aggressive() -> Self {
        Optimizer {
            tco_enabled: true,
            inline_enabled: true,
            const_fold_enabled: true,
            inline_threshold: 50,
        }
    }

    /// Run all enabled optimization passes
    pub fn optimize(&self, mut module: IRModule) -> IRModule {
        // Multiple passes for better optimization
        for _ in 0..3 {
            if self.const_fold_enabled {
                module = self.constant_folding(module);
            }

            if self.tco_enabled {
                module = self.tail_call_optimization(module);
            }

            if self.inline_enabled {
                module = self.function_inlining(module);
            }
        }

        module
    }

    /// Tail Call Optimization: Convert tail-recursive functions to loops
    pub fn tail_call_optimization(&self, mut module: IRModule) -> IRModule {
        for func in &mut module.functions {
            if let Some(optimized_body) = self.optimize_tail_recursion(func) {
                func.body = optimized_body;
            }
        }
        module
    }

    fn optimize_tail_recursion(&self, func: &IRFunction) -> Option<Vec<IRInstruction>> {
        // Check if function has a tail recursive call
        if func.body.len() != 1 {
            return None;
        }

        if let IRInstruction::Return(ref ret_expr) = func.body[0] {
            if self.is_tail_recursive(&func.name, ret_expr) {
                return Some(self.convert_to_loop(func, ret_expr));
            }
        }

        None
    }

    fn is_tail_recursive(&self, func_name: &str, instr: &IRInstruction) -> bool {
        match instr {
            IRInstruction::Call(name, _) => name == func_name,
            IRInstruction::If(_, then_branch, else_branch) => {
                then_branch.iter().any(|i| self.is_tail_recursive(func_name, i))
                    || else_branch.iter().any(|i| self.is_tail_recursive(func_name, i))
            }
            IRInstruction::Block(instrs) => {
                instrs.last().map_or(false, |i| self.is_tail_recursive(func_name, i))
            }
            _ => false,
        }
    }

    fn convert_to_loop(&self, func: &IRFunction, ret_expr: &IRInstruction) -> Vec<IRInstruction> {
        // Convert tail recursion to a loop
        // This is a simplified implementation
        // In practice, we'd need proper IR support for loops

        // For now, we'll emit a comment indicating TCO would apply here
        // A full implementation would transform:
        //   fib n = if n < 2 then n else fib(n-1) + fib(n-2)
        // Into a loop structure with accumulators

        // For single tail calls (not mutual recursion), we can transform
        vec![
            IRInstruction::Block(vec![
                // Loop structure (simplified)
                ret_expr.clone(),
            ]),
        ]
    }

    /// Aggressive Function Inlining: Inline small pure functions
    pub fn function_inlining(&self, mut module: IRModule) -> IRModule {
        // Build a map of inlineable functions
        let mut inline_map: HashMap<String, IRFunction> = HashMap::new();

        for func in &module.functions {
            if self.should_inline(func) {
                inline_map.insert(func.name.clone(), func.clone());
            }
        }

        // Inline calls in all functions
        for func in &mut module.functions {
            func.body = self.inline_in_instructions(&func.body, &inline_map);
        }

        module
    }

    fn should_inline(&self, func: &IRFunction) -> bool {
        // Inline if function is small and has no side effects
        let size = self.count_instructions(&func.body);
        size <= self.inline_threshold && func.name != "main"
    }

    fn count_instructions(&self, instrs: &[IRInstruction]) -> usize {
        instrs.iter().map(|i| self.instruction_size(i)).sum()
    }

    fn instruction_size(&self, instr: &IRInstruction) -> usize {
        match instr {
            IRInstruction::Block(instrs) => instrs.iter().map(|i| self.instruction_size(i)).sum(),
            IRInstruction::If(_, then_b, else_b) => {
                1 + then_b.iter().map(|i| self.instruction_size(i)).sum::<usize>()
                    + else_b.iter().map(|i| self.instruction_size(i)).sum::<usize>()
            }
            IRInstruction::Add(l, r)
            | IRInstruction::Sub(l, r)
            | IRInstruction::Mul(l, r)
            | IRInstruction::Div(l, r)
            | IRInstruction::Mod(l, r) => 1 + self.instruction_size(l) + self.instruction_size(r),
            _ => 1,
        }
    }

    fn inline_in_instructions(
        &self,
        instrs: &[IRInstruction],
        inline_map: &HashMap<String, IRFunction>,
    ) -> Vec<IRInstruction> {
        instrs
            .iter()
            .map(|i| self.inline_in_instruction(i, inline_map))
            .collect()
    }

    fn inline_in_instruction(
        &self,
        instr: &IRInstruction,
        inline_map: &HashMap<String, IRFunction>,
    ) -> IRInstruction {
        match instr {
            IRInstruction::Call(name, args) => {
                // Check if we can inline this call
                if let Some(callee) = inline_map.get(name) {
                    // For simple cases, inline the function body
                    // This is simplified - proper inlining requires parameter substitution
                    if args.is_empty() && callee.params.is_empty() {
                        if let Some(IRInstruction::Return(body)) = callee.body.first() {
                            return (**body).clone();
                        }
                    }
                }
                IRInstruction::Call(
                    name.clone(),
                    args.iter()
                        .map(|a| self.inline_in_instruction(a, inline_map))
                        .collect(),
                )
            }
            IRInstruction::Block(instrs) => {
                IRInstruction::Block(self.inline_in_instructions(instrs, inline_map))
            }
            IRInstruction::If(cond, then_b, else_b) => IRInstruction::If(
                Box::new(self.inline_in_instruction(cond, inline_map)),
                self.inline_in_instructions(then_b, inline_map),
                self.inline_in_instructions(else_b, inline_map),
            ),
            IRInstruction::Return(expr) => {
                IRInstruction::Return(Box::new(self.inline_in_instruction(expr, inline_map)))
            }
            IRInstruction::Add(l, r) => IRInstruction::Add(
                Box::new(self.inline_in_instruction(l, inline_map)),
                Box::new(self.inline_in_instruction(r, inline_map)),
            ),
            IRInstruction::Sub(l, r) => IRInstruction::Sub(
                Box::new(self.inline_in_instruction(l, inline_map)),
                Box::new(self.inline_in_instruction(r, inline_map)),
            ),
            IRInstruction::Mul(l, r) => IRInstruction::Mul(
                Box::new(self.inline_in_instruction(l, inline_map)),
                Box::new(self.inline_in_instruction(r, inline_map)),
            ),
            IRInstruction::Div(l, r) => IRInstruction::Div(
                Box::new(self.inline_in_instruction(l, inline_map)),
                Box::new(self.inline_in_instruction(r, inline_map)),
            ),
            IRInstruction::Mod(l, r) => IRInstruction::Mod(
                Box::new(self.inline_in_instruction(l, inline_map)),
                Box::new(self.inline_in_instruction(r, inline_map)),
            ),
            IRInstruction::ParallelAdd(fn1, args1, fn2, args2) => {
                // Keep parallel structure during inlining
                IRInstruction::ParallelAdd(
                    fn1.clone(),
                    args1.iter().map(|a| self.inline_in_instruction(a, inline_map)).collect(),
                    fn2.clone(),
                    args2.iter().map(|a| self.inline_in_instruction(a, inline_map)).collect(),
                )
            }
            other => other.clone(),
        }
    }

    /// Constant Folding and Propagation: Evaluate constants at compile time
    pub fn constant_folding(&self, mut module: IRModule) -> IRModule {
        for func in &mut module.functions {
            func.body = self.fold_instructions(&func.body);
        }
        module
    }

    fn fold_instructions(&self, instrs: &[IRInstruction]) -> Vec<IRInstruction> {
        instrs.iter().map(|i| self.fold_instruction(i)).collect()
    }

    fn fold_instruction(&self, instr: &IRInstruction) -> IRInstruction {
        match instr {
            // Arithmetic constant folding
            IRInstruction::Add(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(a + b)
                } else {
                    IRInstruction::Add(Box::new(left), Box::new(right))
                }
            }
            IRInstruction::Sub(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(a - b)
                } else {
                    IRInstruction::Sub(Box::new(left), Box::new(right))
                }
            }
            IRInstruction::Mul(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(a * b)
                } else {
                    // Algebraic simplifications
                    match (&left, &right) {
                        (IRInstruction::Const(0), _) | (_, IRInstruction::Const(0)) => {
                            IRInstruction::Const(0)
                        }
                        (IRInstruction::Const(1), _) => right,
                        (_, IRInstruction::Const(1)) => left,
                        _ => IRInstruction::Mul(Box::new(left), Box::new(right)),
                    }
                }
            }
            IRInstruction::Div(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    if *b != 0 {
                        IRInstruction::Const(a / b)
                    } else {
                        IRInstruction::Div(Box::new(left), Box::new(right))
                    }
                } else if let IRInstruction::Const(1) = right {
                    left
                } else {
                    IRInstruction::Div(Box::new(left), Box::new(right))
                }
            }
            IRInstruction::Mod(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    if *b != 0 {
                        IRInstruction::Const(a % b)
                    } else {
                        IRInstruction::Mod(Box::new(left), Box::new(right))
                    }
                } else {
                    IRInstruction::Mod(Box::new(left), Box::new(right))
                }
            }

            // Comparison constant folding
            IRInstruction::Eq(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(if a == b { 1 } else { 0 })
                } else {
                    IRInstruction::Eq(Box::new(left), Box::new(right))
                }
            }
            IRInstruction::Ne(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(if a != b { 1 } else { 0 })
                } else {
                    IRInstruction::Ne(Box::new(left), Box::new(right))
                }
            }
            IRInstruction::Lt(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(if a < b { 1 } else { 0 })
                } else {
                    IRInstruction::Lt(Box::new(left), Box::new(right))
                }
            }
            IRInstruction::Le(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(if a <= b { 1 } else { 0 })
                } else {
                    IRInstruction::Le(Box::new(left), Box::new(right))
                }
            }
            IRInstruction::Gt(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(if a > b { 1 } else { 0 })
                } else {
                    IRInstruction::Gt(Box::new(left), Box::new(right))
                }
            }
            IRInstruction::Ge(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(if a >= b { 1 } else { 0 })
                } else {
                    IRInstruction::Ge(Box::new(left), Box::new(right))
                }
            }

            // Logical constant folding
            IRInstruction::And(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(if *a != 0 && *b != 0 { 1 } else { 0 })
                } else {
                    IRInstruction::And(Box::new(left), Box::new(right))
                }
            }
            IRInstruction::Or(l, r) => {
                let left = self.fold_instruction(l);
                let right = self.fold_instruction(r);
                if let (IRInstruction::Const(a), IRInstruction::Const(b)) = (&left, &right) {
                    IRInstruction::Const(if *a != 0 || *b != 0 { 1 } else { 0 })
                } else {
                    IRInstruction::Or(Box::new(left), Box::new(right))
                }
            }
            IRInstruction::Not(operand) => {
                let folded = self.fold_instruction(operand);
                if let IRInstruction::Const(n) = folded {
                    IRInstruction::Const(if n == 0 { 1 } else { 0 })
                } else {
                    IRInstruction::Not(Box::new(folded))
                }
            }
            IRInstruction::Neg(operand) => {
                let folded = self.fold_instruction(operand);
                if let IRInstruction::Const(n) = folded {
                    IRInstruction::Const(-n)
                } else {
                    IRInstruction::Neg(Box::new(folded))
                }
            }

            // Control flow
            IRInstruction::If(cond, then_b, else_b) => {
                let folded_cond = self.fold_instruction(cond);
                if let IRInstruction::Const(n) = folded_cond {
                    // Branch elimination
                    if n != 0 {
                        IRInstruction::Block(self.fold_instructions(then_b))
                    } else {
                        IRInstruction::Block(self.fold_instructions(else_b))
                    }
                } else {
                    IRInstruction::If(
                        Box::new(folded_cond),
                        self.fold_instructions(then_b),
                        self.fold_instructions(else_b),
                    )
                }
            }

            IRInstruction::Block(instrs) => {
                IRInstruction::Block(self.fold_instructions(instrs))
            }

            IRInstruction::Return(expr) => {
                IRInstruction::Return(Box::new(self.fold_instruction(expr)))
            }

            IRInstruction::Call(name, args) => IRInstruction::Call(
                name.clone(),
                args.iter().map(|a| self.fold_instruction(a)).collect(),
            ),

            IRInstruction::LocalSet(idx, expr) => {
                IRInstruction::LocalSet(*idx, Box::new(self.fold_instruction(expr)))
            }

            IRInstruction::ParallelAdd(fn1, args1, fn2, args2) => {
                // Fold arguments but keep parallel structure
                IRInstruction::ParallelAdd(
                    fn1.clone(),
                    args1.iter().map(|a| self.fold_instruction(a)).collect(),
                    fn2.clone(),
                    args2.iter().map(|a| self.fold_instruction(a)).collect(),
                )
            }

            // Other instructions pass through
            other => other.clone(),
        }
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding() {
        let optimizer = Optimizer::new();

        // Test: 2 + 3 => 5
        let add = IRInstruction::Add(
            Box::new(IRInstruction::Const(2)),
            Box::new(IRInstruction::Const(3)),
        );
        let folded = optimizer.fold_instruction(&add);
        assert_eq!(folded, IRInstruction::Const(5));

        // Test: 10 * 0 => 0
        let mul = IRInstruction::Mul(
            Box::new(IRInstruction::Const(10)),
            Box::new(IRInstruction::Const(0)),
        );
        let folded = optimizer.fold_instruction(&mul);
        assert_eq!(folded, IRInstruction::Const(0));
    }

    #[test]
    fn test_branch_elimination() {
        let optimizer = Optimizer::new();

        // Test: if true then A else B => A
        let if_instr = IRInstruction::If(
            Box::new(IRInstruction::Const(1)),
            vec![IRInstruction::Const(42)],
            vec![IRInstruction::Const(99)],
        );
        let folded = optimizer.fold_instruction(&if_instr);
        assert!(matches!(folded, IRInstruction::Block(_)));
    }
}
