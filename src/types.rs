use alloc::{
    rc::{Rc, Weak as RcWeak},
    sync::{Arc, Weak as ArcWeak},
};

pub trait Downgrade {
    type Output: Upgrade<Output = Self>;
    fn downgrade(&self) -> Self::Output;
}

impl<T> Downgrade for Arc<T> {
    type Output = ArcWeak<T>;
    fn downgrade(&self) -> Self::Output {
        Arc::downgrade(self)
    }
}

impl<T> Downgrade for Rc<T> {
    type Output = RcWeak<T>;
    fn downgrade(&self) -> Self::Output {
        Rc::downgrade(self)
    }
}

pub trait Upgrade {
    type Output;
    fn upgrade(&self) -> Option<Self::Output>;
}

impl<T> Upgrade for ArcWeak<T> {
    type Output = Arc<T>;
    fn upgrade(&self) -> Option<Self::Output> {
        ArcWeak::upgrade(self)
    }
}

impl<T> Upgrade for RcWeak<T> {
    type Output = Rc<T>;
    fn upgrade(&self) -> Option<Self::Output> {
        RcWeak::upgrade(self)
    }
}

pub trait Lockable {
    type Guard<'a>
    where
        Self: 'a;
    fn lock(&self) -> Self::Guard<'_>;
}

#[cfg(feature = "std")]
impl Lockable for std::io::Stdout {
    type Guard<'a> = std::io::StdoutLock<'a>;
    fn lock(&self) -> Self::Guard<'_> {
        self.lock()
    }
}
