use indoc::indoc;
use proc_macro2::TokenStream;
use test_case::test_case;

#[test_case(
    indoc! {r#"
        impl std::str::FromStr for MyType {}
    "#},
    indoc! {r#"
        impl std::str::FromStr for MyType {}
        impl<'tryfrom_str_lifetime> ::std::convert::TryFrom<&'tryfrom_str_lifetime str>
        for MyType {
            type Error = <Self as ::std::str::FromStr>::Err;
            fn try_from(s: &'tryfrom_str_lifetime str) -> Result<Self, Self::Error> {
                use ::std::str::FromStr;

                Self::from_str(s)
            }
        }
    "#}
    ; "basic"
)]
#[test_case(
    indoc! {r#"
        impl<T> FromStr for UnconstrainedWrapper<T> {}
    "#},
    indoc! {r#"
        impl<T> FromStr for UnconstrainedWrapper<T> {}
        impl<'tryfrom_str_lifetime, T> ::std::convert::TryFrom<&'tryfrom_str_lifetime str>
        for UnconstrainedWrapper<T> {
            type Error = <Self as ::std::str::FromStr>::Err;
            fn try_from(s: &'tryfrom_str_lifetime str) -> Result<Self, Self::Error> {
                use ::std::str::FromStr;

                Self::from_str(s)
            }
        }
    "#}
    ; "generic-type-unconstrained"
)]
#[test_case(
    indoc! {r#"
        impl<T> FromStr for ConstrainedWrapper<T> where T: FromStr {}
    "#},
    indoc! {r#"
        impl<T> FromStr for ConstrainedWrapper<T>
        where
            T: FromStr,
        {}
        impl<'tryfrom_str_lifetime, T> ::std::convert::TryFrom<&'tryfrom_str_lifetime str>
        for ConstrainedWrapper<T>
        where
            T: FromStr,
        {
            type Error = <Self as ::std::str::FromStr>::Err;
            fn try_from(s: &'tryfrom_str_lifetime str) -> Result<Self, Self::Error> {
                use ::std::str::FromStr;

                Self::from_str(s)
            }
        }
    "#}
    ; "generic-type-constrained"
)]
fn transform(input: &str, expected: &str) {
    let input = input.trim();
    eprintln!("For input:\n{}", quote_code(input));
    match transform_res(input, expected) {
        Ok((found, expected)) => assert_eq!(
            &expected,
            &found,
            "\n\nexpected:\n{}\n\nfound:\n{}",
            quote_code(&expected),
            quote_code(&found),
        ),
        Err(Error::Lex(e)) => panic!("lex error: {e}"),
        Err(Error::Syn(e, src)) => {
            panic!("syn error {e}\n-while parsing:\n{}", quote_code(&src))
        }
    }
}

fn quote_code(s: &str) -> String {
    format!("| {}", s.replace('\n', "\n| "))
}

#[derive(Debug, derive_more::From)]
enum Error {
    Lex(proc_macro2::LexError),
    Syn(syn::Error, String),
}

type Result<T> = std::result::Result<T, Error>;

fn transform_res(input: &str, expected: &str) -> Result<(String, String)> {
    use quote::quote;

    let input: TokenStream = input.parse()?;
    let inputstr = input.to_string();
    let output = crate::transform(quote! {}, input).map_err(|e| Error::Syn(e, inputstr))?;
    let output_pretty = unparse(output)?;
    let expected_pretty = prettify(expected)?;
    Ok((output_pretty, expected_pretty))
}

fn prettify(src: &str) -> Result<String> {
    let ts: TokenStream = src.parse()?;
    let pretty = unparse(ts)?;
    Ok(pretty)
}

fn unparse(itemstream: TokenStream) -> Result<String> {
    let f: syn::File = parse(itemstream)?;
    Ok(prettyplease::unparse(&f))
}

fn parse<T>(stream: TokenStream) -> Result<T>
where
    T: syn::parse::Parse,
{
    let streamstr = stream.to_string();
    syn::parse2(stream).map_err(|e| Error::Syn(e, streamstr))
}
