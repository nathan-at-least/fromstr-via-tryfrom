use crate::error_res;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{ItemImpl, Path, Result, Type, TypePath};

pub(crate) trait GetPath {
    fn get_path(&self) -> Result<&Path>;
}

// This impl gets the trait path from `ItemImpl::trait_`
impl GetPath for ItemImpl {
    fn get_path(&self) -> Result<&Path> {
        if let Some((optnot, path, _)) = &self.trait_ {
            if optnot.is_some() {
                error_res(self.span(), "! unsupported")
            } else {
                Ok(path)
            }
        } else {
            error_res(self.span(), "non-trait impl unsupported")
        }
    }
}

impl<T> GetPath for Box<T>
where
    T: GetPath,
{
    fn get_path(&self) -> Result<&Path> {
        use std::ops::Deref;

        self.deref().get_path()
    }
}

impl GetPath for Type {
    fn get_path(&self) -> Result<&Path> {
        if let Type::Path(tp) = self {
            tp.get_path()
        } else {
            error_res(
                self.span(),
                format!(
                    "expected path, found {:?}",
                    self.to_token_stream().to_string()
                ),
            )
        }
    }
}

impl GetPath for TypePath {
    fn get_path(&self) -> Result<&Path> {
        if self.qself.is_none() {
            Ok(&self.path)
        } else {
            error_res(self.span(), "expected simple path, found {self:?}")
        }
    }
}
