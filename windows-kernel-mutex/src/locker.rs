#![allow(non_snake_case)]
pub trait Locker {
    fn Init(&mut self);

    fn Lock(&mut self);

    fn Unlock(&mut self);
}