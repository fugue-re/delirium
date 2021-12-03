use ron_uuid::UUID;
use std::fmt::{self, Display};
use std::marker::PhantomData;

use crate::prelude::Erased;

#[derive(educe::Educe)]
#[educe(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<T> {
    tag: &'static str,
    uuid: UUID,
    #[educe(Debug(ignore), PartialEq(ignore), Eq(ignore), PartialOrd(ignore), Ord(ignore), Hash(ignore))]
    marker: PhantomData<T>,
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.tag, self.uuid)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self {
            tag: self.tag,
            uuid: self.uuid,
            marker: PhantomData,
        }
    }
}
impl<T> Copy for Id<T> { }

impl<T> Id<T> {
    pub fn new(tag: &'static str) -> Self {
        Self::from_parts(tag, UUID::now())
    }
    
    pub const fn from_parts(tag: &'static str, uuid: UUID) -> Self {
        Self {
            tag,
            uuid,
            marker: PhantomData,
        }
    }
    
    pub fn erase(self) -> Id<Erased> {
        self.transmute::<Erased>()
    }
    
    pub fn transmute<U>(self) -> Id<U> {
        Id {
            tag: self.tag,
            uuid: self.uuid,
            marker: PhantomData,
        }
    }
    
    pub fn invalid(tag: &'static str) -> Self {
        Self {
            tag,
            uuid: UUID::zero(),
            marker: PhantomData,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.is_invalid()
    }
    
    pub fn is_invalid(&self) -> bool {
        self.uuid.is_zero()
    }
    
    pub fn tag(&self) -> &'static str {
        self.tag
    }
    
    pub fn uuid(&self) -> UUID {
        self.uuid
    }
}