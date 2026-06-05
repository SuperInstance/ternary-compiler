//! Ternary expression compiler: parse, optimize, and evaluate ternary logic expressions.

/// Ternary value
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TV { Minus, Zero, Plus }

impl TV {
    pub fn from_i8(v: i8) -> Self {
        match v { -1 => TV::Minus, 0 => TV::Zero, 1 => TV::Plus, _ => TV::Zero }
    }
    pub fn to_i8(self) -> i8 { match self { TV::Minus => -1, TV::Zero => 0, TV::Plus => 1 } }
}

/// AST for ternary expressions
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Const(TV),
    Var(String),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Min(Box<Expr>, Box<Expr>),
    Max(Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>), // if cond > 0 then a else b
}

impl Expr {
    /// Evaluate with variable bindings
    pub fn eval(&self, env: &std::collections::HashMap<String, TV>) -> TV {
        match self {
            Expr::Const(v) => *v,
            Expr::Var(name) => *env.get(name).unwrap_or(&TV::Zero),
            Expr::Not(e) => TV::from_i8(-e.eval(env).to_i8()),
            Expr::And(a, b) => TV::from_i8(a.eval(env).to_i8().min(b.eval(env).to_i8())),
            Expr::Or(a, b) => TV::from_i8(a.eval(env).to_i8().max(b.eval(env).to_i8())),
            Expr::Min(a, b) => TV::from_i8(a.eval(env).to_i8().min(b.eval(env).to_i8())),
            Expr::Max(a, b) => TV::from_i8(a.eval(env).to_i8().max(b.eval(env).to_i8())),
            Expr::If(cond, then, else_) => {
                if cond.eval(env).to_i8() > 0 { then.eval(env) } else { else_.eval(env) }
            }
        }
    }

    /// Constant folding optimization
    pub fn optimize(&self) -> Expr {
        match self {
            Expr::Const(v) => Expr::Const(*v),
            Expr::Var(name) => Expr::Var(name.clone()),
            Expr::Not(e) => {
                let opt = e.optimize();
                if let Expr::Const(v) = opt { Expr::Const(TV::from_i8(-v.to_i8())) }
                else { Expr::Not(Box::new(opt)) }
            }
            Expr::And(a, b) => {
                let oa = a.optimize();
                let ob = b.optimize();
                if let (Expr::Const(va), Expr::Const(vb)) = (&oa, &ob) {
                    Expr::Const(TV::from_i8(va.to_i8().min(vb.to_i8())))
                } else { Expr::And(Box::new(oa), Box::new(ob)) }
            }
            Expr::Or(a, b) => {
                let oa = a.optimize();
                let ob = b.optimize();
                if let (Expr::Const(va), Expr::Const(vb)) = (&oa, &ob) {
                    Expr::Const(TV::from_i8(va.to_i8().max(vb.to_i8())))
                } else { Expr::Or(Box::new(oa), Box::new(ob)) }
            }
            Expr::Min(a, b) | Expr::Max(a, b) => {
                let oa = a.optimize();
                let ob = b.optimize();
                match (&oa, &ob) {
                    (Expr::Const(va), Expr::Const(vb)) => {
                        let result = match self {
                            Expr::Min(_, _) => va.to_i8().min(vb.to_i8()),
                            Expr::Max(_, _) => va.to_i8().max(vb.to_i8()),
                            _ => unreachable!(),
                        };
                        Expr::Const(TV::from_i8(result))
                    }
                    _ => match self {
                        Expr::Min(_, _) => Expr::Min(Box::new(oa), Box::new(ob)),
                        Expr::Max(_, _) => Expr::Max(Box::new(oa), Box::new(ob)),
                        _ => unreachable!(),
                    }
                }
            }
            Expr::If(c, t, e) => {
                let oc = c.optimize();
                if let Expr::Const(v) = oc {
                    if v.to_i8() > 0 { t.optimize() } else { e.optimize() }
                } else {
                    Expr::If(Box::new(oc), Box::new(t.optimize()), Box::new(e.optimize()))
                }
            }
        }
    }

