use core::cell::{Ref, RefCell, RefMut};
use core::future::Future;

use crate::{
    error::{LockError, Result},
    locking::{LockApiReadGuard, LockApiReadWriteGuard},
};

pub trait AsyncLockApi<T> {
    type ReadGuard<'a>: LockApiReadGuard<'a, T>
    where
        Self: 'a;
    type ReadWriteGuard<'a>: LockApiReadWriteGuard<'a, T>
    where
        Self: 'a;

    type ReadFuture<'a>: Future<Output = Result<Self::ReadGuard<'a>>>
    where
        Self: 'a;

    type WriteFuture<'a>: Future<Output = Result<Self::ReadWriteGuard<'a>>>
    where
        Self: 'a;

    fn read(&self) -> Self::ReadFuture<'_>;

    fn write(&self) -> Self::WriteFuture<'_>;

    fn new(inner: T) -> Self;
}

impl<T> AsyncLockApi<T> for RefCell<T>
where
    for<'a> T: 'a,
{
    type ReadGuard<'a> = Ref<'a, T>;

    type ReadWriteGuard<'a> = RefMut<'a, T>;

    type ReadFuture<'a> = core::future::Ready<Result<Self::ReadGuard<'a>>>;

    type WriteFuture<'a> = core::future::Ready<Result<Self::ReadWriteGuard<'a>>>;

    fn read(&self) -> Self::ReadFuture<'_> {
        core::future::ready(self.try_borrow().map_err(|_| LockError))
    }

    fn write(&self) -> Self::WriteFuture<'_> {
        core::future::ready(self.try_borrow_mut().map_err(|_| LockError))
    }

    fn new(inner: T) -> Self {
        RefCell::new(inner)
    }
}

#[cfg(feature = "async-lock")]
mod async_lock_impl {
    use super::AsyncLockApi;
    use crate::{
        error::Result,
        locking::{LockApiReadGuard, LockApiReadWriteGuard},
    };
    use async_lock::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
    use core::{
        future::Future,
        ops::{Deref, DerefMut},
        pin::Pin,
        task::{ready, Poll},
    };
    use pin_project_lite::pin_project;

    impl<'a, T> LockApiReadGuard<'a, T> for MutexGuard<'a, T> {
        fn get(&self) -> &T {
            self.deref()
        }
    }

    impl<'a, T> LockApiReadWriteGuard<'a, T> for MutexGuard<'a, T> {
        fn get_mut(&mut self) -> &mut T {
            self.deref_mut()
        }
    }

    impl<T> AsyncLockApi<T> for Mutex<T>
    where
        T: Send,
        for<'a> T: 'a,
    {
        type ReadGuard<'a> = MutexGuard<'a, T>;

        type ReadWriteGuard<'a> = MutexGuard<'a, T>;

        type ReadFuture<'a> = FutureResult<async_lock::futures::Lock<'a, T>>;

        type WriteFuture<'a> = FutureResult<async_lock::futures::Lock<'a, T>>;

        fn read(&self) -> Self::ReadFuture<'_> {
            FutureResult {
                future: self.lock(),
            }
        }

        fn write(&self) -> Self::WriteFuture<'_> {
            FutureResult {
                future: self.lock(),
            }
        }

        fn new(inner: T) -> Self {
            Mutex::new(inner)
        }
    }

    // RwLock

    impl<'a, T> LockApiReadGuard<'a, T> for RwLockReadGuard<'a, T> {
        fn get(&self) -> &T {
            self.deref()
        }
    }

    impl<'a, T> LockApiReadGuard<'a, T> for RwLockWriteGuard<'a, T> {
        fn get(&self) -> &T {
            self.deref()
        }
    }

    impl<'a, T> LockApiReadWriteGuard<'a, T> for RwLockWriteGuard<'a, T> {
        fn get_mut(&mut self) -> &mut T {
            self.deref_mut()
        }
    }

    impl<T> AsyncLockApi<T> for RwLock<T>
    where
        T: Send,
        for<'a> T: 'a,
    {
        type ReadGuard<'a> = RwLockReadGuard<'a, T>;

        type ReadWriteGuard<'a> = RwLockWriteGuard<'a, T>;

        type ReadFuture<'a> = FutureResult<async_lock::futures::Read<'a, T>>;

        type WriteFuture<'a> = FutureResult<async_lock::futures::Write<'a, T>>;

        fn read(&self) -> Self::ReadFuture<'_> {
            FutureResult {
                future: self.read(),
            }
        }

        fn write(&self) -> Self::WriteFuture<'_> {
            FutureResult {
                future: self.write(),
            }
        }
        fn new(inner: T) -> Self {
            RwLock::new(inner)
        }
    }

    pin_project! {
        pub struct FutureResult<F> {
            #[pin]
            future: F
        }
    }

    impl<F> Future for FutureResult<F>
    where
        F: Future,
    {
        type Output = Result<F::Output>;

        fn poll(
            self: Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
        ) -> core::task::Poll<Self::Output> {
            let ret = ready!(self.project().future.poll(cx));
            Poll::Ready(Ok(ret))
        }
    }
}
