use crate::ir::{Blk, Expr};
use crate::prelude::Id;

#[derive(Clone)]
pub enum Loc {
    Fixed(Id<Blk>),
    Computed(Expr),
}

impl From<Id<Blk>> for Loc {
    fn from(id: Id<Blk>) -> Self {
        Loc::Fixed(id)
    }
}

impl From<Expr> for Loc {
    fn from(expr: Expr) -> Self {
        Loc::Computed(expr)
    }
}

impl Loc {
    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed(_))
    }
    
    pub fn is_computed(&self) -> bool {
        matches!(self, Self::Computed(_))
    }
}