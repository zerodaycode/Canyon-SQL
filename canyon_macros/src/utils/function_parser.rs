use syn::{parse::{Parse, ParseBuffer}, Attribute, Block, ItemFn, Signature, Visibility};

/// Implementation of syn::Parse for the `#[canyon]` proc-macro
#[derive(Clone)]
pub struct FunctionParser {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub sig: Signature,
    pub block: Box<Block>,
}

impl Parse for FunctionParser {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let func = input.parse::<ItemFn>()?;

        Ok(Self {
            attrs: func.attrs,
            vis: func.vis,
            sig: func.sig,
            block: func.block,
        })
    }
}
