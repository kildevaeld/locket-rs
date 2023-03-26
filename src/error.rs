pub type Result<T> = core::result::Result<T, LockError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockError;

impl core::fmt::Display for LockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "lock failed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LockError {}
