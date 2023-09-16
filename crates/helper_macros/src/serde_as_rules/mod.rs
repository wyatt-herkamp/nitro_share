use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, ItemStruct, Result};

/// Rewrites all struct fields with the `#[serde_as_rules]` to `[serde(serialized_with = "<#Type as common::rules::Rule>::serialize")]`
pub(crate) fn rewrite(mut item_struct: ItemStruct) -> Result<TokenStream> {
    match &mut item_struct.fields {
        syn::Fields::Named(fields) => {
            fields.named.iter_mut().for_each(|field| {
                let attrs = field
                    .attrs
                    .iter_mut()
                    .find(|attr| attr.path().is_ident("serde_as_rules"));
                if let Some(attrs) = attrs {
                    let ty = &field.ty;
                    let serialize_with_value = quote! {
                        <#ty as common::rules::Rules>::serialize
                    }
                    .to_string();
                    let value = syn::parse_quote!(#[serde(serialize_with = #serialize_with_value)]);
                    *attrs = value;
                }
            });
        }
        _ => {
            return Err(Error::new_spanned(
                item_struct,
                "serde_as_rules can only be applied to structs",
            ))
        }
    };

    Ok(quote! {
        #item_struct
    })
}
