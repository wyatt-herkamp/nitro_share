use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Attribute, Expr, LitInt,
};

mod keywords {
    use syn::custom_keyword;
    custom_keyword!(weeks);
    custom_keyword!(days);
    custom_keyword!(Some);
}
#[derive(Debug, PartialEq)]
pub enum Duration {
    Weeks(LitInt),
    Days(LitInt),
}
impl ToTokens for Duration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Duration::Weeks(weeks) => {
                tokens.extend(quote! {
                    _chrono::Duration::weeks(#weeks)
                });
            }
            Duration::Days(days) => {
                tokens.extend(quote! {
                    _chrono::Duration::days(#days)
                });
            }
        }
    }
}
impl Parse for Duration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keywords::weeks) {
            input.parse::<keywords::weeks>()?;
            let content;
            parenthesized!(content in input);
            return Ok(Duration::Weeks(content.parse::<LitInt>()?));
        } else if lookahead.peek(keywords::days) {
            input.parse::<keywords::days>()?;
            let content;
            parenthesized!(content in input);
            return Ok(Duration::Days(content.parse::<LitInt>()?));
        } else {
            Err(lookahead.error())
        }
    }
}
#[derive(Debug)]
pub struct MaybeWrapInOption<T> {
    pub inner: T,
    pub wrap_in_option: bool,
}
impl<T: ToTokens> ToTokens for MaybeWrapInOption<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            inner,
            wrap_in_option,
        } = self;
        if *wrap_in_option {
            tokens.extend(quote! {
                Some(#inner)
            });
        } else {
            tokens.extend(quote! {
                #inner
            });
        }
    }
}
impl<T: Parse> Parse for MaybeWrapInOption<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(keywords::Some) {
            let _: keywords::Some = input.parse()?;
            let content;
            parenthesized!(content in input);
            let inner = content.parse::<T>()?;
            Ok(Self {
                inner,
                wrap_in_option: true,
            })
        } else {
            Ok(Self {
                inner: input.parse::<T>()?,
                wrap_in_option: false,
            })
        }
    }
}
#[derive(Debug)]
pub struct ResponseAttr {
    // Path to variable within the struct
    pub last_modified: Option<MaybeWrapInOption<Expr>>,
    pub expires: Option<MaybeWrapInOption<Expr>>,
    pub refresh_duration: Option<MaybeWrapInOption<Duration>>,
    pub private_or_public: &'static str,
}

impl ResponseAttr {
    pub fn new(attrs: &Vec<Attribute>) -> syn::Result<Self> {
        let mut last_modified = None;
        let mut expires = None;
        let mut refresh_duration = None;
        let mut private_or_public = "public";

        for attr in attrs {
            if attr.path().is_ident("last_modified") {
                last_modified = Some(attr.parse_args::<MaybeWrapInOption<Expr>>()?);
            } else if attr.path().is_ident("expires") {
                expires = Some(attr.parse_args::<MaybeWrapInOption<Expr>>()?);
            } else if attr.path().is_ident("refresh_duration") {
                refresh_duration = Some(attr.parse_args::<MaybeWrapInOption<Duration>>()?);
            } else if attr.path().is_ident("private") {
                private_or_public = "private";
            } else if attr.path().is_ident("public") {
                private_or_public = "public";
            }
        }
        Ok(Self {
            last_modified,
            expires,
            refresh_duration,
            private_or_public,
        })
    }
}

impl ToTokens for ResponseAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            last_modified,
            expires,
            refresh_duration,
            private_or_public,
        } = self;
        if let Some(last_modified) = last_modified {
            tokens.extend(quote! {
                #[inline(always)]
                fn last_modified(&self) -> Option<_chrono::DateTime<_chrono::FixedOffset>> {
                    #last_modified
                }
            });
        }
        if let Some(expires) = expires {
            tokens.extend(quote! {
                #[inline(always)]
                fn expires(&self) -> Option<_chrono::DateTime<_chrono::FixedOffset>> {
                    #expires
                }
            });
        }
        let mut cache_control_params = vec![];
        match private_or_public {
            &"private" => {
                cache_control_params.push(quote! {
                    _common::response_type::CacheControlParams::Private
                });
            }
            &"public" => {
                cache_control_params.push(quote! {
                    _common::response_type::CacheControlParams::Public
                });
            }
            _ => unreachable!(),
        }
        if let Some(refresh_duration) = refresh_duration {
            cache_control_params.push(quote! {
                _common::response_type::CacheControlParams::MaxAge(#refresh_duration)
            });
        }
        tokens.extend(quote! {
             #[inline(always)]
             fn cache_control_params(&self) -> Vec<_common::response_type::CacheControlParams> {
                vec![
                    #(#cache_control_params),*
                ]
             }
        })
    }
}
