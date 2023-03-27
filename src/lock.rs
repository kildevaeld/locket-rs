use alloc::{rc::Rc, sync::Arc};

use crate::{Downgrade, LockApi};

pub trait Locket<T>: LockApi<T> + Downgrade + Clone {}

impl<T, L> Locket<T> for L where L: LockApi<T> + Downgrade + Clone {}

impl<L, T> LockApi<T> for Arc<L>
where
    L: LockApi<T>,
    for<'a> L: 'a,
{
    type ReadGuard<'a> = L::ReadGuard<'a>;

    type WriteGuard<'a> = L::WriteGuard<'a>;

    fn read(&self) -> crate::error::Result<Self::ReadGuard<'_>> {
        (**self).read()
    }

    fn write(&self) -> crate::error::Result<Self::WriteGuard<'_>> {
        (**self).write()
    }

    fn new(inner: T) -> Self {
        Arc::new(L::new(inner))
    }
}

impl<L, T> LockApi<T> for Rc<L>
where
    L: LockApi<T>,
    for<'a> L: 'a,
{
    type ReadGuard<'a> = L::ReadGuard<'a>;

    type WriteGuard<'a> = L::WriteGuard<'a>;

    fn read(&self) -> crate::error::Result<Self::ReadGuard<'_>> {
        (**self).read()
    }

    fn write(&self) -> crate::error::Result<Self::WriteGuard<'_>> {
        (**self).write()
    }

    fn new(inner: T) -> Self {
        Rc::new(L::new(inner))
    }
}
