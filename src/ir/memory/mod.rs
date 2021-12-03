pub mod address;
pub use address::Address;

pub mod region;
pub use region::Region;

use crate::prelude::intervals::collections::IntervalMap;
use crate::prelude::{Id, Identifiable, Entity};

use std::borrow::Cow;

#[derive(Clone)]
pub struct Memory<'r> {
    id: Id<Memory<'r>>,
    name: Cow<'static, str>,
    mapping: IntervalMap<Address, Entity<Region<'r>>>,
}

impl<'r> Identifiable<Memory<'r>> for Memory<'r> {
    fn id(&self) -> Id<Self> {
        self.id
    }
}

impl<'r> Memory<'r> {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            id: Id::new("memory"),
            name: name.into(),
            mapping: IntervalMap::default(),
        }
    }
    
    pub fn name(&self) -> Cow<str> {
        Cow::Borrowed(&*self.name)
    }
}