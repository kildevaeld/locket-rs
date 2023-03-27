use super::async_locking::AsyncLockApi;
use crate::Downgrade;
use alloc::{rc::Rc, sync::Arc};

pub trait AsyncLocket<T>: AsyncLockApi<T> + Downgrade + Clone {}

impl<T, L> AsyncLocket<T> for L where L: AsyncLockApi<T> + Downgrade + Clone {}

impl<L, T> AsyncLockApi<T> for Arc<L>
where
    L: AsyncLockApi<T>,
    for<'a> L: 'a,
{
    type ReadGuard<'a> = L::ReadGuard<'a>;

    type WriteGuard<'a> = L::WriteGuard<'a>;

    type ReadFuture<'a> = L::ReadFuture<'a>;
    type WriteFuture<'a> = L::WriteFuture<'a>;

    fn read(&self) -> Self::ReadFuture<'_> {
        (**self).read()
    }

    fn write(&self) -> Self::WriteFuture<'_> {
        (**self).write()
    }

    fn new(inner: T) -> Self {
        Arc::new(L::new(inner))
    }
}

impl<L, T> AsyncLockApi<T> for Rc<L>
where
    L: AsyncLockApi<T>,
    for<'a> L: 'a,
{
    type ReadGuard<'a> = L::ReadGuard<'a>;

    type WriteGuard<'a> = L::WriteGuard<'a>;

    type ReadFuture<'a> = L::ReadFuture<'a>;
    type WriteFuture<'a> = L::WriteFuture<'a>;

    fn read(&self) -> Self::ReadFuture<'_> {
        (**self).read()
    }

    fn write(&self) -> Self::WriteFuture<'_> {
        (**self).write()
    }

    fn new(inner: T) -> Self {
        Rc::new(L::new(inner))
    }
}
