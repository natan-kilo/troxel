use std::any::Any;
use std::ops::Index;

pub trait Stateful: Any {
    fn render(&mut self);
    fn update(&mut self);
    fn input(&mut self);
    fn id(&self) -> usize;
    fn box_eq(&self, other: &dyn Any) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl IntoIterator for Box<dyn Stateful> {
    type Item = Box<dyn Stateful>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}