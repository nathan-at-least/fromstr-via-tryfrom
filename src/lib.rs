#![doc = include_str!("../description.md")]
//!
//! See [macro@tryfrom_via_fromstr] for examples.
mod error;
mod getpath;

use self::error::error_res;
use self::getpath::GetPath;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;

#[doc = include_str!("../description.md")]
///
/// This assumes `FromStr` is defined and simply delegates to it.
///
/// # Example
///
/// ```
/// use tryfrom_via_fromstr::tryfrom_via_fromstr;
///
/// struct Cheer {
///     happy: bool,
/// }
///
/// #[tryfrom_via_fromstr]
/// impl std::str::FromStr for Cheer {
///     type Err = &'static str;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         if s == "YAY!" {
///             Ok(Cheer { happy: true })
///         } else if s == "BOO!" {
///             Ok(Cheer { happy: false })
///         } else {
///             Err("unknown cheer")
///         }
///     }
/// }
///
/// // We can use the `FromStr` impl from above:
/// let cheer: Cheer = "YAY!".parse()?;
/// assert!(cheer.happy);
///
/// // We can also use the newly derived `TryFrom` impls:
/// let cheer = Cheer::try_from("YAY!")?;
/// assert!(cheer.happy);
/// # Ok::<(), &'static str>(())
/// ```
///
/// The above snippet expands to something like this:
/// ```
/// # use std::str::FromStr;
/// # struct Cheer;
/// # impl FromStr for Cheer {
/// #     type Err = ();
/// #     fn from_str(_: &str) -> Result<Cheer, ()> {
/// #         Ok(Cheer)
/// #     }
/// # }
/// impl<'a> TryFrom<&'a str> for Cheer {
///     type Error = <Self as FromStr>::Err;
///
///     fn try_from(s: &'a str) -> Result<Self, Self::Error> {
///         Self::from_str(s)
///     }
/// }
/// ```
///
/// # Example with Generics
///
/// Derivation works for types with generics:
/// ```
/// use std::str::FromStr;
/// use tryfrom_via_fromstr::tryfrom_via_fromstr;
///
/// #[derive(Debug)]
/// struct Wrapper<T>(T);
///
/// #[tryfrom_via_fromstr]
/// impl<T> FromStr for Wrapper<T>
/// where T: FromStr,
///  {
///     type Err = <T as FromStr>::Err;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         s.parse().map(Wrapper)
///     }
/// }
///
/// let wv = Wrapper::<i64>::try_from("42")?;
/// assert_eq!(format!("{wv:?}"), "Wrapper(42)".to_string());
/// # Ok::<(), std::num::ParseIntError>(())
/// ```
///
/// Type constraints are carried through, so the above example expands
/// to something similar to:
///
/// ```
/// # use std::str::FromStr;
/// # struct Wrapper<T>(T);
/// # impl<T: FromStr> FromStr for Wrapper<T>  {
/// #     type Err = <T as FromStr>::Err;
/// #     fn from_str(s: &str) -> Result<Self, Self::Err> {
/// #         T::from_str(s).map(Wrapper)
/// #     }
/// # }
/// impl<'a, T> TryFrom<&'a str> for Wrapper<T>
/// where
///     T: FromStr,
/// {
///     type Error = <Self as FromStr>::Err;
///
///     fn try_from(s: &'a str) -> Result<Self, Self::Error> {
///         Self::from_str(s)
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn tryfrom_via_fromstr(args: TokenStream1, input: TokenStream1) -> TokenStream1 {
    TokenStream1::from(
        transform(TokenStream2::from(args), TokenStream2::from(input))
            .unwrap_or_else(syn::Error::into_compile_error),
    )
}

fn transform(args: TokenStream2, input: TokenStream2) -> syn::Result<TokenStream2> {
    parse_args(args)?;

    let itemimpl: syn::ItemImpl = syn::parse2(input)?;
    require_impl_for_fromstr(&itemimpl)?;

    let app_path = itemimpl.self_ty.get_path()?;

    // Construct new impl generics with a prefixed 'a lifetime for `&'a str` `TryFrom` impl:
    let tryfrom_generics = prefix_impl_lifetime(&itemimpl.generics);
    let (impl_generics, _, _) = tryfrom_generics.split_for_impl();
    let (_, _, where_clause) = itemimpl.generics.split_for_impl();

    Ok(quote! {
        #itemimpl

        impl #impl_generics ::std::convert::TryFrom<&'tryfrom_str_lifetime str> for #app_path #where_clause {
            type Error = <Self as ::std::str::FromStr>::Err;

            fn try_from(s: &'tryfrom_str_lifetime str) -> Result<Self, Self::Error> {
                use ::std::str::FromStr;

                Self::from_str(s)
            }
        }
    })
}

fn parse_args(args: TokenStream2) -> syn::Result<()> {
    if args.is_empty() {
        Ok(())
    } else {
        error_res(args.span(), "no arguments supported")
    }
}

fn require_impl_for_fromstr(itemimpl: &syn::ItemImpl) -> syn::Result<()> {
    use quote::ToTokens;

    const EXPECTED: &[&str] = &[
        "FromStr",
        "std :: str :: FromStr",
        ":: std :: str :: FromStr",
    ];

    let fromstrpath = itemimpl.get_path()?;
    let span = fromstrpath.span();
    let path = fromstrpath.to_token_stream().to_string();
    if EXPECTED.iter().any(|s| s == &path) {
        Ok(())
    } else {
        error_res(
            span,
            format!("expecting impl for one of {EXPECTED:?}, found {path:?}"),
        )
    }
}

fn prefix_impl_lifetime(def_generics: &syn::Generics) -> syn::Generics {
    use syn::punctuated::Punctuated;

    let mut impl_params = Punctuated::new();

    impl_params.push_value({
        use syn::{GenericParam, Lifetime, LifetimeParam};

        GenericParam::Lifetime(LifetimeParam {
            attrs: vec![],
            lifetime: Lifetime::new("'tryfrom_str_lifetime", def_generics.span()),
            colon_token: None,
            bounds: Punctuated::default(),
        })
    });

    for def_param in &def_generics.params {
        use syn::token::Comma;

        impl_params.push_punct(Comma::default());
        impl_params.push_value(def_param.clone());
    }

    syn::Generics {
        lt_token: def_generics.lt_token,
        params: impl_params,
        gt_token: def_generics.gt_token,
        where_clause: None,
    }
}

#[cfg(test)]
mod tests;
