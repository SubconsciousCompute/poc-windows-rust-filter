use crate::locker::Locker;

pub struct AutoLock<'a, Lock: Locker> {
    l: &'a mut Lock,
}

impl<'a, Lock: Locker> AutoLock<'a, Lock> {
    pub fn new(locker: &'a mut Lock) -> Self {
        locker.Lock();

        Self { l: locker }
    }
}

impl<'a, Lock: Locker> Drop for AutoLock<'a, Lock> {
    fn drop(&mut self) {
        self.l.Unlock();
    }
}