use core::cell::{Ref, RefCell, RefMut};
use core::future::Future;

use crate::{
    error::{LockError, Result},
    locking::{LockApiReadGuard, LockApiWriteGuard},
};

pub trait AsyncLockApi<T> {
    type ReadGuard<'a>: LockApiReadGuard<'a, T>
    where
        Self: 'a;
    type WriteGuard<'a>: LockApiWriteGuard<'a, T>
    where
        Self: 'a;

    type ReadFuture<'a>: Future<Output = Result<Self::ReadGuard<'a>>>
    where
        Self: 'a;

    type WriteFuture<'a>: Future<Output = Result<Self::WriteGuard<'a>>>
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

    type WriteGuard<'a> = RefMut<'a, T>;

    type ReadFuture<'a> = core::future::Ready<Result<Self::ReadGuard<'a>>>;

    type WriteFuture<'a> = core::future::Ready<Result<Self::WriteGuard<'a>>>;

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
        locking::{LockApiReadGuard, LockApiWriteGuard},
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

    impl<'a, T> LockApiWriteGuard<'a, T> for MutexGuard<'a, T> {
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

        type WriteGuard<'a> = MutexGuard<'a, T>;

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

    impl<'a, T> LockApiWriteGuard<'a, T> for RwLockWriteGuard<'a, T> {
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

        type WriteGuard<'a> = RwLockWriteGuard<'a, T>;

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

#[cfg(feature = "tokio")]
mod tokio_impl {
    use super::AsyncLockApi;
    use crate::{
        error::Result,
        locking::{LockApiReadGuard, LockApiWriteGuard},
    };
    use core::{
        future::Future,
        ops::{Deref, DerefMut},
        pin::Pin,
    };

    use alloc::boxed::Box;
    use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

    impl<'a, T> LockApiReadGuard<'a, T> for MutexGuard<'a, T> {
        fn get(&self) -> &T {
            self.deref()
        }
    }

    impl<'a, T> LockApiWriteGuard<'a, T> for MutexGuard<'a, T> {
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

        type WriteGuard<'a> = MutexGuard<'a, T>;

        type ReadFuture<'a> = BoxFuture<'a, Result<Self::ReadGuard<'a>>>;

        type WriteFuture<'a> = BoxFuture<'a, Result<Self::WriteGuard<'a>>>;

        fn read(&self) -> Self::ReadFuture<'_> {
            Box::pin(async move { Ok(self.lock().await) })
        }

        fn write(&self) -> Self::WriteFuture<'_> {
            Box::pin(async move { Ok(self.lock().await) })
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

    impl<'a, T> LockApiWriteGuard<'a, T> for RwLockWriteGuard<'a, T> {
        fn get_mut(&mut self) -> &mut T {
            self.deref_mut()
        }
    }

    impl<T> AsyncLockApi<T> for RwLock<T>
    where
        T: Send + Sync,
        for<'a> T: 'a,
    {
        type ReadGuard<'a> = RwLockReadGuard<'a, T>;

        type WriteGuard<'a> = RwLockWriteGuard<'a, T>;

        type ReadFuture<'a> = BoxFuture<'a, Result<Self::ReadGuard<'a>>>;

        type WriteFuture<'a> = BoxFuture<'a, Result<Self::WriteGuard<'a>>>;

        fn read(&self) -> Self::ReadFuture<'_> {
            Box::pin(async move { Ok(self.read().await) })
        }

        fn write(&self) -> Self::WriteFuture<'_> {
            Box::pin(async move { Ok(self.write().await) })
        }

        fn new(inner: T) -> Self {
            RwLock::new(inner)
        }
    }
}

#[cfg(all(feature = "async-std", not(feature = "async-lock")))]
mod async_std_impl {
    use super::AsyncLockApi;
    use crate::{
        error::Result,
        locking::{LockApiReadGuard, LockApiWriteGuard},
    };
    use alloc::boxed::Box;
    use async_std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
    use core::{
        future::Future,
        ops::{Deref, DerefMut},
        pin::Pin,
    };

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

    impl<'a, T> LockApiReadGuard<'a, T> for MutexGuard<'a, T> {
        fn get(&self) -> &T {
            self.deref()
        }
    }

    impl<'a, T> LockApiWriteGuard<'a, T> for MutexGuard<'a, T> {
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

        type WriteGuard<'a> = MutexGuard<'a, T>;

        type ReadFuture<'a> = BoxFuture<'a, Result<Self::ReadGuard<'a>>>;

        type WriteFuture<'a> = BoxFuture<'a, Result<Self::WriteGuard<'a>>>;

        fn read(&self) -> Self::ReadFuture<'_> {
            Box::pin(async move { Ok(self.lock().await) })
        }

        fn write(&self) -> Self::WriteFuture<'_> {
            Box::pin(async move { Ok(self.lock().await) })
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

    impl<'a, T> LockApiWriteGuard<'a, T> for RwLockWriteGuard<'a, T> {
        fn get_mut(&mut self) -> &mut T {
            self.deref_mut()
        }
    }

    impl<T> AsyncLockApi<T> for RwLock<T>
    where
        T: Send + Sync,
        for<'a> T: 'a,
    {
        type ReadGuard<'a> = RwLockReadGuard<'a, T>;

        type WriteGuard<'a> = RwLockWriteGuard<'a, T>;

        type ReadFuture<'a> = BoxFuture<'a, Result<Self::ReadGuard<'a>>>;

        type WriteFuture<'a> = BoxFuture<'a, Result<Self::WriteGuard<'a>>>;

        fn read(&self) -> Self::ReadFuture<'_> {
            Box::pin(async move { Ok(self.read().await) })
        }

        fn write(&self) -> Self::WriteFuture<'_> {
            Box::pin(async move { Ok(self.write().await) })
        }

        fn new(inner: T) -> Self {
            RwLock::new(inner)
        }
    }
}
