use proc_macro2::Ident;
use syn::Token;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

/// Type that helps to parse the: `#[canyon_crud(maps_to = Ident)]` proc macro attribute
///
/// The ident value of the `maps_to` argument brings a type that is the target type for which
/// `CrudOperations` will write the queries as the implementor of [`RowMapper`]
pub(crate) struct CanyonCrudAttribute {
    pub maps_to: Option<Ident>,
    pub pk: Option<Ident>,
    pub pk_type: Option<Ident>,
}

impl Parse for CanyonCrudAttribute {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut maps_to = None;
        let mut pk = None;
        let mut pk_type = None;

        let pairs = Punctuated::<MetaEntry, Token![,]>::parse_terminated(input)?;

        for pair in pairs {
            match pair.name.to_string().as_str() {
                "maps_to" => maps_to = Some(pair.value),
                "primary_key" => pk = Some(pair.value),
                "pk_type" => pk_type = Some(pair.value),
                other => {
                    return Err(syn::Error::new_spanned(
                        pair.name,
                        format!("Unsupported attribute key: `{}`", other),
                    ));
                }
            }
        }

        Ok(Self {
            maps_to,
            pk,
            pk_type,
        })
    }
}

struct MetaEntry {
    name: Ident,
    eq_token: Token![=],
    value: Ident,
}

impl Parse for MetaEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}
