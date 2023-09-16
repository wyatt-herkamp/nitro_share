mod response_type;
mod rules;
mod serde_as_rules;

use proc_macro::{Span, TokenStream};
use syn::{parse_macro_input, DeriveInput, ItemStruct};

#[proc_macro_derive(Rules, attributes(rule))]
pub fn rules(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // Check if its an enum
    if let syn::Data::Struct(_) = &input.data {
        match rules::expand(input) {
            Ok(ok) => ok.into(),
            Err(err) => err.to_compile_error().into(),
        }
    } else {
        syn::Error::new_spanned(input, "rules can only be derived for structs")
            .to_compile_error()
            .into()
    }
}
#[proc_macro_derive(
    Response,
    attributes(expires, last_modified, refresh_duration, private, public)
)]
pub fn response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // Check if its an enum
    if let syn::Data::Struct(_) = &input.data {
        match response_type::expand(input) {
            Ok(ok) => ok.into(),
            Err(err) => err.to_compile_error().into(),
        }
    } else {
        syn::Error::new_spanned(input, "Response can only be derived for structs")
            .to_compile_error()
            .into()
    }
}
#[proc_macro_attribute]
pub fn serde_as_rules(_: TokenStream, input: TokenStream) -> TokenStream {
    if let Ok(input) = syn::parse::<ItemStruct>(input.clone()) {
        match serde_as_rules::rewrite(input.clone()) {
            Ok(ok) => ok.into(),
            Err(err) => err.to_compile_error().into(),
        }
    } else {
        let span = Span::call_site();
        syn::Error::new(span.into(), "serde_as_rules can only be applied to structs")
            .to_compile_error()
            .into()
    }
}
