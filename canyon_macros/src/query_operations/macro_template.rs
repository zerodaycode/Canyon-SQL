use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, Type};

pub struct MacroOperationBuilder {
    fn_name: Option<Ident>,
    user_type: Option<Ident>,
    lifetime: bool, // bool true always will generate <'a>
    input_param: Option<TokenStream>,
    input_fwd_arg: Option<TokenStream>,
    return_type: Option<Ident>,
    where_clause_bounds: Vec<TokenStream>,
    doc_comments: Vec<String>,
    body_tokens: Option<TokenStream>,
    query_string: Option<String>,
    input_parameters: Option<TokenStream>,
    forwarded_parameters: Option<TokenStream>,
    single_result: bool,
    with_unwrap: bool,
    transaction_as_variable: bool,
    disable_mapping: bool,
    raw_return: bool,
    propagate_transaction_result: bool,
    post_body: Option<TokenStream>,
}

impl ToTokens for MacroOperationBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate_tokens());
    }
}

impl MacroOperationBuilder {
    pub const fn new() -> Self {
        Self {
            fn_name: None,
            user_type: None,
            lifetime: false,
            input_param: None,
            input_fwd_arg: None,
            return_type: None,
            where_clause_bounds: Vec::new(),
            doc_comments: Vec::new(),
            body_tokens: None,
            query_string: None,
            input_parameters: None,
            forwarded_parameters: None,
            single_result: false,
            with_unwrap: false,
            transaction_as_variable: false,
            disable_mapping: false,
            raw_return: false,
            propagate_transaction_result: false,
            post_body: None,
        }
    }

