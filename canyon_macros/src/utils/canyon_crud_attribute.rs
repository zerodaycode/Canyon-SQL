use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::Token;

/// Type that helps to parse the: `#[canyon_crud(maps_to = Ident)]` proc macro attribute
///
/// The ident value of the `maps_to` argument brings a type that is the target type for which
/// `CrudOperations` will write the queries as the implementor of [`RowMapper`]
pub(crate) struct CanyonCrudAttribute {
    pub maps_to: Option<Ident>,
}

impl Parse for CanyonCrudAttribute {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let arg_name: Ident = input.parse()?;
        if arg_name != "maps_to" {
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
