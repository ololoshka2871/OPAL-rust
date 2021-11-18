use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, Expr, ExprBinary, Token};

#[proc_macro]
pub fn test_macro(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}

#[proc_macro]
pub fn store_coeff(cfg: TokenStream) -> TokenStream {
    #[derive(Debug)]
    struct Config {
        pub ex: ExprBinary,
        _separator1: Token![;],
        src_field: Expr,
        _separator2: Token![;],
        flag_name: Expr,
    }

    impl Parse for Config {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(Config {
                ex: input.parse()?,
                _separator1: input.parse()?,
                src_field: input.parse()?,
                _separator2: input.parse()?,
                flag_name: input.parse()?,
            })
        }
    }

    let s = syn::parse_macro_input!(cfg as Config);

    assert!(matches!(s.ex.op, syn::BinOp::Le(_)));

    format!(
        r##"if {src}.has_{condition_var} {{
            {destination} = {src}.{condition_var};
            {flag} = true;
        }}"##,
        src = s.ex.right.to_token_stream().to_string(),
        condition_var = s.src_field.to_token_stream().to_string(),
        destination = s.ex.left.into_token_stream().to_string(),
        flag = s.flag_name.to_token_stream().to_string()
    )
    .parse()
    .unwrap()
}
