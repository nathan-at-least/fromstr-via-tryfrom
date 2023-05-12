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
                s.parse()
            }
        }
    "#}
    ; "basic"
)]
#[test_case(
    indoc! {r#"
        impl<T> FromStr for MyWrapper<T> {}
    "#},
    indoc! {r#"
        impl<T> FromStr for MyWrapper<T> {}

        impl<'tryfrom_str_lifetime, T> ::std::convert::TryFrom<&'tryfrom_str_lifetime str>
        for Wrapper<T>
        {
            type Error = <Self as ::std::str::FromStr>::Err;
            fn try_from(s: &'tryfrom_str_lifetime str) -> Result<Self, Self::Error> {
                s.parse()
            }
        }
    "#}
    ; "generic-type-unconstrained"
)]
fn transform(input: &str, expected: &str) {
    let input = input.trim();
    println!("For input:\n{}", quote_code(input));
    match transform_res(input) {
        Ok(found) => assert_eq!(
            expected,
            &found,
            "\n\nexpected:\n{}\n\nfound:\n{}",
            quote_code(expected),
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

fn transform_res(src: &str) -> Result<String> {
    use quote::quote;

    let input: TokenStream = src.parse()?;
    let inputstr = input.to_string();
    let output = crate::transform(quote! {}, input).map_err(|e| Error::Syn(e, inputstr))?;
    let pretty = unparse(output)?;
    Ok(pretty)
}

fn unparse(itemstream: TokenStream) -> Result<String> {
    let item: syn::Item = parse(itemstream)?;
    Ok(prettyplease::unparse(&syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![item],
    }))
}

fn parse<T>(stream: TokenStream) -> Result<T>
where
    T: syn::parse::Parse,
{
    let streamstr = stream.to_string();
    syn::parse2(stream).map_err(|e| Error::Syn(e, streamstr))
}
