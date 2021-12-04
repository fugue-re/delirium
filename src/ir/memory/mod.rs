pub mod address;
pub use address::Addr;

pub mod region;
pub use region::Region;

use crate::prelude::intervals::collections::IntervalMap;
use crate::prelude::{Id, Identifiable, Entity, EntityRef};

use std::borrow::Cow;

#[derive(Clone)]
pub struct Mem<'r> {
    id: Id<Mem<'r>>,
    name: Cow<'static, str>,
    mapping: IntervalMap<Addr, Entity<Region<'r>>>,
}

impl<'r> Identifiable<Mem<'r>> for Mem<'r> {
    fn id(&self) -> Id<Self> {
        self.id
    }
}

impl<'r> Mem<'r> {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            id: Id::new("mem"),
            name: name.into(),
            mapping: IntervalMap::default(),
        }
    }
    
    pub fn name(&self) -> Cow<str> {
        Cow::Borrowed(&*self.name)
    }
    
    pub fn add_region(&mut self, region: Entity<Region<'r>>) {
        self.mapping.insert(region.interval().clone(), region);
    }
    
    pub fn find_region(&self, addr: &Addr) -> Option<EntityRef<Region<'r>>> {
        self.mapping.find_point(addr).map(|iv| EntityRef::Borrowed(iv.value()))
    }
    
    pub fn regions(&self) -> &IntervalMap<Addr, Entity<Region<'r>>> {
        &self.mapping
    }
}