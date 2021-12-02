use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::BTreeMap;

use fugue::ir::{AddressSpaceId, Translator};
use fugue::ir::il::ecode::ECode;
use fugue::ir::il::ecode::Expr as ECodeExpr;
use fugue::ir::il::ecode::Var as ECodeVar;

use crate::lift::passes::{Visit, VisitMut};

use crate::types::intervals::Interval;
use crate::types::intervals::collections::IntervalSet;

#[derive(Clone)]
pub(crate) struct ECodeVarIndex {
    space_id: AddressSpaceId,
    index: IntervalSet<u64>,
}

impl ECodeVarIndex {
    pub(crate) fn empty(space_id: AddressSpaceId) -> Self {
        Self {
            space_id,
            index: IntervalSet::default(),
        }
    }

    pub(crate) fn registers(translator: &Translator) -> Self {
        let space_id = translator.manager().register_space_id();

        Self {
            space_id,
            index: IntervalSet::from_iter(
                translator
                    .registers()
                    .iter()
                    .map(|((off, sz), _)| Interval::from(*off..=(off + *sz as u64)))
            )
        }
    }

    fn interval(var: &ECodeVar) -> Interval<u64> {
        Interval::from(var.offset()..(var.offset() + (var.bits() as u64) / 8))
    }
    
    fn insert(&mut self, var: &ECodeVar) {
        self.index.insert(Self::interval(var));
    }

    fn enclosing(&self, var: &ECodeVar) -> ECodeVar {
        let iv = Self::interval(var);
        let iv = self.index.find_all(&iv).into_iter().fold(&iv, |iv, ent| {
            let eiv = ent.interval();
            if (eiv.start() < iv.start() && eiv.end() >= iv.end())
                || (eiv.start() <= iv.start() && eiv.end() > iv.end())
                {
                    eiv
                } else {
                    iv
                }
        });
        ECodeVar::new(
            self.space_id,
            *iv.start(),
            8 * (1 + *iv.end() - *iv.start()) as usize,
            var.generation(),
        )
    }
}

pub(crate) struct ECodeVarAliasNormalisePass<'v> {
    registers: &'v ECodeVarIndex,
    indexes: BTreeMap<AddressSpaceId, ECodeVarIndex>,
}

impl<'v> ECodeVarAliasNormalisePass<'v> {
    pub(crate) fn new(registers: &'v ECodeVarIndex) -> Self {
        assert!(registers.space_id.is_register());
        Self {
            registers,
            indexes: BTreeMap::default(),
        }
    }

    fn insert(&mut self, var: &ECodeVar) {
        let space = var.space();
        if space.is_register() {
            return
        }

        self.indexes.entry(space)
            .or_insert_with(|| ECodeVarIndex::empty(space))
            .insert(var);
    }

    fn enclosing(&self, var: &ECodeVar) -> ECodeVar {
        let space = var.space();
        if let Some(index) = self.indexes.get(&space) {
            index.enclosing(var)
        } else {
            *var
        }
    }

    /// Assumes that svar.bits() != pvar.bits()
    /// Assumes that either svar completely contains pvar or pvar completely contains svar
    fn resize_expr(svar: &ECodeVar, pvar: &ECodeVar, expr: impl Borrow<ECodeExpr>) -> ECodeExpr {
        let expr = expr.borrow().clone();

        match svar.bits().cmp(&pvar.bits()) {
            Ordering::Greater => if svar.offset() == pvar.offset() { // truncate
                // e.g svar: RAX, pvar: AL
                ECodeExpr::extract_low(expr, pvar.bits())
            } else {
                // e.g. svar: RAX, pvar: AH
                let loff = (pvar.offset() - svar.offset()) as usize * 8;
                let moff = loff + pvar.bits();
                ECodeExpr::extract(expr, loff, moff)
            },
            Ordering::Less => if svar.offset() == pvar.offset() {
                // e.g. svar: AL, pvar: RAX
                let hbits = ECodeExpr::extract_high(*pvar, pvar.bits() - svar.bits());
                ECodeExpr::concat(hbits, expr)
            } else {
                if svar.offset() + (svar.bits() as u64 / 8) == (pvar.bits() as u64 / 8) {
                    // e.g. svar: AH, pvar: AX
                    let lbits = ECodeExpr::extract_low(*pvar, pvar.bits() - svar.bits());
                    ECodeExpr::concat(expr, lbits)
                } else {
                    // e.g. svar: AH, pvar: RAX
                    let shift = (svar.offset() - pvar.offset()) as usize * 8;

                    let hbits = ECodeExpr::extract_high(*pvar, pvar.bits() - svar.bits() - shift);
                    let lbits = ECodeExpr::extract_low(*pvar, shift);

                    ECodeExpr::concat(hbits, ECodeExpr::concat(expr, lbits))
                }
            },
            Ordering::Equal => expr,
        }
    }
    
