use crate::types::Id;

pub trait Identifiable<V> {
    fn id(&self) -> Id<V>;
}