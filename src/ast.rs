//! Abstract syntax tree for TinyS.

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Impl(Impl),
    Trait(Trait),
    Use(Use),
}

/// The raw text inside an attribute `#[...]`.
pub type Attr = String;

#[derive(Debug, Clone)]
pub enum GenericParam {
    Type { name: String, bounds: Vec<String> },
    Lifetime(String),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub attrs: Vec<Attr>,
    pub is_pub: bool,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub ret: Type,
    /// `None` for trait method signatures without a default body.
    pub body: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub attrs: Vec<Attr>,
    pub is_pub: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub is_pub: bool,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub attrs: Vec<Attr>,
    pub is_pub: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub payload: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct Impl {
    pub generics: Vec<GenericParam>,
    /// `Some(trait)` for `impl Trait for Type`.
    pub trait_name: Option<Type>,
    pub ty: Type,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Trait {
    pub attrs: Vec<Attr>,
    pub is_pub: bool,
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Use {
    /// Path segments after resolving the `rust`/native root, e.g. `["serde"]`.
    pub path: Vec<String>,
    /// Imported names for `from X import a, b`; empty for `import X`.
    pub names: Vec<UseName>,
    /// Alias for `import X as Y`.
    pub alias: Option<String>,
    /// True when this came from the `macro`/`macro.*` root (no `use` emitted).
    pub is_macro: bool,
    /// True when routed through the explicit `rust` root (external crate/std).
    pub is_rust: bool,
}

#[derive(Debug, Clone)]
pub struct UseName {
    pub name: String,
    pub alias: Option<String>,
}

/// A type expression.
#[derive(Debug, Clone)]
pub enum Type {
    /// The unit type (`void`).
    Unit,
    /// A (possibly qualified) named type with generic arguments, e.g.
    /// `Option[T]` → path `["Option"]`, args `[T]`.
    Path {
        segments: Vec<String>,
        args: Vec<Type>,
    },
    /// `ref T` / `mut ref T`, with an optional lifetime.
    Ref {
        mutable: bool,
        lifetime: Option<String>,
        inner: Box<Type>,
    },
    Tuple(Vec<Type>),
    /// `array[T, N]` → `[T; N]`.
    Array { elem: Box<Type>, len: Box<Expr> },
    /// `slice[T]` → `[T]`.
    Slice(Box<Type>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `x = e`, `mut x = e`, `x: T = e`.
    Let {
        mutable: bool,
        name: String,
        ty: Option<Type>,
        value: Expr,
    },
    /// Destructuring bind `a, b = e`.
    LetTuple {
        names: Vec<String>,
        value: Expr,
    },
    /// `target op= value` or plain `target = value` where target is not a fresh name.
    Assign {
        target: Expr,
        op: AssignOp,
        value: Expr,
    },
    Expr(Expr),
    Return(Option<Expr>),
    If(IfChain),
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    /// `while case PAT = EXPR:`
    WhileCase {
        pattern: Pattern,
        expr: Expr,
        body: Vec<Stmt>,
    },
    For {
        pattern: Pattern,
        iter: Expr,
        /// `for x in ref xs` / `for x in mut ref xs`.
        iter_borrow: Borrow,
        body: Vec<Stmt>,
    },
    Loop {
        label: Option<String>,
        body: Vec<Stmt>,
    },
    Break {
        label: Option<String>,
        value: Option<Expr>,
    },
    Continue,
    Match(MatchExpr),
    /// `unsafe:` block.
    Unsafe(Vec<Stmt>),
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Borrow {
    Owned,
    Ref,
    MutRef,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignOp {
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Debug, Clone)]
pub struct IfChain {
    /// `(condition, body)` pairs: the `if` and each `elif`.
    pub arms: Vec<(CondKind, Vec<Stmt>)>,
    pub else_body: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone)]
pub enum CondKind {
    Bool(Expr),
    /// `if case PAT = EXPR`.
    Case { pattern: Pattern, expr: Expr },
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub arms: Vec<(CondKind, Vec<Stmt>)>,
    pub else_body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    /// A binding like `value`, or `none` → `None`.
    Ident(String),
    Literal(Expr),
    Tuple(Vec<Pattern>),
    /// `Token.Number(value)` / `Some(x)`.
    TupleStruct {
        path: Vec<String>,
        elems: Vec<Pattern>,
    },
    /// A path with no payload: `Token.Plus`.
    Path(Vec<String>),
    Or(Vec<Pattern>),
    /// `PAT as name`.
    Binding {
        pattern: Box<Pattern>,
        name: String,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    /// Inner text of a normal string literal.
    Str(String),
    RawStr(String),
    Bool(bool),
    Ident(String),
    /// The `none` literal → `None`.
    NoneLit,
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// Field access or path segment: `recv.name`. Codegen decides `.` vs `::`.
    Field {
        recv: Box<Expr>,
        name: String,
    },
    Index {
        recv: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        /// Turbofish generic args from `f[T](...)`.
        type_args: Vec<Type>,
        args: Vec<Arg>,
    },
    /// `ref x` / `mut ref x`.
    Ref {
        mutable: bool,
        expr: Box<Expr>,
    },
    /// `at x` → `*x`.
    Deref(Box<Expr>),
    /// `clone x` → `x.clone()`.
    CloneExpr(Box<Expr>),
    /// `move x` → `x` (ownership move is implicit in Rust).
    Move(Box<Expr>),
    /// `x.await`.
    Await(Box<Expr>),
    /// `x?`.
    Try(Box<Expr>),
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },
    Tuple(Vec<Expr>),
    /// `[a, b, c]` → `vec![...]`.
    List(Vec<Expr>),
    /// `{k: v, ...}` → a HashMap built from an array of pairs.
    Dict(Vec<(Expr, Expr)>),
    If(Box<IfExpr>),
    Match(Box<MatchExpr>),
    /// A `loop` used in value position; its value comes from `break <value>`.
    Loop {
        label: Option<String>,
        body: Vec<Stmt>,
    },
    Closure {
        is_move: bool,
        params: Vec<Param>,
        ret: Option<Type>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub enum Arg {
    Positional(Expr),
    /// `name=value` → struct-literal field.
    Keyword(String, Expr),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

impl BinaryOp {
    pub fn rust(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Rem => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Le => "<=",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }
}
