use crate::ast::*;

pub struct EffectChecker;

impl EffectChecker {
    pub fn new() -> Self {
        EffectChecker
    }

    /// Check if caller_effects can call a function with callee_effects
    pub fn can_call(&self, caller_effects: &EffectSet, callee_effects: &EffectSet) -> bool {
        caller_effects.includes(callee_effects)
    }

    /// Merge two effect sets (used when combining expressions)
    pub fn merge(&self, e1: &EffectSet, e2: &EffectSet) -> EffectSet {
        EffectSet {
            purity: self.merge_purity(&e1.purity, &e2.purity),
            execution: self.merge_execution(&e1.execution, &e2.execution),
            allocation: self.merge_allocation(&e1.allocation, &e2.allocation),
            concurrency: self.merge_concurrency(&e1.concurrency, &e2.concurrency),
            debug: e1.debug || e2.debug,
        }
    }

    fn merge_purity(&self, p1: &Purity, p2: &Purity) -> Purity {
        match (p1, p2) {
            (Purity::IO, _) | (_, Purity::IO) => Purity::IO,
            (Purity::State, _) | (_, Purity::State) => Purity::State,
            (Purity::Debug, _) | (_, Purity::Debug) => Purity::Debug,
            (Purity::Pure, Purity::Pure) => Purity::Pure,
        }
    }

    fn merge_execution(&self, e1: &Execution, e2: &Execution) -> Execution {
        // For now, if any expression runs on GPU, the whole thing is GPU
        // In a real implementation, this would be more sophisticated
        match (e1, e2) {
            (Execution::Gpu, _) | (_, Execution::Gpu) => Execution::Gpu,
            (Execution::Cpu, Execution::Cpu) => Execution::Cpu,
        }
    }

    fn merge_allocation(&self, a1: &Allocation, a2: &Allocation) -> Allocation {
        match (a1, a2) {
            (Allocation::Heap, _) | (_, Allocation::Heap) => Allocation::Heap,
            (Allocation::Arena, _) | (_, Allocation::Arena) => Allocation::Arena,
            (Allocation::None, Allocation::None) => Allocation::None,
        }
    }

    fn merge_concurrency(&self, c1: &Concurrency, c2: &Concurrency) -> Concurrency {
        match (c1, c2) {
            (Concurrency::Concurrent, _) | (_, Concurrency::Concurrent) => Concurrency::Concurrent,
            (Concurrency::Single, Concurrency::Single) => Concurrency::Single,
        }
    }

