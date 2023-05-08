pub mod bot;
pub mod creator;

/// Extend `Option` with a fallible map method
///
/// This is useful for mapping fallible operations (i.e. operations that)
/// return `Result`, over an optional type. The result will be
/// `Result<Option<U>>`, which makes it easy to handle the errors that
/// originate from inside the closure that's being mapped.
///
/// # Type parameters
///
/// - `T`: The input `Option`'s value type
/// - `U`: The outputs `Option`'s value type
/// - `E`: The possible error during the mapping
pub trait FallibleMapExt<T, U, E> {
    /// Try to apply a fallible map function to the option
    fn try_map<F>(self, f: F) -> Result<Option<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>;
}

// Implementions
impl<T, U, E> FallibleMapExt<T, U, E> for Option<T> {
    fn try_map<F>(self, f: F) -> Result<Option<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        match self {
            Some(x) => f(x).map(Some),
            None => Ok(None),
        }
    }
}
