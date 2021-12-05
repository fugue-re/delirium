use fugue::ir::il::ecode::{BranchTarget, ECode, Expr, Location, Stmt};

#[derive(Clone)]
pub enum ECodeTarget {
    Unresolved,
    IntraIns(Location),
    IntraBlk(Location),
    InterBlk(BranchTarget),
    InterSub(BranchTarget),
}

pub trait ECodeExt {
    fn branch_targets(&self) -> Vec<(usize, ECodeTarget)>;
}

impl ECodeExt for ECode {
    fn branch_targets(&self) -> Vec<(usize, ECodeTarget)> {
        let address = self.address();
        let naddress = self.address() + self.length();
        let op_count = self.operations().len();

        let mut targets = Vec::new();
        
        let is_local = |loc: &Location| -> bool { *loc.address() == address };
        let is_fall = |loc: &Location| -> bool { *loc.address() == naddress };

        let nlocation = |i: usize| -> Location { if i >= op_count {
            Location::new(naddress.clone(), i - op_count)
         } else {
            Location::new(naddress.clone(), i)
         }};
        
         let nbranch = |i: usize, tgt: &BranchTarget, targets: &mut Vec<(usize, ECodeTarget)>| {
            match tgt {
                BranchTarget::Computed(exp) => if let Expr::Val(ref bv) = exp {
                    if let Some(off) = bv.to_u64() {
                        if off == address.offset() {
                            targets.push((i, ECodeTarget::IntraIns(Location::new(address.clone(), 0))));
                        } else if off == naddress.offset() {
                            targets.push((i, ECodeTarget::IntraBlk(Location::new(naddress.clone(), 0))));
                        } else {
                            targets.push((i, ECodeTarget::InterBlk(tgt.clone())));
                        }
                    } else {
                        targets.push((i, ECodeTarget::Unresolved));
                    }
                } else {
                    targets.push((i, ECodeTarget::Unresolved));
                },
                BranchTarget::Location(loc) => if is_local(loc) {
                    targets.push((i, ECodeTarget::IntraIns(loc.clone())));
                } else if is_fall(loc) {
                    targets.push((i, ECodeTarget::IntraBlk(loc.clone())));
                } else {
                    targets.push((i, ECodeTarget::InterBlk(tgt.clone())));
                }
            }
         };
         
         let nfall = |i: usize, fall: Location, targets: &mut Vec<(usize, ECodeTarget)>| {
            targets.push((i, if is_local(&fall) {
                ECodeTarget::IntraIns(fall)
            } else {
                ECodeTarget::IntraBlk(fall)
            }));
         };

        for (i, stmt) in self.operations().iter().enumerate() {
            let next = nlocation(i);
            match stmt {
                Stmt::Branch(tgt) => {
                    nbranch(i, tgt, &mut targets);
                },
                Stmt::CBranch(_, tgt) => {
                    nbranch(i, tgt, &mut targets);
                    nfall(i, next, &mut targets);
                },
                Stmt::Call(tgt, _) => {
                    targets.push((i, ECodeTarget::InterSub(tgt.clone())));
                },
                Stmt::Return(tgt) => {
                    targets.push((i, ECodeTarget::InterSub(tgt.clone())));
                },
                Stmt::Intrinsic(_, _) => {
                    targets.push((i, ECodeTarget::Unresolved));
                },
                _ => (),
            }
        }

        targets
    }
}

pub trait StmtExt {
    fn is_branch(&self) -> bool;
    fn is_jump(&self) -> bool;
    fn is_cond(&self) -> bool;
    fn is_call(&self) -> bool;
    fn is_intrinsic(&self) -> bool;
    fn is_return(&self) -> bool;
}

impl StmtExt for Stmt {
    fn is_branch(&self) -> bool {
        matches!(self,
                 Stmt::Branch(_) |
                 Stmt::CBranch(_, _) |
                 Stmt::Call(_, _) |
                 Stmt::Intrinsic(_, _) |
                 Stmt::Return(_))
    }

    fn is_jump(&self) -> bool {
        matches!(self, Stmt::Branch(_) | Stmt::CBranch(_, _))
    }

    fn is_cond(&self) -> bool {
        matches!(self, Stmt::CBranch(_, _))
    }

    fn is_call(&self) -> bool {
        matches!(self, Stmt::Call(_, _))
    }

    fn is_intrinsic(&self) -> bool {
        matches!(self, Stmt::Intrinsic(_, _))
    }

    fn is_return(&self) -> bool {
        matches!(self, Stmt::Return(_))
    }
}