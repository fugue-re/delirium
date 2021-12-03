use std::borrow::Cow;
use ron_uuid::UUID;

use crate::prelude::{Erased, Identifiable};

pub mod bool;
pub mod bv;
pub mod float;
pub mod pointer;

pub use self::bool::BOOL;
pub use self::bv::{U8, U16, U32, U64, U128, U256, U512, I8, I16, I32, I64, I128, I256, I512};
pub use self::float::{F32, F64, F80};

const TYPE_SCOPE: u64 = 0x21341e3f58957821;

pub const fn type_uuid(id: u64) -> UUID {
    UUID::Name { scope: TYPE_SCOPE, name: id }
}

pub type Type = Erased;

pub trait TypeSort: Identifiable<Type> {
    fn name(&self) -> Cow<str>;

    fn bits(&self) -> u32;
    fn bytes(&self) -> Option<usize>;
    
    fn is_primitive(&self) -> bool;
    fn is_composite(&self) -> bool {
        !self.is_primitive()
    }
}