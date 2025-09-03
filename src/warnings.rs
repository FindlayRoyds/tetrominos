// use bevy::{platform::collections::HashSet, prelude::*};

// pub fn warnings_plugin(app: &mut App) {
//     app.insert_resource(WarningsLog::default());
// }

// #[derive(Resource, Default)]
// pub struct WarningsLog(pub HashSet<String>);

/// Generic helper and macro to unwrap `Option<T>` or `Result<T, E>` and return early
/// from the calling function while emitting an error log.
///
/// Usage:
/// ```
/// let a = try_unwrap!(some_option, "missing value");
/// let b = try_unwrap!(some_result, "failed to compute");
/// ```
use std::fmt::Display;

/// Trait used to convert `Option<T>` and `Result<T, E>` into `Option<T>` while
/// emitting the appropriate error-level log. Implementations are inlined to
/// avoid runtime overhead.
pub trait IntoOptionWithError<T> {
    fn into_option_with_error(self, msg: &str) -> Option<T>;
}

impl<T> IntoOptionWithError<T> for Option<T> {
    #[inline]
    fn into_option_with_error(self, msg: &str) -> Option<T> {
        match self {
            Some(v) => Some(v),
            None => {
                bevy::log::error!("{}", msg);
                None
            }
        }
    }
}

impl<T, E: Display> IntoOptionWithError<T> for Result<T, E> {
    #[inline]
    fn into_option_with_error(self, msg: &str) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                bevy::log::error!("{}: {}", msg, e);
                None
            }
        }
    }
}

/// Helper function used by the macro. Marked #[inline] so the call is
/// eliminated in optimized builds.
#[inline]
pub fn try_into_option_error<T, U>(val: U, msg: &str) -> Option<T>
where
    U: IntoOptionWithError<T>,
{
    val.into_option_with_error(msg)
}

/// Single macro that works for both `Option<T>` and `Result<T, E>`.
#[macro_export]
macro_rules! try_unwrap {
    ($expr:expr, $msg:expr) => {
        match $crate::warnings::try_into_option_error($expr, $msg) {
            Some(value) => value,
            None => return,
        }
    };
}
