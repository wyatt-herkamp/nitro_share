mod attrs;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

use crate::response_type::attrs::ResponseAttr;

pub(crate) fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let struct_name = derive_input.ident.clone();
    let attr = ResponseAttr::new(&derive_input.attrs)?;
    Ok(quote! {
        const _ : () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate chrono as _chrono;
        extern crate common as _common;

        #[automatically_derived]
        impl _common::response_type::ResponseType for #struct_name {
            #attr
        }
        };
    })
}