    fn get_fn_name(&self) -> TokenStream {
        if let Some(fn_name) = &self.fn_name {
            quote! { #fn_name }
        } else {
            panic!("No function name provided")
        }
    }

    pub fn fn_name(mut self, name: &str) -> Self {
        self.fn_name = Some(Ident::new(name, Span::call_site()));
        self
    }

    fn get_user_type(&self) -> TokenStream {
        if let Some(user_type) = &self.user_type {
            quote! { #user_type }
        } else {
            panic!("No T type provided for determining the operations implementor")
        }
    }

    pub fn user_type(mut self, ty: &Ident) -> Self {
        self.user_type = Some(ty.clone());
        self
    }

    fn get_lifetime(&self) -> TokenStream {
        if self.lifetime {
            quote! { <'a> }
        } else {
            quote! {}
        }
    }

    fn get_input_param(&self) -> TokenStream {
        let input_param = &self.input_param;
        quote! { #input_param }
    }

    fn get_input_arg(&self) -> TokenStream {
        if let Some(input_arg) = &self.input_fwd_arg {
            let ds_arg0 = input_arg;
            quote! { #ds_arg0 }
        } else {
            quote! { "" }
        }
    }

    pub fn with_lifetime(mut self) -> Self {
        self.lifetime = true;
        self
    }

    pub fn with_input_param(mut self) -> Self {
        self.input_param = Some(quote! { input: I });
        self.input_fwd_arg = Some(quote! { input });
        self.lifetime = true;
        self.where_clause_bounds
            .push(quote! { I: Into<TransactionInput<'a>> + Sync + Send + 'a });
        self
    }

    fn get_return_type(&self) -> TokenStream {
        let organic_ret_type = &self.return_type;
        let container_ret_type = if self.single_result {
            quote! { Option }
        } else {
            quote! { Vec }
        };

        let ret_type = if self.raw_return {
            quote! { #organic_ret_type }
        } else {
            quote! { #container_ret_type<#organic_ret_type> }
        };

        match &self.with_unwrap {
            // TODO: distinguish collection from 1 results
            true => quote! { #ret_type },
            false => {
                let err_variant = if self.lifetime {
                    quote! { Box<(dyn std::error::Error + Send + Sync + 'a)> }
                } else {
                    quote! { Box<(dyn std::error::Error + Send + Sync)>}
                };

                quote! { Result<#ret_type, #err_variant> }
            }
        }
    }

    fn get_where_clause_bounds(&self) -> TokenStream {
        if self.where_clause_bounds.is_empty() {
            quote! {}
        } else {
            let where_bounds = &self.where_clause_bounds;
            quote! {
                where #(#where_bounds),*
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

    pub fn add_doc_comment(mut self, comment: &str) -> Self {
        self.doc_comments.push(comment.to_string());
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

    fn get_forwarded_parameters(&self) -> TokenStream {
        let forwarded_parameters = &self.forwarded_parameters;

        if let Some(fwd_params) = &self.forwarded_parameters {
            quote! { #forwarded_parameters }
        } else {
            quote! { &[] }
        }
    }

    fn get_unwrap(&self) -> TokenStream {
        if self.with_unwrap {
            quote! { .unwrap() }
        } else {
            quote! {}
        }
    }

    pub fn with_unwrap(mut self) -> Self {
        self.with_unwrap = true;
        self
    }

    pub fn transaction_as_variable(mut self, result_handling: TokenStream) -> Self {
        self.transaction_as_variable = true;
        self.post_body = Some(result_handling);
        self
    }

    pub fn disable_mapping(mut self) -> Self {
        self.disable_mapping = true;
        self
    }

    pub fn raw_return(mut self) -> Self {
        self.raw_return = true;
        self
    }

    pub fn propagate_transaction_result(mut self) -> Self {
        self.propagate_transaction_result = true;
        self
    }

    /// Generates the final `quote!` tokens for this operation
    pub fn generate_tokens(&self) -> proc_macro2::TokenStream {
        let doc_comments = &self
            .doc_comments
            .iter()
            .map(|doc_comment| quote! { #[doc = #doc_comment] })
            .collect::<Vec<_>>();

        let ty = self.get_user_type();
        let fn_name = self.get_fn_name();
        let lifetime = self.get_lifetime(); // TODO: generics instead

        let input_param = self.get_input_param();
        let input_fwd_arg = self.get_input_arg(); // TODO: replace
        let fn_parameters = self.get_fn_parameters();

        let query_string = &self.query_string;
        let forwarded_parameters = self.get_forwarded_parameters();
        let return_type = self.get_return_type();
        let where_clause = self.get_where_clause_bounds();
        let unwrap = self.get_unwrap();

        let mut base_body_tokens = quote! {
            <#ty as canyon_sql::core::Transaction<#ty>>::query(
                #query_string,
                #forwarded_parameters,
                #input_fwd_arg
            ).await
        };

        if self.propagate_transaction_result {
            base_body_tokens.extend(quote! { ? })
        };
        if !self.disable_mapping {
            base_body_tokens.extend(quote! { .into_results::<#ty>() })
        };

        let body_tokens = if self.transaction_as_variable {
            let result_handling = &self.post_body;
            quote! {
                let transaction_result = #base_body_tokens;
                #result_handling
            }
        } else {
            base_body_tokens
        };

        let separate_params = if self.input_parameters.is_some() && self.input_param.is_some() // TODO:
                                                                                               // change
                                                                                               // for
                                                                                               // getter?
        {
            quote! {, }
        } else {
            quote! {}
        };

        quote! {
            #(#doc_comments)*
            async fn #fn_name #lifetime(#fn_parameters #separate_params #input_param) -> #return_type
                #where_clause
            {
                #body_tokens
                #unwrap
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
        let user_type = Ident::new("User", Span::call_site());

        let find_operation = MacroOperationBuilder::new()
            .fn_name("find_user_by_id")
            .user_type(&user_type)
            .with_input_param()
            .return_type(&user_type)
            .add_doc_comment("Finds a user by their ID.")
            .add_doc_comment(
                "This operation retrieves a single user record based on the provided ID.",
            )
            .query_string("SELECT * FROM users WHERE id = ?")
            .input_parameters(quote! { id: &dyn QueryParameters<'_> })
            .forwarded_parameters(quote! { &[id] })
            .single_result();

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

        assert_eq!(generated_tokens.to_string(), expected_tokens.to_string());
    }

    #[test]
    fn test_find_all_operation_tokens() {
        let user_type = Ident::new("User", Span::call_site());

        let find_operation = MacroOperationBuilder::new()
            .fn_name("find_all")
            .user_type(&user_type)
            .return_type(&user_type)
            .add_doc_comment("Executes a 'SELECT * FROM <user_type>'")
            .add_doc_comment(
                "This operation retrieves all the users records stored with the default datasource",
            )
            .query_string("SELECT * FROM users");

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

        assert_eq!(generated_tokens.to_string(), expected_tokens.to_string());
    }

    #[test]
    fn test_find_all_datasource_operation_tokens() {
        let user_type = Ident::new("User", Span::call_site());

        let find_operation = MacroOperationBuilder::new()
            .fn_name("find_all_datasource")
            .user_type(&user_type)
            .with_input_param()
            .return_type(&user_type)
            .add_doc_comment("Executes a 'SELECT * FROM <user_type>'")
            .add_doc_comment(
                "This operation retrieves all the users records stored in the provided datasource",
            )
            .query_string("SELECT * FROM users");

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

        assert_eq!(generated_tokens.to_string(), expected_tokens.to_string());
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
