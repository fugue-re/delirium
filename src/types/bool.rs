use std::borrow::Cow;

use crate::prelude::{Id, Identifiable};
use crate::types::{Type, TypeSort, type_uuid};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoolT;

pub const BOOL: BoolT  = BoolT;

impl Identifiable<Type> for BoolT {
    fn id(&self) -> Id<Type> {
        Id::from_parts("type", type_uuid(0x13374eaf1b4db8d4))
    }
}

impl TypeSort for BoolT {
    fn name(&self) -> Cow<str> {
        Cow::Borrowed("bool")
    }
    
    fn bits(&self) -> u32 {
        8
    }
    
    fn bytes(&self) -> Option<usize> {
        Some(1)
    }
    
    fn is_primitive(&self) -> bool {
        true
    }
}