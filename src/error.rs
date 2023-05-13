use proc_macro2::Span;
use std::fmt::Display;
use syn::{Error, Result};

fn error<M>(span: Span, message: M) -> Error
where
    M: Display,
{
    Error::new(span, message)
}

pub(crate) fn error_res<M, T>(span: Span, message: M) -> Result<T>
where
    M: Display,
{
    Err(error(span, message))
}