    /// Get the minimal effect set required for an expression
    pub fn infer_expr_effects(&self, expr: &Expr) -> EffectSet {
        match expr {
            Expr::Literal(_) | Expr::Var(_) => EffectSet::default(),

            Expr::BinOp(_, left, right) => {
                let left_eff = self.infer_expr_effects(left);
                let right_eff = self.infer_expr_effects(right);
                self.merge(&left_eff, &right_eff)
            }

            Expr::UnOp(_, operand) => {
                self.infer_expr_effects(operand)
            }

            Expr::If(cond, then_branch, else_branch) => {
                let cond_eff = self.infer_expr_effects(cond);
                let then_eff = self.infer_expr_effects(then_branch);
                let else_eff = self.infer_expr_effects(else_branch);
                self.merge(&cond_eff, &self.merge(&then_eff, &else_eff))
            }

            Expr::Let(_, value, body) => {
                let value_eff = self.infer_expr_effects(value);
                let body_eff = self.infer_expr_effects(body);
                self.merge(&value_eff, &body_eff)
            }

            Expr::Call(name, args) => {
                // This is simplified - in reality we'd look up the function's effects
                let mut eff = EffectSet::default();
                for arg in args {
                    let arg_eff = self.infer_expr_effects(arg);
                    eff = self.merge(&eff, &arg_eff);
                }

                // Special cases for known primitives
                match name.as_str() {
                    "par_for" | "par_map" | "par_map_inplace" => {
                        eff.concurrency = Concurrency::Concurrent;
                    }
                    "async" | "await" => {
                        eff.purity = Purity::IO;
                        eff.concurrency = Concurrency::Concurrent;
                        eff.allocation = Allocation::Heap;
                    }
                    "log" => {
                        eff.purity = Purity::IO;
                        eff.debug = true;
                    }
                    "assert" => {
                        eff.debug = true;
                    }
                    _ => {}
                }

                eff
            }

            Expr::Match(scrutinee, arms) => {
                let mut eff = self.infer_expr_effects(scrutinee);
                for arm in arms {
                    let arm_eff = self.infer_expr_effects(&arm.body);
                    eff = self.merge(&eff, &arm_eff);
                }
                eff
            }

            Expr::Unsafe(inner) => {
                // Unsafe blocks can have arbitrary effects
                let inner_eff = self.infer_expr_effects(inner);
                EffectSet {
                    purity: Purity::IO,
                    execution: Execution::Cpu,
                    allocation: Allocation::Heap,
                    concurrency: Concurrency::Concurrent,
                    debug: inner_eff.debug,
                }
            }

            Expr::ParFor(_, _, _) | Expr::ParMap(_, _) | Expr::ParMapInplace(_, _, _) => {
                EffectSet {
                    purity: Purity::Pure,
                    execution: Execution::Cpu,
                    allocation: Allocation::None,
                    concurrency: Concurrency::Concurrent,
                    debug: false,
                }
            }

            Expr::Async(inner) => {
                let inner_eff = self.infer_expr_effects(inner);
                EffectSet {
                    purity: Purity::IO,
                    execution: Execution::Cpu,
                    allocation: Allocation::Heap,
                    concurrency: Concurrency::Concurrent,
                    debug: inner_eff.debug,
                }
            }

            Expr::Await(inner) => {
                let inner_eff = self.infer_expr_effects(inner);
                EffectSet {
                    purity: Purity::IO,
                    execution: Execution::Cpu,
                    allocation: Allocation::None,
                    concurrency: Concurrency::Single,
                    debug: inner_eff.debug,
                }
            }

            Expr::ActorCreate(_, _) | Expr::ActorSend(_, _, _) => EffectSet {
                purity: Purity::IO,
                execution: Execution::Cpu,
                allocation: Allocation::Heap,
                concurrency: Concurrency::Concurrent,
                debug: false,
            },

            Expr::NewArray(space, _, _) => EffectSet {
                purity: Purity::Pure,
                execution: match space {
                    SpaceKind::Cpu => Execution::Cpu,
                    SpaceKind::Gpu => Execution::Gpu,
                },
                allocation: Allocation::Arena,
                concurrency: Concurrency::Single,
                debug: false,
            },

            Expr::ArrayGet(_, _) | Expr::ArrayLen(_) => EffectSet::default(),

            Expr::ArraySet(_, _, _) => EffectSet {
                purity: Purity::State,
                execution: Execution::Cpu,
                allocation: Allocation::None,
                concurrency: Concurrency::Single,
                debug: false,
            },

            Expr::GpuKernel(inner) => {
                let inner_eff = self.infer_expr_effects(inner);
                EffectSet {
                    purity: inner_eff.purity,
                    execution: Execution::Gpu,
                    allocation: inner_eff.allocation,
                    concurrency: inner_eff.concurrency,
                    debug: inner_eff.debug,
                }
            }

            Expr::CpuToGpu(_) | Expr::GpuToCpu(_) => EffectSet {
                purity: Purity::IO,
                execution: Execution::Cpu,
                allocation: Allocation::Heap,
                concurrency: Concurrency::Single,
                debug: false,
            },

            Expr::Log(_) => EffectSet {
                purity: Purity::IO,
                execution: Execution::Cpu,
                allocation: Allocation::None,
                concurrency: Concurrency::Single,
                debug: true,
            },

            Expr::Assert(_) => EffectSet {
                purity: Purity::Pure,
                execution: Execution::Cpu,
                allocation: Allocation::None,
                concurrency: Concurrency::Single,
                debug: true,
            },

            Expr::Print(_) => EffectSet {
                purity: Purity::IO,
                execution: Execution::Cpu,
                allocation: Allocation::None,
                concurrency: Concurrency::Single,
                debug: false,
            },

            Expr::RawThreadSpawn(_) | Expr::AtomicLoad(_) | Expr::AtomicStore(_, _) => {
                EffectSet {
                    purity: Purity::IO,
                    execution: Execution::Cpu,
                    allocation: Allocation::Heap,
                    concurrency: Concurrency::Concurrent,
                    debug: false,
                }
            }
        }
    }
}

impl Default for EffectChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_inclusion() {
        let none_effects = EffectSet::default();
        let heap_effects = EffectSet {
            purity: Purity::Pure,
            execution: Execution::Cpu,
            allocation: Allocation::Heap,
            concurrency: Concurrency::Single,
            debug: false,
        };

        // alloc none cannot call alloc heap
        assert!(!none_effects.includes(&heap_effects));

        // alloc heap can call alloc none
        assert!(heap_effects.includes(&none_effects));
    }

    #[test]
    fn test_effect_merge() {
        let checker = EffectChecker::new();

        let e1 = EffectSet::default();
        let e2 = EffectSet {
            purity: Purity::IO,
            execution: Execution::Cpu,
            allocation: Allocation::Heap,
            concurrency: Concurrency::Single,
            debug: false,
        };

        let merged = checker.merge(&e1, &e2);
        assert_eq!(merged.purity, Purity::IO);
        assert_eq!(merged.allocation, Allocation::Heap);
    }
}
