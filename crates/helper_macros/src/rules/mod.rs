mod attrs;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Error, Field, Meta, Result};

use crate::rules::attrs::{RulesAttr, SerializeWith};

pub(crate) fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let struct_name = derive_input.ident.clone();

    let fields = match derive_input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => fields.named,
        _ => {
            return Err(Error::new_spanned(
                derive_input,
                "rules can only be derived for structs",
            ))
        }
    };

    let mut rules = Vec::with_capacity(fields.len());
    for field in fields {
        let attrs = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("rule"))
            .map(|v| {
                return if let Meta::List(_) = &v.meta {
                    v.parse_args()
                } else {
                    Ok(RulesAttr::default())
                };
            })
            .transpose()?;

        if let Some(attrs) = attrs {
            let result = if let Some(rule) = attrs.serialize_with {
                serialize_with(field, rule, struct_name.clone())?
            } else {
                let field_name = field.ident.clone().ok_or(Error::new(
                    field.span(),
                    "rules can only be derived for structs",
                ))?;
                quote! {
                    _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, stringify!(#field_name), &self.#field_name)?;
                }
            };

            rules.push(result);
        }
    }
    let fields_len = rules.len();
    let result = quote! {
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        extern crate common as _common;

        #[automatically_derived]
        impl _common::rules::Rules for #struct_name {
            fn serialize<S: _serde::Serializer>(&self, __serializer: S) -> std::result::Result<S::Ok, S::Error> {
                let mut __serde_state = _serde::Serializer::serialize_struct(__serializer,stringify!(#struct_name), #fields_len)?;

                #(#rules)*

                _serde::ser::SerializeStruct::end(__serde_state)
            }

        }
    };

        };

    Ok(result)
}

fn serialize_with(
    field: Field,
    serialize_with: SerializeWith,
    struct_name: Ident,
) -> Result<TokenStream> {
    let field_name = field.ident.clone().ok_or(Error::new(
        field.span(),
        "rules can only be derived for structs",
    ))?;
    let (path, is_option) = serialize_with;
    let inner_serialize = if is_option {
        quote! {
            match self.value {
                Some(ref value) => #path(value, __s),
                None => __s.serialize_none()
            }
        }
    } else {
        quote! {
            #path(self.value, __s)
        }
    };
    let field_type = field.ty;
    let result = quote! {
        _serde::ser::SerializeStruct::serialize_field(&mut __serde_state, stringify!(#field_name), {
            #[doc(hidden)]
            struct __SerializeWith<'a> {
                value: &'a #field_type,
                phantom: std::marker::PhantomData<#struct_name>,
            }
            impl<'a> _serde::Serialize for __SerializeWith<'a> {
                fn serialize<__S>(&self, __s: __S) -> std::result::Result<__S::Ok, __S::Error> where __S: _serde::Serializer, {
                    #inner_serialize
                }
            }
            &__SerializeWith { value: &self.#field_name, phantom: std::marker::PhantomData::<#struct_name> }
        })?;
    };
    Ok(result)
}
