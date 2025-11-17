use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeEnv {
    pub vars: HashMap<String, Type>,
    pub functions: HashMap<String, TypeSig>,
    pub data_types: HashMap<String, DataDef>,
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = TypeEnv {
            vars: HashMap::new(),
            functions: HashMap::new(),
            data_types: HashMap::new(),
        };

        env.register_primitives();
        env
    }

    fn register_primitives(&mut self) {
        // par_for: (i32, i32) -> (i32 -> Unit) -> Unit !{pure, cpu, alloc none, concurrent}
        self.functions.insert(
            "par_for".to_string(),
            TypeSig {
                param_types: vec![
                    Type::I32,
                    Type::I32,
                    Type::Function(
                        vec![Type::I32],
                        Box::new(Type::Unit),
                        EffectSet::default(),
                    ),
                ],
                return_type: Box::new(Type::Unit),
                effects: EffectSet {
                    purity: Purity::Pure,
                    execution: Execution::Cpu,
                    allocation: Allocation::None,
                    concurrency: Concurrency::Concurrent,
                    debug: false,
                },
            },
        );

        // par_map_inplace: (a -> b) -> Array a (Cpu, arena) -> Array b (Cpu, arena) -> Unit
        self.functions.insert(
            "par_map_inplace".to_string(),
            TypeSig {
                param_types: vec![
                    Type::Function(
                        vec![Type::Var("a".to_string())],
                        Box::new(Type::Var("b".to_string())),
                        EffectSet::default(),
                    ),
                    Type::Array(
                        Box::new(Type::Var("a".to_string())),
                        SpaceKind::Cpu,
                        "arena".to_string(),
                    ),
                    Type::Array(
                        Box::new(Type::Var("b".to_string())),
                        SpaceKind::Cpu,
                        "arena".to_string(),
                    ),
                ],
                return_type: Box::new(Type::Unit),
                effects: EffectSet {
                    purity: Purity::Pure,
                    execution: Execution::Cpu,
                    allocation: Allocation::None,
                    concurrency: Concurrency::Concurrent,
                    debug: false,
                },
            },
        );

        // par_map: (a -> b) -> Array a (Cpu, arena) -> Array b (Cpu, arena)
        self.functions.insert(
            "par_map".to_string(),
            TypeSig {
                param_types: vec![
                    Type::Function(
                        vec![Type::Var("a".to_string())],
                        Box::new(Type::Var("b".to_string())),
                        EffectSet::default(),
                    ),
                    Type::Array(
                        Box::new(Type::Var("a".to_string())),
                        SpaceKind::Cpu,
                        "arena".to_string(),
                    ),
                ],
                return_type: Box::new(Type::Array(
                    Box::new(Type::Var("b".to_string())),
                    SpaceKind::Cpu,
                    "arena".to_string(),
                )),
                effects: EffectSet {
                    purity: Purity::Pure,
                    execution: Execution::Cpu,
                    allocation: Allocation::Heap,
                    concurrency: Concurrency::Concurrent,
                    debug: false,
                },
            },
        );

        // async: (Unit -> a !{io, cpu}) -> Task a !{io, cpu, concurrent}
        self.functions.insert(
            "async".to_string(),
            TypeSig {
                param_types: vec![Type::Function(
                    vec![Type::Unit],
                    Box::new(Type::Var("a".to_string())),
                    EffectSet {
                        purity: Purity::IO,
                        execution: Execution::Cpu,
                        allocation: Allocation::Heap,
                        concurrency: Concurrency::Single,
                        debug: false,
                    },
                )],
                return_type: Box::new(Type::Task(Box::new(Type::Var("a".to_string())))),
                effects: EffectSet {
                    purity: Purity::IO,
                    execution: Execution::Cpu,
                    allocation: Allocation::Heap,
                    concurrency: Concurrency::Concurrent,
                    debug: false,
                },
            },
        );

        // await: Task a -> a !{io, cpu}
        self.functions.insert(
            "await".to_string(),
            TypeSig {
                param_types: vec![Type::Task(Box::new(Type::Var("a".to_string())))],
                return_type: Box::new(Type::Var("a".to_string())),
                effects: EffectSet {
                    purity: Purity::IO,
                    execution: Execution::Cpu,
                    allocation: Allocation::None,
                    concurrency: Concurrency::Single,
                    debug: false,
                },
            },
        );

        // log: a -> Unit !{io, cpu, debug}
        self.functions.insert(
            "log".to_string(),
            TypeSig {
                param_types: vec![Type::Var("a".to_string())],
                return_type: Box::new(Type::Unit),
                effects: EffectSet {
                    purity: Purity::IO,
                    execution: Execution::Cpu,
                    allocation: Allocation::None,
                    concurrency: Concurrency::Single,
                    debug: true,
                },
            },
        );

        // assert: Bool -> Unit !{pure, cpu, debug}
        self.functions.insert(
            "assert".to_string(),
            TypeSig {
                param_types: vec![Type::Bool],
                return_type: Box::new(Type::Unit),
                effects: EffectSet {
                    purity: Purity::Pure,
                    execution: Execution::Cpu,
                    allocation: Allocation::None,
                    concurrency: Concurrency::Single,
                    debug: true,
                },
            },
        );
    }

    pub fn add_var(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }

    pub fn get_var(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }

    pub fn add_function(&mut self, name: String, sig: TypeSig) {
        self.functions.insert(name, sig);
    }

    pub fn get_function(&self, name: &str) -> Option<&TypeSig> {
        self.functions.get(name)
    }

    pub fn add_data_type(&mut self, name: String, def: DataDef) {
        self.data_types.insert(name, def);
    }

    pub fn get_data_type(&self, name: &str) -> Option<&DataDef> {
        self.data_types.get(name)
    }

    pub fn push_scope(&self) -> TypeEnv {
        TypeEnv {
            vars: self.vars.clone(),
            functions: self.functions.clone(),
            data_types: self.data_types.clone(),
        }
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

pub fn unify(t1: &Type, t2: &Type) -> bool {
    match (t1, t2) {
        (Type::I32, Type::I32) => true,
        (Type::Bool, Type::Bool) => true,
        (Type::Unit, Type::Unit) => true,
        (Type::String, Type::String) => true,
        (Type::Arena, Type::Arena) => true,
        (Type::Var(v1), Type::Var(v2)) => v1 == v2,
        (Type::Var(_), _) => true,
        (_, Type::Var(_)) => true,
        (Type::Function(p1, r1, e1), Type::Function(p2, r2, e2)) => {
            p1.len() == p2.len()
                && p1.iter().zip(p2.iter()).all(|(a, b)| unify(a, b))
                && unify(r1, r2)
                && e1 == e2
        }
        (Type::Array(t1, s1, a1), Type::Array(t2, s2, a2)) => {
            unify(t1, t2) && s1 == s2 && a1 == a2
        }
        (Type::Task(t1), Type::Task(t2)) => unify(t1, t2),
        (Type::Custom(n1, args1), Type::Custom(n2, args2)) => {
            n1 == n2
                && args1.len() == args2.len()
                && args1.iter().zip(args2.iter()).all(|(a, b)| unify(a, b))
        }
        _ => false,
    }
}
