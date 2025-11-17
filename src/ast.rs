use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    FunctionDef(FunctionDef),
    DataDef(DataDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub type_sig: TypeSig,
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeSig {
    pub param_types: Vec<Type>,
    pub return_type: Box<Type>,
    pub effects: EffectSet,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub constructors: Vec<Constructor>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constructor {
    pub name: String,
    pub fields: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I32,
    Bool,
    Unit,
    String,
    Var(String),
    Function(Vec<Type>, Box<Type>, EffectSet),
    Arena,
    Space(SpaceKind),
    Array(Box<Type>, SpaceKind, String), // Array<T, Space, arena_name>
    Task(Box<Type>),
    Actor(String), // Actor type with state type name
    Message(String), // Message type for actor
    Custom(String, Vec<Type>), // For user-defined types
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpaceKind {
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectSet {
    pub purity: Purity,
    pub execution: Execution,
    pub allocation: Allocation,
    pub concurrency: Concurrency,
    pub debug: bool,
}

impl Default for EffectSet {
    fn default() -> Self {
        EffectSet {
            purity: Purity::Pure,
            execution: Execution::Cpu,
            allocation: Allocation::None,
            concurrency: Concurrency::Concurrent,  // Concurrent by default!
            debug: false,
        }
    }
}

impl EffectSet {
    pub fn includes(&self, other: &EffectSet) -> bool {
        // Check if self includes (is superset of) other
        let purity_ok = match (&self.purity, &other.purity) {
            (Purity::Pure, Purity::Pure) => true,
            (Purity::IO, _) => true,
            (Purity::State, Purity::Pure) | (Purity::State, Purity::State) => true,
            _ => false,
        };

        let exec_ok = match (&self.execution, &other.execution) {
            (a, b) if a == b => true,
            _ => false,
        };

        let alloc_ok = match (&self.allocation, &other.allocation) {
            (Allocation::None, Allocation::None) => true,
            (Allocation::Arena, Allocation::None) | (Allocation::Arena, Allocation::Arena) => true,
            (Allocation::Heap, _) => true,
            _ => false,
        };

        let concur_ok = match (&self.concurrency, &other.concurrency) {
            (Concurrency::Single, Concurrency::Single) => true,
            (Concurrency::Concurrent, _) => true,
            _ => false,
        };

        let debug_ok = self.debug || !other.debug;

        purity_ok && exec_ok && alloc_ok && concur_ok && debug_ok
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Purity {
    Pure,
    IO,
    State,
    Debug,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Execution {
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Allocation {
    None,
    Arena,
    Heap,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Concurrency {
    Single,
    Concurrent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Var(String),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
    Match(Box<Expr>, Vec<MatchArm>),
    Unsafe(Box<Expr>),
    // Parallel primitives
    ParFor(Box<Expr>, Box<Expr>, Box<Expr>), // (start, end, body)
    ParMap(Box<Expr>, Box<Expr>), // (fn, array)
    ParMapInplace(Box<Expr>, Box<Expr>, Box<Expr>), // (fn, src_array, dst_array)
    // Async/Task
    Async(Box<Expr>),
    Await(Box<Expr>),
    // Actor
    ActorCreate(String, Box<Expr>), // (actor_type, initial_state)
    ActorSend(Box<Expr>, String, Vec<Expr>), // (actor, message, args)
    // Array operations
    NewArray(SpaceKind, String, Box<Expr>), // (space, arena, size)
    ArrayGet(Box<Expr>, Box<Expr>), // (array, index)
    ArraySet(Box<Expr>, Box<Expr>, Box<Expr>), // (array, index, value)
    ArrayLen(Box<Expr>),
    // GPU operations
    GpuKernel(Box<Expr>), // Marks a GPU kernel function call
    CpuToGpu(Box<Expr>), // Transfer data CPU -> GPU
    GpuToCpu(Box<Expr>), // Transfer data GPU -> CPU
    // Debug
    Log(Box<Expr>),
    Assert(Box<Expr>),
    Print(Box<Expr>),
    // Low-level unsafe operations
    RawThreadSpawn(Box<Expr>),
    AtomicLoad(Box<Expr>),
    AtomicStore(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i32),
    Bool(bool),
    Unit,
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Literal(Literal),
    Var(String),
    Constructor(String, Vec<Pattern>),
}
