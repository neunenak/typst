//! A prelude for building custom functions.

pub use crate::compute::value::*;
pub use crate::layout::prelude::*;
pub use crate::layout::Commands;
pub use crate::layout::Command::{self, *};
pub use crate::style::*;
pub use crate::syntax::parsing::parse;
pub use crate::syntax::span::{Pos, Span, SpanVec, Spanned};
pub use crate::syntax::tree::*;
pub use crate::{Pass, Feedback};
pub use super::*;

/// Extra methods on `Option`s used for function argument parsing.
pub trait OptionExt<T>: Sized {
    /// Call `f` with `val` if this is `Some(val)`.
    fn with(self, f: impl FnOnce(T));

    /// Report an error about a missing argument with the given name and span if
    /// the option is `None`.
    fn or_missing(self, span: Span, arg: &str, f: &mut Feedback) -> Self;
}

impl<T> OptionExt<T> for Option<T> {
    fn with(self, f: impl FnOnce(T)) {
        if let Some(val) = self {
            f(val);
        }
    }

    fn or_missing(self, span: Span, arg: &str, f: &mut Feedback) -> Self {
        if self.is_none() {
            error!(@f, span, "missing argument: {}", arg);
        }
        self
    }
}