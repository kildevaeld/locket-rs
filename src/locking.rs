use crate::error::{LockError, Result};
use core::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
};

pub trait LockApiReadGuard<'a, T> {
    fn get(&self) -> &T;
}

pub trait LockApiWriteGuard<'a, T>: LockApiReadGuard<'a, T> {
    fn get_mut(&mut self) -> &mut T;
}

pub trait LockApi<T> {
    type ReadGuard<'a>: LockApiReadGuard<'a, T>
    where
        Self: 'a;
    type WriteGuard<'a>: LockApiWriteGuard<'a, T>
    where
        Self: 'a;

    fn read(&self) -> Result<Self::ReadGuard<'_>>;

    fn write(&self) -> Result<Self::WriteGuard<'_>>;

    fn new(inner: T) -> Self;
}

impl<'a, T> LockApiReadGuard<'a, T> for Ref<'a, T> {
    fn get(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> LockApiReadGuard<'a, T> for RefMut<'a, T> {
    fn get(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> LockApiWriteGuard<'a, T> for RefMut<'a, T> {
    fn get_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<T> LockApi<T> for RefCell<T>
where
    for<'a> T: 'a,
{
    type ReadGuard<'a> = Ref<'a, T>;

    type WriteGuard<'a> = RefMut<'a, T>;

    fn read(&self) -> Result<Self::ReadGuard<'_>> {
        self.try_borrow().map_err(|_| LockError)
    }

    fn write(&self) -> Result<Self::WriteGuard<'_>> {
        self.try_borrow_mut().map_err(|_| LockError)
    }

    fn new(inner: T) -> Self {
        RefCell::new(inner)
    }
}

#[cfg(feature = "parking_lot")]
mod parking_lot_impl {
    // Mutex
    use super::*;
    use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

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

    impl<T> LockApi<T> for Mutex<T>
    where
        for<'a> T: 'a,
    {
        type ReadGuard<'a> = MutexGuard<'a, T>;

        type WriteGuard<'a> = MutexGuard<'a, T>;

        fn read(&self) -> Result<Self::ReadGuard<'_>> {
            Ok(self.lock())
        }

        fn write(&self) -> Result<Self::WriteGuard<'_>> {
            Ok(self.lock())
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

    impl<T> LockApi<T> for RwLock<T>
    where
        for<'a> T: 'a,
    {
        type ReadGuard<'a> = RwLockReadGuard<'a, T>;

        type WriteGuard<'a> = RwLockWriteGuard<'a, T>;

        fn read(&self) -> Result<Self::ReadGuard<'_>> {
            Ok((*self).read())
        }

        fn write(&self) -> Result<Self::WriteGuard<'_>> {
            Ok((*self).write())
        }

        fn new(inner: T) -> Self {
            RwLock::new(inner)
        }
    }
}

#[cfg(feature = "spin")]
mod spin_impl {
    // Mutex
    use super::*;
    use spin::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

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

    impl<T> LockApi<T> for Mutex<T>
    where
        for<'a> T: 'a,
    {
        type ReadGuard<'a> = MutexGuard<'a, T>;

        type WriteGuard<'a> = MutexGuard<'a, T>;

        fn read(&self) -> Result<Self::ReadGuard<'_>> {
            Ok(self.lock())
        }

        fn write(&self) -> Result<Self::WriteGuard<'_>> {
            Ok(self.lock())
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

    impl<T> LockApi<T> for RwLock<T>
    where
        for<'a> T: 'a,
    {
        type ReadGuard<'a> = RwLockReadGuard<'a, T>;

        type WriteGuard<'a> = RwLockWriteGuard<'a, T>;

        fn read(&self) -> Result<Self::ReadGuard<'_>> {
            Ok((*self).read())
        }

        fn write(&self) -> Result<Self::WriteGuard<'_>> {
            Ok((*self).write())
        }

        fn new(inner: T) -> Self {
            RwLock::new(inner)
        }
    }
}

#[cfg(feature = "std-lock")]
mod std_impl {
    // Mutex
    use super::*;
    use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

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

    impl<T> LockApi<T> for Mutex<T>
    where
        for<'a> T: 'a,
    {
        type ReadGuard<'a> = MutexGuard<'a, T>;

        type WriteGuard<'a> = MutexGuard<'a, T>;

        fn read(&self) -> Result<Self::ReadGuard<'_>> {
            self.lock().map_err(|_| LockError)
        }

        fn write(&self) -> Result<Self::WriteGuard<'_>> {
            self.lock().map_err(|_| LockError)
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

    impl<T> LockApi<T> for RwLock<T>
    where
        for<'a> T: 'a,
    {
        type ReadGuard<'a> = RwLockReadGuard<'a, T>;

        type WriteGuard<'a> = RwLockWriteGuard<'a, T>;

        fn read(&self) -> Result<Self::ReadGuard<'_>> {
            (*self).read().map_err(|_| LockError)
        }

        fn write(&self) -> Result<Self::WriteGuard<'_>> {
            (*self).write().map_err(|_| LockError)
        }

        fn new(inner: T) -> Self {
            RwLock::new(inner)
        }
    }
}