    pub(crate) fn apply(mut self, ecode: &mut ECode) {
        struct AllVariables<'a, 'v>(&'a mut ECodeVarAliasNormalisePass<'v>);
        
        impl<'ecode, 'a, 'v> Visit<'ecode> for AllVariables<'a, 'v> {
            fn visit_var(&mut self, var: &'ecode ECodeVar) {
                self.0.insert(var);
            }
        }
        
        let mut visit = AllVariables(&mut self);

        // index variables in the ECode block
        for op in ecode.operations().iter() {
            visit.visit_stmt(op);
        }

        // rewrite variables in the ECode block
        for op in ecode.operations_mut().iter_mut() {
            self.visit_stmt_mut(op);
        }
    }
}

impl<'ecode, 'v> VisitMut<'ecode> for ECodeVarAliasNormalisePass<'v> {
    fn visit_expr_mut(&mut self, expr: &'ecode mut ECodeExpr) {
        match expr {
            ECodeExpr::UnRel(op, ref mut expr) => self.visit_expr_unrel_mut(*op, expr),
            ECodeExpr::UnOp(op, ref mut expr) => self.visit_expr_unop_mut(*op, expr),
            ECodeExpr::BinRel(op, ref mut lexpr, ref mut rexpr) => {
                self.visit_expr_binrel_mut(*op, lexpr, rexpr)
            }
            ECodeExpr::BinOp(op, ref mut lexpr, ref mut rexpr) => {
                self.visit_expr_binop_mut(*op, lexpr, rexpr)
            }
            ECodeExpr::Cast(ref mut expr, ref mut cast) => self.visit_expr_cast_mut(expr, cast),
            ECodeExpr::Load(ref mut expr, size, space) => {
                self.visit_expr_load_mut(expr, *size, *space)
            }
            ECodeExpr::Extract(ref mut expr, lsb, msb) => self.visit_expr_extract_mut(expr, *lsb, *msb),
            ECodeExpr::Concat(ref mut lexpr, ref mut rexpr) => self.visit_expr_concat_mut(lexpr, rexpr),
            ECodeExpr::IfElse(ref mut cond, ref mut texpr, ref mut fexpr) => self.visit_expr_ite_mut(cond, texpr, fexpr),
            ECodeExpr::Call(ref mut branch_target, ref mut args, bits) => {
                self.visit_expr_call_mut(branch_target, args, *bits)
            }
            ECodeExpr::Intrinsic(ref name, ref mut args, bits) => {
                self.visit_expr_intrinsic_mut(name, args, *bits)
            }
            ECodeExpr::Var(ref mut var) => {
                let svar = *var;
                let pvar = self.enclosing(&svar);

                let rvar = pvar;
                *expr = Self::resize_expr(&pvar, &svar, ECodeExpr::from(rvar));
            },
            ECodeExpr::Val(_) => (),
        }
    }

    fn visit_stmt_assign_mut(&mut self, var: &'ecode mut ECodeVar, expr: &'ecode mut ECodeExpr) {
        let svar = *var;
        let pvar = self.enclosing(&svar);

        // expand
        self.visit_expr_mut(expr);

        let rvar = ECodeVar::new(pvar.space(), pvar.offset(), pvar.bits(), var.generation());
        *expr = Self::resize_expr(&svar, &pvar, &*expr);
        *var = rvar;
    }
}