//! Drzewo składni: to, co skryba zrozumiał z prozy.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Divisible,
    /// `Contains(całość, część)`
    Contains,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnOp {
    Neg,
    Even,
    Odd,
    Empty,
    Positive,
    Negative,
    Reverse,
    Len,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Interp(Vec<Expr>),
    Bool(bool),
    Nic,
    Var(String),
    Bin(BinOp, Box<Expr>, Box<Expr>, usize),
    Un(UnOp, Box<Expr>, usize),
    Not(Box<Expr>, usize),
    And(Box<Expr>, Box<Expr>, usize),
    Or(Box<Expr>, Box<Expr>, usize),
    /// Uczynek Waszmości: indeks w `Program::funcs`.
    Call(usize, Vec<Expr>),
    /// Uczynek wbudowany: nazwa funkcji w biegu (runtime) C.
    Builtin(&'static str, Vec<Expr>, usize),
    Index(Box<Expr>, Box<Expr>, usize),
    Last(Box<Expr>, usize),
    Field(Box<Expr>, usize, usize),
    List(Vec<Expr>),
    /// Regestr o długości N pełen X (X liczony osobno dla każdego miejsca).
    Fill(Box<Expr>, Box<Expr>, usize),
    New(usize, Vec<(usize, Expr)>, usize),
}

#[derive(Clone, Debug)]
pub enum Place {
    Var(String),
    Index(Expr, Expr, usize),
    Last(Expr, usize),
    Field(Expr, usize, usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompoundOp {
    Add,
    Sub,
    Mul,
    Div,
    /// „Dodaj X do Y”: do regestru wkłada, do liczby dolicza.
    Put,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Assign(Place, Expr),
    Compound(Place, CompoundOp, Expr, usize),
    Print(Vec<Expr>, bool),
    ReadLine(Place, Option<Expr>),
    ReadToken(Place),
    If(Vec<(Expr, Vec<Stmt>)>, Option<Vec<Stmt>>, usize),
    While(Expr, Vec<Stmt>, usize),
    Forever(Vec<Stmt>),
    ForRange {
        var: String,
        from: Expr,
        to: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
        line: usize,
    },
    ForEach {
        var: String,
        coll: Expr,
        body: Vec<Stmt>,
        line: usize,
    },
    Repeat(Expr, Vec<Stmt>, usize),
    Break,
    Continue,
    Return(Option<Expr>),
    Expr(Expr),
    Sort {
        coll: Expr,
        field: Option<usize>,
        desc: bool,
        line: usize,
    },
    Remove {
        coll: Expr,
        index: Option<Expr>,
        line: usize,
    },
    WriteFile {
        content: Expr,
        path: Expr,
        append: bool,
        line: usize,
    },
}

#[derive(Clone, Debug)]
pub struct FuncDef {
    pub index: usize,
    pub params: Vec<String>,
    /// (miano w C, miano z pisma)
    pub locals: Vec<(String, String)>,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub struct FuncSig {
    pub display: String,
    pub cname: String,
    pub arity: usize,
}

#[derive(Clone, Debug)]
pub struct RodDef {
    pub display: String,
    pub fields: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub sigs: Vec<FuncSig>,
    pub funcs: Vec<FuncDef>,
    pub globals: Vec<(String, String)>,
    pub main: Vec<Stmt>,
    pub rods: Vec<RodDef>,
    pub fields: Vec<String>,
}
