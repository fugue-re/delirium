use std::borrow::Cow;

use crate::prelude::{Id, Identifiable};
use crate::types::{Type, TypeSort, type_uuid};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FloatT(Id<Type>, u32);

pub const F32: FloatT  = FloatT::new(32, 0xb2e5c631d50d5436);
pub const F64: FloatT  = FloatT::new(64, 0xb15fba32c4f1e09c);
pub const F80: FloatT  = FloatT::new(80, 0xfa8604c26e19216);

impl FloatT {
    pub const fn new(bits: u32, id: u64) -> Self {
        Self(Id::from_parts("type", type_uuid(id)), bits)
    }
}

impl Identifiable<Type> for FloatT {
    fn id(&self) -> Id<Type> {
        self.0
    }
}

impl TypeSort for FloatT {
    fn name(&self) -> Cow<str> {
        Cow::Owned(format!("f{}", self.1))
    }
    
    fn bits(&self) -> u32 {
        self.1
    }
    
    fn bytes(&self) -> Option<usize> {
        if self.1 % 8 == 0 {
            Some(self.1 as usize / 8)
        } else {
            None
        }
    }
    
    fn is_primitive(&self) -> bool {
        true
    }
}