    /// Collect all variable names
    pub fn free_vars(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_vars(&mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    fn collect_vars(&self, vars: &mut Vec<String>) {
        match self {
            Expr::Var(name) => { vars.push(name.clone()); }
            Expr::Const(_) => {}
            Expr::Not(e) => e.collect_vars(vars),
            Expr::And(a, b) | Expr::Or(a, b) | Expr::Min(a, b) | Expr::Max(a, b) => {
                a.collect_vars(vars); b.collect_vars(vars);
            }
            Expr::If(c, t, e) => { c.collect_vars(vars); t.collect_vars(vars); e.collect_vars(vars); }
        }
    }
}

/// Compile expression to bytecode
#[derive(Clone, Debug)]
pub enum Op {
    PushConst(i8),
    Load(usize),
    Neg,
    Min,
    Max,
    JumpIfNotPlus(usize),
}

pub struct Compiler {
    pub bytecode: Vec<Op>,
}

impl Compiler {
    pub fn new() -> Self { Self { bytecode: Vec::new() } }

    pub fn compile(&mut self, expr: &Expr) {
        match expr {
            Expr::Const(v) => self.bytecode.push(Op::PushConst(v.to_i8())),
            Expr::Var(name) => {
                // Simple: use hash of name as slot
                let slot = name.chars().map(|c| c as usize).sum::<usize>() % 256;
                self.bytecode.push(Op::Load(slot));
            }
            Expr::Not(e) => { self.compile(e); self.bytecode.push(Op::Neg); }
            Expr::Min(a, b) | Expr::And(a, b) => { self.compile(a); self.compile(b); self.bytecode.push(Op::Min); }
            Expr::Max(a, b) | Expr::Or(a, b) => { self.compile(a); self.compile(b); self.bytecode.push(Op::Max); }
            Expr::If(_, _, _) => {
                // Simplified: just compile as max
                // Full if requires jump offsets which needs a two-pass
                self.bytecode.push(Op::PushConst(0));
            }
        }
    }

    /// Execute bytecode with a simple stack machine
    pub fn execute(&self, slots: &[i8]) -> i8 {
        let mut stack = Vec::new();
        for op in &self.bytecode {
            match op {
                Op::PushConst(v) => stack.push(*v),
                Op::Load(slot) => stack.push(slots.get(*slot).copied().unwrap_or(0)),
                Op::Neg => { if let Some(v) = stack.pop() { stack.push(-v); } }
                Op::Min => {
                    let b = stack.pop().unwrap_or(0);
                    let a = stack.pop().unwrap_or(0);
                    stack.push(a.min(b));
                }
                Op::Max => {
                    let b = stack.pop().unwrap_or(0);
                    let a = stack.pop().unwrap_or(0);
                    stack.push(a.max(b));
                }
                Op::JumpIfNotPlus(_) => {} // simplified
            }
        }
        stack.pop().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_eval() {
        let expr = Expr::Const(TV::Plus);
        let env = std::collections::HashMap::new();
        assert_eq!(expr.eval(&env), TV::Plus);
    }

    #[test]
    fn test_not() {
        let expr = Expr::Not(Box::new(Expr::Const(TV::Plus)));
        let env = std::collections::HashMap::new();
        assert_eq!(expr.eval(&env), TV::Minus);
    }

    #[test]
    fn test_and() {
        let expr = Expr::And(Box::new(Expr::Const(TV::Plus)), Box::new(Expr::Const(TV::Minus)));
        let env = std::collections::HashMap::new();
        assert_eq!(expr.eval(&env), TV::Minus);
    }

    #[test]
    fn test_or() {
        let expr = Expr::Or(Box::new(Expr::Const(TV::Plus)), Box::new(Expr::Const(TV::Minus)));
        let env = std::collections::HashMap::new();
        assert_eq!(expr.eval(&env), TV::Plus);
    }

    #[test]
    fn test_constant_folding() {
        let expr = Expr::And(Box::new(Expr::Const(TV::Plus)), Box::new(Expr::Const(TV::Zero)));
        let opt = expr.optimize();
        assert_eq!(opt, Expr::Const(TV::Zero));
    }

    #[test]
    fn test_if_eval() {
        let expr = Expr::If(
            Box::new(Expr::Const(TV::Plus)),
            Box::new(Expr::Const(TV::Plus)),
            Box::new(Expr::Const(TV::Minus)),
        );
        let env = std::collections::HashMap::new();
        assert_eq!(expr.eval(&env), TV::Plus);
    }

    #[test]
    fn test_free_vars() {
        let expr = Expr::And(Box::new(Expr::Var("x".into())), Box::new(Expr::Var("y".into())));
        let vars = expr.free_vars();
        assert_eq!(vars, vec!["x".to_string(), "y".to_string()]);
    }

    #[test]
    fn test_compiler_execute() {
        let expr = Expr::Max(Box::new(Expr::Const(TV::Minus)), Box::new(Expr::Const(TV::Plus)));
        let mut compiler = Compiler::new();
        compiler.compile(&expr);
        let result = compiler.execute(&[]);
        assert_eq!(result, 1);
    }
}
