use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::Token;

// TODO: docs
pub(super) struct CanyonCrudAttribute {
    pub maps_to: Option<Ident>,
}

impl Parse for CanyonCrudAttribute {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // Parse the argument name
        let arg_name: Ident = input.parse()?;
        if arg_name != "maps_to" {
            // Same error as before when encountering an unsupported attribute
            return Err(syn::Error::new_spanned(
                arg_name,
                "unsupported 'canyon_crud' attribute, expected `maps_to`",
            ));
        }

        // Parse (and discard the span of) the `=` token
        let _: Token![=] = input.parse()?;

        // Parse the argument value
        let name = input.parse()?;

        Ok(Self {
            maps_to: Some(name),
        })
    }
}
