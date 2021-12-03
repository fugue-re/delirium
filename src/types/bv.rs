use std::borrow::Cow;

use crate::prelude::{Id, Identifiable};
use crate::types::{Type, TypeSort, type_uuid};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitVecT {
    id: Id<Type>,
    signed: bool,
    bits: u32,
}

pub const U8: BitVecT  = BitVecT::new(8, false, 0x119e6d7d2b71a2ee);
pub const U16: BitVecT = BitVecT::new(16, false, 0xdf153e77940e8cb6);
pub const U32: BitVecT = BitVecT::new(32, false, 0xed7670e79be1004a);
pub const U64: BitVecT = BitVecT::new(64, false, 0x970642d009b7dbbf);
pub const U128: BitVecT = BitVecT::new(128, false, 0x9311f07f94067011);
pub const U256: BitVecT = BitVecT::new(256, false, 0x11181ac23564ef0c);
pub const U512: BitVecT = BitVecT::new(512, false, 0xfdd18500239ea271);

pub const I8: BitVecT  = BitVecT::new(8, true, 0xe8c5bdf5003305af);
pub const I16: BitVecT  = BitVecT::new(16, true, 0xe4f13e886256d086);
pub const I32: BitVecT  = BitVecT::new(32, true, 0xefc6825656833849);
pub const I64: BitVecT  = BitVecT::new(64, true, 0x842618f4caf73f92);
pub const I128: BitVecT  = BitVecT::new(128, true, 0x8967a93cbe0d3727);
pub const I256: BitVecT  = BitVecT::new(256, true, 0xba71d38ea5c5da7a);
pub const I512: BitVecT  = BitVecT::new(512, true, 0xb1222584f163fbef);

impl BitVecT {
    pub const fn new(bits: u32, signed: bool, id: u64) -> Self {
        Self {
            id: Id::from_parts("type", type_uuid(id)),
            signed,
            bits,
        }
    }
}

impl Identifiable<Type> for BitVecT {
    fn id(&self) -> Id<Type> {
        self.id
    }
}

impl TypeSort for BitVecT {
    fn name(&self) -> Cow<str> {
        Cow::Owned(format!("{}{}", if self.signed { "i" } else { "u" }, self.bits))
    }
    
    fn bits(&self) -> u32 {
        self.bits
    }
    
    fn bytes(&self) -> Option<usize> {
        if self.bits % 8 == 0 {
            Some(self.bits as usize / 8)
        } else {
            None
        }
    }
    
    fn is_primitive(&self) -> bool {
        true
    }
}