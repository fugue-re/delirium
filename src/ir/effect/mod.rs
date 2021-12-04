use crate::ir::{Expr, Loc, Var};
use crate::prelude::Entity;

use std::sync::Arc;
use smallvec::SmallVec;

// effects that affect data flow
#[derive(Clone)]
pub enum Def {
    Assign(Var, Expr),
    Assume(Expr),
}

impl Def {
    pub fn assign(var: impl Into<Var>, expr: impl Into<Expr>) -> Entity<Self> {
        Entity::new("def", Self::Assign(var.into(), expr.into()))
    }
    
    pub fn assume(cnd: impl Into<Expr>) -> Entity<Self> {
        Entity::new("def", Self::Assume(cnd.into()))
    }
}

// effects that affect control flow
#[derive(Clone)]
pub enum Jmp {
    Branch(Loc),
    CBranch(Loc, Expr),
    Call(Loc, SmallVec<[Expr; 4]>),
    Intrinsic(Arc<str>, SmallVec<[Expr; 4]>),
    Return(Loc),
}

impl Jmp {
    pub fn branch(loc: impl Into<Loc>) -> Entity<Self> {
        Entity::new("jmp", Self::Branch(loc.into()))
    }

    pub fn cbranch(loc: impl Into<Loc>, cnd: impl Into<Expr>) -> Entity<Self> {
        Entity::new("jmp", Self::CBranch(loc.into(), cnd.into()))
    }
}