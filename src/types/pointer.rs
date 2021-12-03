use std::borrow::Cow;

use crate::prelude::{Id, Identifiable};
use crate::types::{Type, TypeSort, type_uuid};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PointerT {
    id: Id<Type>,
    pointee: Id<Type>,
    bits: u32,
}

impl PointerT {
    pub fn new(pointee: impl TypeSort, bits: u32, id: u64) -> Self {
        Self {
            id: Id::from_parts("type", type_uuid(id)),
            pointee: pointee.id(),
            bits,
        }
    }
    
    pub fn pointee_type(&self) -> Id<Type> {
        self.pointee
    }
}

impl Identifiable<Type> for PointerT {
    fn id(&self) -> Id<Type> {
        self.id
    }
}

impl TypeSort for PointerT {
    fn name(&self) -> Cow<str> {
        Cow::Owned(format!("ptr{}", self.bits))
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
        false
    }
}