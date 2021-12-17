use chrono::{DateTime, Datelike, Local};
use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;

use syn::Lit;
use syn::{parse::Parse, Expr, ExprBinary, Token};

#[proc_macro]
pub fn store_coeff_nanopb(cfg: TokenStream) -> TokenStream {
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
        r##"{src}.{field}.map(|v| {{
            {destination} = v;
            {flag} = true;
        }})"##,
        src = s.ex.right.to_token_stream().to_string(),
        field = s.src_field.to_token_stream().to_string(),
        destination = s.ex.left.into_token_stream().to_string(),
        flag = s.flag_name.to_token_stream().to_string()
    )
    .parse()
    .unwrap()
}

#[proc_macro]
pub fn build_year(_: TokenStream) -> TokenStream {
    let local: DateTime<Local> = Local::now();
    let y = local.year() as u32;

    quote! {
        #y
    }
    .into()
}

#[proc_macro]
pub fn build_month(_: TokenStream) -> TokenStream {
    let local: DateTime<Local> = Local::now();
    let m = local.month();

    quote! {
        #m
    }
    .into()
}

#[proc_macro]
pub fn build_day(_: TokenStream) -> TokenStream {
    let local: DateTime<Local> = Local::now();
    let d = local.day();

    quote! {
        #d
    }
    .into()
}

#[proc_macro]
pub fn git_version(_: TokenStream) -> TokenStream {
    use git_version::git_version;

    static V: &str = git_version!(args = ["--always", "--abbrev=16"]);
    format!("0x{}_u64", V).parse().unwrap()
}

#[proc_macro]
pub fn c_str(s: TokenStream) -> TokenStream {
    let lit = syn::parse_macro_input!(s as Lit);
    if let Lit::Str(litstr) = lit {
        format!("\"{}\0\"", litstr.value()).parse().unwrap()
    } else {
        panic!("Not a string literal!")
    }
}
