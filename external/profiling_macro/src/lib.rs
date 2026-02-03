use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

/// Usage:
///   #[profile]
///   fn foo() {}
///
///   #[profile("db::query_users")]
///   fn bar() {}
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = input_fn.sig.ident.to_string();
    let label = if attr.is_empty() {
        LitStr::new(&fn_name, input_fn.sig.ident.span())
    } else {
        parse_macro_input!(attr as LitStr)
    };
    let block = input_fn.block;
    input_fn.block = Box::new(
        syn::parse2(quote!({
            #[cfg(feature = "profiling")]
            let _profile_guard = profiling::ProfileGuard::new(#label);

            #block
        }))
            .expect("profiling macro failed"),
    );

    TokenStream::from(quote!(#input_fn))
}