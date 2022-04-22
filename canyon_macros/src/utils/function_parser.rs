use syn::{parse::{Parse, ParseBuffer}, ItemFn, Attribute, Visibility, Signature, Block};

/// Implementation of syn::Parse for the `#[canyon]` proc-macro
#[derive(Clone)]
pub struct FunctionParser {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub sig: Signature,
    pub block: Box<Block>
}

impl Parse for FunctionParser {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let func = input.parse::<ItemFn>();

        if func.is_err() {
            return Err(
                syn::Error::new(
                    input.cursor().span(), "Error on `fn main()`"
                )
            )
        }

        let func_ok = func.ok().unwrap();
        Ok(
            Self {
                attrs: func_ok.attrs,
                vis: func_ok.vis,
                sig: func_ok.sig,
                block: func_ok.block
            }
        )
        
    }
}