use std::borrow::Borrow;
use std::sync::Arc;

use crate::prelude::{Id, Identifiable, Entity, Erased};

use crate::ir::memory::Memory;
use crate::types::{Type, TypeSort};


pub enum VarKind {
    Memory {
        id: Id<Erased>,
    },
    Physical {
        typ: Id<Type>,
        bits: u32,
    },
    Transient {
        typ: Id<Type>,
        bits: u32,
    },
}

pub struct Var {
    name: Arc<str>,
    kind: VarKind,
    generation: u32,
}

impl Var {
    fn new(name: impl Borrow<str>, kind: VarKind) -> Entity<Self> {
        Entity::new("var", Self {
            name: Arc::from(name.borrow()),
            kind,
            generation: 0,
        })
    }
    
    pub fn memory(memory: &Entity<Memory>) -> Entity<Self> {
        Entity::new("var", Self {
            name: Arc::from(memory.name()),
            kind: VarKind::Memory {
                id: memory.id().erase(),
            },
            generation: 0,
        })
    }
    
    pub fn physical(name: impl Borrow<str>, typ: impl TypeSort) -> Entity<Self> {
        Self::new(name, VarKind::Physical {
            typ: typ.id(),
            bits: typ.bits(),
        })
    }

    pub fn transient(name: impl Borrow<str>, typ: impl TypeSort) -> Entity<Self> {
        Self::new(name, VarKind::Transient {
            typ: typ.id(),
            bits: typ.bits(),
        })
    }
    
    pub fn name(&self) -> &Arc<str> {
        &self.name
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
    
    pub fn is_memory(&self) -> bool {
        matches!(self.kind, VarKind::Memory { .. })
    }

    pub fn is_physical(&self) -> bool {
        matches!(self.kind, VarKind::Physical { .. })
    }

    pub fn is_transient(&self) -> bool {
        matches!(self.kind, VarKind::Transient { .. })
    }

    pub fn is_typed(&self) -> bool {
        !self.is_memory()
    }
    
    pub fn is_sized(&self) -> bool {
        !self.is_memory()
    }

    pub fn region_id(&self) -> Option<Id<Memory>> {
        match self.kind {
            VarKind::Memory { id } => Some(id.transmute()),
            _ => None,
        }
    }
    
    pub fn type_id(&self) -> Option<Id<Type>> {
        match self.kind {
            VarKind::Physical { typ, .. } | VarKind::Transient { typ, .. } => Some(typ),
            VarKind::Memory { .. } => None,
        }
    }
    
    pub fn bits(&self) -> Option<u32> {
        match self.kind {
            VarKind::Physical { bits, .. } | VarKind::Transient { bits, .. } => Some(bits),
            VarKind::Memory { .. } => None,
        }
    }
}