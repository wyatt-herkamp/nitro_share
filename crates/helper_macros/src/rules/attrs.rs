use syn::{
    parse::{Parse, ParseStream},
    Path,
};

mod keywords {
    use syn::custom_keyword;
    custom_keyword!(serialize_with);
    custom_keyword!(serialize_with_option);
}
/// A struct field attribute that controls how serialization should be performed.
///
/// Value One: Path to a function that follows the signature of `fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer`
/// Value Two: Whether the field is an Option
pub type SerializeWith = (Path, bool);
#[derive(Debug, Default)]
pub struct RulesAttr {
    pub serialize_with: Option<SerializeWith>,
}
impl Parse for RulesAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut serialize_with = None;
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(keywords::serialize_with) {
                let _ = input.parse::<keywords::serialize_with>()?;
                let _: syn::Token![=] = input.parse()?;
                serialize_with = Some((input.parse()?, false));
            } else if lookahead.peek(keywords::serialize_with_option) {
                let _ = input.parse::<keywords::serialize_with_option>()?;
                let _: syn::Token![=] = input.parse()?;
                serialize_with = Some((input.parse()?, true));
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(RulesAttr { serialize_with })
    }
}
