use crate::ir::{Addr, Blk, Expr};
use crate::prelude::Id;

#[derive(Clone)]
pub enum Loc {
    Resolved(Id<Blk>),
    Fixed(Addr),
    Computed(Expr),
}

impl From<Id<Blk>> for Loc {
    fn from(id: Id<Blk>) -> Self {
        Loc::Resolved(id)
    }
}

impl From<Addr> for Loc {
    fn from(addr: Addr) -> Self {
        Loc::Fixed(addr)
    }
}

impl From<Expr> for Loc {
    fn from(expr: Expr) -> Self {
        Loc::Computed(expr)
    }
}

impl Loc {
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved(_))
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed(_))
    }
    
    pub fn is_determined(&self) -> bool {
        self.is_fixed() || self.is_resolved()
    }
    
    pub fn is_computed(&self) -> bool {
        matches!(self, Self::Computed(_))
    }
}