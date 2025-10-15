use std::ops::Deref;

#[derive(Clone)]
pub struct Described<T> {
    pub value: T,
    pub desc: &'static str,
}

impl<T> Deref for Described<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Described<T> {
    pub fn desc(&self) -> &'static str {
        self.desc
    }
}