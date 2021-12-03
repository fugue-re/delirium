use crate::prelude::Id;

pub trait Identifiable<V> {
    fn id(&self) -> Id<V>;
}