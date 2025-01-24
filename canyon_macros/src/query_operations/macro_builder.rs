use quote::quote;
use syn::{Ident, Type};

/// Builder for constructing CRUD operation metadata
pub struct OperationBuilder {
    operation_name: Option<Ident>,
    fn_name: Option<Ident>,
    datasource_param: Option<Ident>,
    datasource_arg: Option<Ident>,
    return_type: Option<Type>,
    base_doc_comment: Option<String>,
    doc_comment: Option<String>,
    body_tokens: Option<proc_macro2::TokenStream>,
    with_unwrap: bool,
}

impl OperationBuilder {
    /// Creates a new builder instance
    pub fn new() -> Self {
        Self {
            operation_name: None,
            fn_name: None,
            datasource_param: None,
            datasource_arg: None,
            return_type: None,
            base_doc_comment: None,
            doc_comment: None,
            body_tokens: None,
            with_unwrap: false,
        }
    }

    /// Sets the name of the operation
    pub fn operation_name(mut self, name: &str) -> Self {
        self.operation_name = Some(syn::Ident::new(name, proc_macro2::Span::call_site()));
        self
    }

    /// Sets the function name
    pub fn fn_name(mut self, name: &str) -> Self {
        self.fn_name = Some(syn::Ident::new(name, proc_macro2::Span::call_site()));
        self
    }

    /// Sets the datasource parameter
    pub fn datasource_param(mut self, param: &str) -> Self {
        self.datasource_param = Some(syn::Ident::new(param, proc_macro2::Span::call_site()));
        self
    }

    /// Sets the datasource argument
    pub fn datasource_arg(mut self, arg: &str) -> Self {
        self.datasource_arg = Some(syn::Ident::new(arg, proc_macro2::Span::call_site()));
        self
    }

    /// Sets the return type
    pub fn return_type(mut self, ty: Type) -> Self {
        self.return_type = Some(ty);
        self
    }

    /// Adds a base doc comment
    pub fn base_doc_comment(mut self, comment: &str) -> Self {
        self.base_doc_comment = Some(comment.to_string());
        self
    }

    /// Adds an additional doc comment
    pub fn doc_comment(mut self, comment: &str) -> Self {
        self.doc_comment = Some(comment.to_string());
        self
    }

    /// Sets the body of the function
    pub fn body_tokens(mut self, tokens: proc_macro2::TokenStream) -> Self {
        self.body_tokens = Some(tokens);
        self
    }

    /// Configures whether to use `.unwrap()`
    pub fn with_unwrap(mut self, unwrap: bool) -> Self {
        self.with_unwrap = unwrap;
        self
    }

    /// Finalizes the builder and returns the operation
    pub fn build(self) -> Operation {
        Operation {
            operation_name: self.operation_name.unwrap(),
            fn_name: self.fn_name.unwrap(),
            datasource_param: self.datasource_param.unwrap(),
            datasource_arg: self.datasource_arg.unwrap(),
            return_type: self.return_type.unwrap(),
            base_doc_comment: self.base_doc_comment.unwrap(),
            doc_comment: self.doc_comment.unwrap(),
            body_tokens: self.body_tokens.unwrap(),
            with_unwrap: self.with_unwrap,
        }
    }
}

/// Represents a fully constructed CRUD operation
pub struct Operation {
    pub operation_name: Ident,
    pub fn_name: Ident,
    pub datasource_param: Ident,
    pub datasource_arg: Ident,
    pub return_type: Type,
    pub base_doc_comment: String,
    pub doc_comment: String,
    pub body_tokens: proc_macro2::TokenStream,
    pub with_unwrap: bool,
}

impl Operation {
    /// Generates the final `quote!` tokens for this operation
    pub fn generate_tokens(&self) -> proc_macro2::TokenStream {
        let base_doc_comment = &self.base_doc_comment;
        let doc_comment = &self.doc_comment;
        let fn_name = &self.fn_name;
        let datasource_param = &self.datasource_param;
        let return_type = &self.return_type;
        let body_tokens = &self.body_tokens;
        let unwrap_tokens = if self.with_unwrap {
            quote! { .unwrap() }
        } else {
            quote! {}
        };

        quote! {
            #[doc = #base_doc_comment]
            #[doc = #doc_comment]
            async fn #fn_name(#datasource_param) -> #return_type {
                #body_tokens
                #unwrap_tokens
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import your structs and builder
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_find_operation_tokens() {
        // Arrange: Build the operation
        let find_operation = OperationBuilder::new()
            .operation_name("find")
            .fn_name("find_user_by_id")
            .datasource_param(parse_quote!(datasource))
            .datasource_arg(quote! { datasource_arg })
            .return_type((Result<User, Error>))
            .base_doc_comment("Finds a user by their ID.")
            .doc_comment("This operation retrieves a single user record based on the provided ID.")
            .query_string("SELECT * FROM users WHERE id = ?")
            .input_parameters(quote! { &[id] })
            .parameterized(true)
            .with_unwrap(false)
            .build();

        // Act: Generate tokens
        let generated_tokens = find_operation.generate_tokens();

        // Assert: Compare against expected tokens
        let expected_tokens = quote! {
            #[doc = "Finds a user by their ID."]
            #[doc = "This operation retrieves a single user record based on the provided ID."]
            async fn find_user_by_id(datasource) -> Result<User, Error> {
                <User as canyon_sql::core::Transaction<User>>::query(
                    "SELECT * FROM users WHERE id = ?",
                    &[id],
                    datasource_arg
                ).await
                .into_results::<User>()
            }
        };

        assert_eq!(
            generated_tokens.to_string(),
            expected_tokens.to_string(),
            "Generated tokens do not match expected tokens!"
        );
    }

    #[test]
    fn test_insert_operation_tokens() {
        // Arrange: Build the operation
        let insert_operation = OperationBuilder::new()
            .operation_name("insert")
            .fn_name("insert_user")
            .datasource_param(parse_quote!(datasource))
            .datasource_arg(quote! { datasource_arg })
            .return_type(parse_quote!(Result<(), Error>))
            .base_doc_comment("Inserts a new user into the database.")
            .doc_comment("This operation inserts a new user record with the provided data.")
            .query_string("INSERT INTO users (name, email) VALUES (?, ?)")
            .input_parameters(quote! { &dyn QueryParameters })
            .parameterized(true)
            .with_unwrap(false)
            .build();

        // Act: Generate tokens
        let generated_tokens = insert_operation.generate_tokens();

        // Assert: Compare against expected tokens
        let expected_tokens = quote! {
            #[doc = "Inserts a new user into the database."]
            #[doc = "This operation inserts a new user record with the provided data."]
            async fn insert_user(datasource) -> Result<(), Error> {
                <User as canyon_sql::core::Transaction<User>>::query(
                    "INSERT INTO users (name, email) VALUES (?, ?)",
                    &dyn QueryParameters,
                    datasource_arg
                ).await
                .into_results::<User>()
            }
        };

        assert_eq!(
            generated_tokens.to_string(),
            expected_tokens.to_string(),
            "Generated tokens do not match expected tokens!"
        );
    }
}

