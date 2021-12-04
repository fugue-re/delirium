use crate::ir::Addr;
use std::collections::BTreeSet;

pub trait BlkOracle {
    fn blk_size(&self, addr: &Addr) -> Option<usize>;
    fn blk_jmps(&self, addr: &Addr) -> BTreeSet<Addr>;
}

pub trait SubOracle {
    fn sub_starts(&self) -> BTreeSet<Addr>;
    fn sub_symbol(&self, addr: &Addr) -> Option<String>;
    fn sub_blocks(&self, addr: &Addr) -> BTreeSet<Addr>;
}