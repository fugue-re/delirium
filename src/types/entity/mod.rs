use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use crate::traits::Identifiable;
use crate::types::Id;

#[derive(Debug, Clone)]
pub struct Entity<V> {
    id: Id<V>,
    value: V,
}

impl<V> From<V> for Entity<V> where V: Identifiable<V> {
    fn from(value: V) -> Self {
        Self {
            id: value.id(),
            value,
        }
    }
}

impl<V> PartialEq for Entity<V> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<V> Eq for Entity<V> {}

impl<V> Ord for Entity<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}
impl<V> PartialOrd for Entity<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V> Hash for Entity<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl<V> Deref for Entity<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V> DerefMut for Entity<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<V> Entity<V> {
    pub fn new(tag: &'static str, value: V) -> Self {
        Self {
            id: Id::new(tag),
            value,
        }
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }

    pub fn into_value(self) -> V {
        self.value
    }

    pub fn from_parts(id: Id<V>, value: V) -> Self {
        Self {
            id,
            value,
        }
    }

    pub fn into_parts(self) -> (Id<V>, V) {
        (self.id, self.value)
    }
}

impl<V> Identifiable<V> for Entity<V> {
    fn id(&self) -> Id<V> {
        self.id
    }
}