use crate::ir::{Expr, Var};

#[derive(Clone)]
pub struct Phi {
    var: Var,
    choices: Vec<(Expr, Expr)>,
}