use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Type, parse_quote};

pub struct MacroOperationBuilder {
    fn_name: Option<Ident>,
    lifetime: bool, // bool true always will generate <'a>
    datasource_param: Option<TokenStream>,
    datasource_arg: TokenStream,
    return_type: Option<Ident>,
    base_doc_comment: Option<String>,
    doc_comment: Option<String>,
    body_tokens: Option<TokenStream>,
    query_string: Option<String>,
    input_parameters: Option<TokenStream>,
    forwarded_parameters: Option<TokenStream>,
    single_result: bool,
    with_unwrap: bool,
}

impl MacroOperationBuilder {
    pub fn new() -> Self {
        Self {
            fn_name: None,
            lifetime: false,
            datasource_param: None,
            datasource_arg: quote! { "" },
            return_type: None,
            base_doc_comment: None,
            doc_comment: None,
            body_tokens: None,
            query_string: None,
            input_parameters: None,
            forwarded_parameters: None,
            single_result: false,
            with_unwrap: false,
        }
    }

    fn get_fn_name(&self) -> TokenStream {
        if let Some(fn_name) = &self.fn_name {
            quote!{ #fn_name }
        } else {
            panic!("No function name provided")
        }
    }

    pub fn fn_name(mut self, name: &str) -> Self {
        self.fn_name = Some(Ident::new(name, Span::call_site()));
        self
    }

    fn get_lifetime(&self) -> TokenStream {
        if self.lifetime { quote!{ <'a> } } else { quote!{} }
    }

    fn get_datasource_param(&self) -> TokenStream {
        let ds_param = &self.datasource_param;
        quote! { #ds_param }
    }

    fn get_datasource_arg(&self) -> &TokenStream {
        &self.datasource_arg
    }

    pub fn with_datasource_param(mut self) -> Self {
        self.datasource_param = Some(quote! { datasource_name: &'a str });
        self.datasource_arg = quote! { datasource_name };
        self.lifetime = true;
        self
    }

    fn get_return_type(&self) -> TokenStream {
        let organic_ret_type = &self.return_type;
        let container_ret_type = if self.single_result {
            quote! { Option }
        } else { quote! { Vec } };

        match &self.with_unwrap { // TODO: distinguish collection from 1 results
            true => quote! { #container_ret_type<#organic_ret_type> },
            false => {
                let err_variant = if self.lifetime {
                    quote! { Box<(dyn std::error::Error + Send + Sync + 'a)> }
                } else {
                    quote! { Box<(dyn std::error::Error + Send + Sync)>}
                };

                quote! { Result<#container_ret_type<#organic_ret_type>, #err_variant> }
            }
        }
    }

    pub fn return_type(mut self, return_type: &Ident) -> Self {
        self.return_type = Some(return_type.clone());
        self
    }

    pub fn single_result(mut self) -> Self {
        self.single_result = true;
        self
    }

    pub fn base_doc_comment(mut self, comment: &str) -> Self {
        self.base_doc_comment = Some(comment.to_string());
        self
    }

    pub fn doc_comment(mut self, comment: &str) -> Self {
        self.doc_comment = Some(comment.to_string());
        self
    }

    pub fn query_string(mut self, query: &str) -> Self {
        self.query_string = Some(query.to_string());
        self
    }

    pub fn input_parameters(mut self, params: TokenStream) -> Self {
        self.input_parameters = Some(params);
        self
    }

    pub fn get_fn_parameters(&self) -> TokenStream {
        let func_parameters = &self.input_parameters;
        quote! { #func_parameters }
    }

    pub fn forwarded_parameters(mut self, params: TokenStream) -> Self {
        self.forwarded_parameters = Some(params);
        self
    }

    pub fn get_forwarded_parameters(&self) -> TokenStream {
        let forwarded_parameters = &self.forwarded_parameters;

        if let Some(fwd_params) = &self.forwarded_parameters {
            quote! { #forwarded_parameters }
        } else { quote!{ &[] } }
    }

    pub fn with_unwrap(mut self, value: bool) -> Self {
        self.with_unwrap = value;
        self
    }


    /// Generates the final `quote!` tokens for this operation
    pub fn generate_tokens(&self) -> proc_macro2::TokenStream {
        let base_doc_comment = &self.base_doc_comment;
        let doc_comment = &self.doc_comment;
        
        let fn_name = self.get_fn_name();
        let lifetime = self.get_lifetime();

        let datasource_param = self.get_datasource_param();
        let datasource_name = self.get_datasource_arg();
        let fn_parameters = self.get_fn_parameters();

        let query_string = &self.query_string;
        let forwarded_parameters = self.get_forwarded_parameters();
        let return_type = self.get_return_type();
        
        let unwrap_tokens = if self.with_unwrap {
            quote! { .unwrap() }
        } else {
            quote! {}
        };

        let body_tokens = quote!{
            <User as canyon_sql::core::Transaction<User>>::query(
                #query_string,
                #forwarded_parameters,
                #datasource_name
            ).await
            .into_results::<User>()
        };

        let separate_params = if self.input_parameters.is_some() && self.datasource_param.is_some() {
            quote! {, }
        } else { quote! {} };

        quote! {
            #[doc = #base_doc_comment]
            #[doc = #doc_comment]
            async fn #fn_name #lifetime(#fn_parameters #separate_params #datasource_param) -> #return_type {
                #body_tokens
                #unwrap_tokens
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;
    
    #[test]
    fn test_find_operation_tokens() {
        let ret_type = Ident::new("User", Span::call_site());

        let find_operation = MacroOperationBuilder::new()
            .fn_name("find_user_by_id")
            .with_datasource_param()
            .return_type(&ret_type)
            .base_doc_comment("Finds a user by their ID.")
            .doc_comment("This operation retrieves a single user record based on the provided ID.")
            .query_string("SELECT * FROM users WHERE id = ?")
            .input_parameters(quote! { id: &dyn QueryParameters<'_> })
            .forwarded_parameters(quote!{ &[id] })
            .single_result()
            .with_unwrap(false);

        let generated_tokens = find_operation.generate_tokens();
        let expected_tokens = quote! {
            #[doc = "Finds a user by their ID."]
            #[doc = "This operation retrieves a single user record based on the provided ID."]
            async fn find_user_by_id<'a>(id: &dyn QueryParameters<'_>, datasource_name: &'a str) -> Result<Option<User>, Box<(dyn std::error::Error + Send + Sync + 'a)> > {
                <User as canyon_sql::core::Transaction<User>>::query(
                    "SELECT * FROM users WHERE id = ?",
                    &[id],
                    datasource_name
                ).await
                .into_results::<User>()
            }
        };

        assert_eq!(
            generated_tokens.to_string(),
            expected_tokens.to_string()
        );
    }

    #[test]
    fn test_find_all_operation_tokens() {
        let ret_type = Ident::new("User", Span::call_site());

        let find_operation = MacroOperationBuilder::new()
            .fn_name("find_all")
            .return_type(&ret_type)
            .base_doc_comment("Executes a 'SELECT * FROM <user_type>'")
            .doc_comment("This operation retrieves all the users records stored with the default datasource")
            .query_string("SELECT * FROM users")
            .with_unwrap(false);

        let generated_tokens = find_operation.generate_tokens();
        let expected_tokens = quote! {
            #[doc = "Executes a 'SELECT * FROM <user_type>'"]
            #[doc = "This operation retrieves all the users records stored with the default datasource"]
            async fn find_all() -> Result<Vec<User>, Box<(dyn std::error::Error + Send + Sync)> > {
                <User as canyon_sql::core::Transaction<User>>::query(
                    "SELECT * FROM users",
                    &[],
                    ""
                ).await
                .into_results::<User>()
            }
        };

        assert_eq!(
            generated_tokens.to_string(),
            expected_tokens.to_string()
        );
    }

    #[test]
    fn test_find_all_datasource_operation_tokens() {
        let ret_type = Ident::new("User", Span::call_site());

        let find_operation = MacroOperationBuilder::new()
            .fn_name("find_all_datasource")
            .with_datasource_param()
            .return_type(&ret_type)
            .base_doc_comment("Executes a 'SELECT * FROM <user_type>'")
            .doc_comment("This operation retrieves all the users records stored in the provided datasource")
            .query_string("SELECT * FROM users")
            .with_unwrap(false);

        let generated_tokens = find_operation.generate_tokens();
        let expected_tokens = quote! {
            #[doc = "Executes a 'SELECT * FROM <user_type>'"]
            #[doc = "This operation retrieves all the users records stored in the provided datasource"]
            async fn find_all_datasource<'a>(datasource_name: &'a str) -> Result<Vec<User>, Box<(dyn std::error::Error + Send + Sync + 'a)> > {
                <User as canyon_sql::core::Transaction<User>>::query(
                    "SELECT * FROM users",
                    &[],
                    datasource_name
                ).await
                .into_results::<User>()
            }
        };

        assert_eq!(
            generated_tokens.to_string(),
            expected_tokens.to_string()
        );
    }

    // #[test]
    // fn test_find_operation_tokens() {
    //     // Arrange: Build the operation
    //     let find_operation = MacroOperationBuilder::new()
    //         .fn_name("find_user_by_id")
    //         .datasource_param(parse_quote!(datasource))
    //         .datasource_arg(quote! { datasource_arg })
    //         .return_type(parse_quote!(Result<User, Error>))
    //         .base_doc_comment("Finds a user by their ID.")
    //         .doc_comment("This operation retrieves a single user record based on the provided ID.")
    //         .query_string("SELECT * FROM users WHERE id = ?")
    //         .input_parameters(quote! { &[id] })
    //         // .parameterized(true)
    //         .with_unwrap(false);

    //     // Act: Generate tokens
    //     let generated_tokens = find_operation.generate_tokens();

    //     // Assert: Compare against expected tokens
    //     let expected_tokens = quote! {
    //         #[doc = "Finds a user by their ID."]
    //         #[doc = "This operation retrieves a single user record based on the provided ID."]
    //         async fn find_user_by_id(datasource) -> Result<User, Error> {
    //             <User as canyon_sql::core::Transaction<User>>::query(
    //                 "SELECT * FROM users WHERE id = ?",
    //                 &[id],
    //                 datasource_arg
    //             ).await
    //             .into_results::<User>()
    //         }
    //     };

    //     assert_eq!(
    //         generated_tokens.to_string(),
    //         expected_tokens.to_string(),
    //         "Generated tokens do not match expected tokens!"
    //     );
    // }

    // #[test]
    // fn test_insert_operation_tokens() {
    //     // Arrange: Build the operation
    //     let insert_operation = MacroOperationBuilder::new()
    //         .fn_name("insert_user")
    //         .datasource_param(parse_quote!(datasource))
    //         .datasource_arg(quote! { datasource_arg })
    //         .return_type(parse_quote!(Result<(), Error>))
    //         .base_doc_comment("Inserts a new user into the database.")
    //         .doc_comment("This operation inserts a new user record with the provided data.")
    //         .query_string("INSERT INTO users (name, email) VALUES (?, ?)")
    //         .input_parameters(quote! { &dyn QueryParameters })
    //         // .parameterized(true)
    //         .with_unwrap(false);

    //     // Act: Generate tokens
    //     let generated_tokens = insert_operation.generate_tokens();

    //     // Assert: Compare against expected tokens
    //     let expected_tokens = quote! {
    //         #[doc = "Inserts a new user into the database."]
    //         #[doc = "This operation inserts a new user record with the provided data."]
    //         async fn insert_user(datasource) -> Result<(), Error> {
    //             <User as canyon_sql::core::Transaction<User>>::query(
    //                 "INSERT INTO users (name, email) VALUES (?, ?)",
    //                 &dyn QueryParameters,
    //                 datasource_arg
    //             ).await
    //             .into_results::<User>()
    //         }
    //     };

    //     assert_eq!(
    //         generated_tokens.to_string(),
    //         expected_tokens.to_string(),
    //         "Generated tokens do not match expected tokens!"
    //     );
    // }
}
