use crate::utils::macro_tokens::MacroTokens;
use canyon_entities::field_annotation::EntityFieldAnnotation;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::Path;

/// Generates all read operations derived from Canyon's `foreign_key` field annotation.
///
/// A foreign key represents an outbound relationship from the current entity to another
/// entity. For example, if `Player.team_id` points to `Team.id`, then `Player` owns the
/// foreign-key column and `Team` is the referenced parent entity.
///
/// From a single `foreign_key(references = Team::id)` annotation, Canyon
/// generates two families of operations:
///
/// - Parent lookup operations: instance methods that start from the child entity and fetch
///   the referenced parent entity. Example: `player.find_team().await`.
/// - Child lookup operations: associated functions that start from a parent entity and fetch
///   all child rows that reference it. Example: `Player::find_all_by_team(&team).await`.
///
/// Method names are derived from the annotated field. The referenced table is
/// obtained from the related entity's generated metadata, so the annotation
/// cannot disagree with its `canyon_entity` table mapping.
pub fn generate_find_by_fk_ops(
    macro_data: &MacroTokens<'_>,
    table_schema_data: &str,
) -> syn::Result<TokenStream> {
    __operations::generate_find_by_fk_ops(macro_data, table_schema_data)
}

fn database_field_name(field_ident: &Ident) -> String {
    field_ident.to_string().trim_start_matches("r#").to_owned()
}

mod __operations {
    use super::*;
    use crate::utils::helpers::{CanyonMethodKind, ReturnTypeTokens};

    pub(super) fn generate_find_by_fk_ops(
        macro_data: &MacroTokens<'_>,
        table_schema_data: &str,
    ) -> syn::Result<TokenStream> {
        let ty = macro_data.ty;
        let (impl_generics, ty_generics, where_clause) = macro_data.generics.split_for_impl();
        let foreign_keys = macro_data.get_fk_annotations()?;
        validate_unique_relation_names(&foreign_keys)?;

        let parent_lookup_operations = generate_parent_lookup_tokens(&foreign_keys);
        let child_lookup_operations =
            generate_child_lookup_tokens(macro_data, table_schema_data, &foreign_keys);

        if parent_lookup_operations.is_empty() && child_lookup_operations.is_empty() {
            return Ok(TokenStream::new());
        }

        let method_implementations = parent_lookup_operations
            .iter()
            .chain(child_lookup_operations.iter())
            .map(FkOperationTokens::implementation);

        Ok(quote! {
            impl #impl_generics #ty #ty_generics #where_clause {
                #(#method_implementations)*
            }
        })
    }

    /// Generates parent lookup operations for every foreign-key field declared by the entity.
    ///
    /// A parent lookup follows the foreign-key reference from the current row to the row it points
    /// to. In relational terms, this is the many-to-one side of the relationship.
    fn generate_parent_lookup_tokens(
        foreign_keys: &[(&Ident, EntityFieldAnnotation)],
    ) -> Vec<FkOperationTokens> {
        let mut operations = Vec::new();

        for (field_ident, annotation) in foreign_keys {
            if let EntityFieldAnnotation::ForeignKey { entity, field } = annotation {
                let relation_name = relation_name(field_ident);
                let method_name = format_ident!("find_{}", relation_name);
                let method_name_with = format_ident!("find_{}_with", relation_name);
                let query_source = __detail::LookupQuerySource::Parent {
                    entity,
                    predicate_field: field,
                };

                let fk_operation = __impl::generate_fk_operations_tokens(
                    query_source,
                    field_ident,
                    &method_name,
                    entity,
                    ReturnTypeTokens::Option,
                    CanyonMethodKind::Default,
                );

                let fk_operation_with = __impl::generate_fk_operations_tokens(
                    query_source,
                    field_ident,
                    &method_name_with,
                    entity,
                    ReturnTypeTokens::Option,
                    CanyonMethodKind::WithInput,
                );

                operations.extend([fk_operation, fk_operation_with]);
            }
        }

        operations
    }

    /// Generates child lookup operations for every foreign-key field declared by the entity.
    ///
    /// This is sometimes called a "reverse foreign-key search", but the database does not contain
    /// a second or inverted foreign key. The same child table foreign-key column is reused in the
    /// opposite navigation direction: starting from a parent row, Canyon fetches all child rows that
    /// reference it.
    fn generate_child_lookup_tokens(
        macro_data: &MacroTokens<'_>,
        table_schema_data: &str,
        foreign_keys: &[(&Ident, EntityFieldAnnotation)],
    ) -> Vec<FkOperationTokens> {
        let ty = macro_data.ty;
        let mapper_ty = macro_data
            .retrieve_mapping_target_type()
            .as_ref()
            .unwrap_or(ty);
        let mapper_ty = Path::from(mapper_ty.clone());
        let mut operations = Vec::new();

        for (field_ident, annotation) in foreign_keys {
            if let EntityFieldAnnotation::ForeignKey { entity, field } = annotation {
                let relation_name = relation_name(field_ident);
                let method_name = format_ident!("find_all_by_{}", relation_name);
                let method_name_with = format_ident!("find_all_by_{}_with", relation_name);
                let field_name = database_field_name(field_ident);
                let query_source = __detail::LookupQuerySource::Child {
                    table: table_schema_data,
                    predicate_column: &field_name,
                };

                let child_operation = __impl::generate_child_fk_operations_tokens(
                    query_source,
                    field,
                    &method_name,
                    &mapper_ty,
                    entity,
                    CanyonMethodKind::Default,
                );

                let child_operation_with = __impl::generate_child_fk_operations_tokens(
                    query_source,
                    field,
                    &method_name_with,
                    &mapper_ty,
                    entity,
                    CanyonMethodKind::WithInput,
                );

                operations.extend([child_operation, child_operation_with]);
            }
        }

        operations
    }

    fn relation_name(field_ident: &Ident) -> String {
        let field_name = database_field_name(field_ident);
        field_name
            .strip_suffix("_id")
            .unwrap_or(&field_name)
            .to_owned()
    }

    fn validate_unique_relation_names(
        foreign_keys: &[(&Ident, EntityFieldAnnotation)],
    ) -> syn::Result<()> {
        let mut relation_names = HashSet::new();

        for (field_ident, _) in foreign_keys {
            let relation_name = relation_name(field_ident);
            if !relation_names.insert(relation_name.clone()) {
                return Err(syn::Error::new_spanned(
                    field_ident,
                    format!(
                        "duplicate Canyon relationship name `{relation_name}` after normalizing the field name"
                    ),
                ));
            }
        }

        Ok(())
    }
}

mod __impl {
    use super::*;
    use crate::utils::helpers::{CanyonMethodKind, ReturnTypeTokens};

    pub(crate) fn generate_fk_operations_tokens(
        query_source: __detail::LookupQuerySource<'_>,
        field_ident: &Ident,
        method_name: &Ident,
        parent_ty: &Path,
        return_type_tokens: ReturnTypeTokens,
        method_kind: CanyonMethodKind,
    ) -> FkOperationTokens {
        __detail::generate_operation_tokens(
            query_source,
            __detail::LookupValueSource::SelfField(field_ident),
            method_name,
            parent_ty,
            return_type_tokens,
            __detail::FkLookupKind::Parent,
            method_kind,
        )
    }

    pub(crate) fn generate_child_fk_operations_tokens(
        query_source: __detail::LookupQuerySource<'_>,
        referenced_field: &Ident,
        method_name: &Ident,
        mapper_ty: &Path,
        parent_ty: &Path,
        method_kind: CanyonMethodKind,
    ) -> FkOperationTokens {
        __detail::generate_operation_tokens(
            query_source,
            __detail::LookupValueSource::ParentField(referenced_field),
            method_name,
            mapper_ty,
            ReturnTypeTokens::Vec,
            __detail::FkLookupKind::Child { parent_ty },
            method_kind,
        )
    }
}

mod __detail {
    use super::*;
    use crate::utils::helpers::{CanyonMethodKind, ReturnTypeTokens};

    #[derive(Clone, Copy)]
    pub(super) enum FkLookupKind<'a> {
        Parent,
        Child { parent_ty: &'a Path },
    }

    #[derive(Clone, Copy)]
    pub(super) enum LookupQuerySource<'a> {
        Parent {
            entity: &'a Path,
            predicate_field: &'a Ident,
        },
        Child {
            table: &'a str,
            predicate_column: &'a str,
        },
    }

    #[derive(Clone, Copy)]
    pub(super) enum LookupValueSource<'a> {
        SelfField(&'a Ident),
        ParentField(&'a Ident),
    }

    pub(super) fn generate_operation_tokens(
        query_source: LookupQuerySource<'_>,
        lookup_value_source: LookupValueSource<'_>,
        method_name: &Ident,
        return_ty: &Path,
        return_type_tokens: ReturnTypeTokens,
        lookup_kind: FkLookupKind<'_>,
        method_kind: CanyonMethodKind,
    ) -> FkOperationTokens {
        let signature = create_method_signature(
            method_name,
            return_ty,
            return_type_tokens,
            lookup_kind,
            method_kind,
        );

        let implementation = generate_method_implementation_body(
            query_source,
            lookup_value_source,
            return_ty,
            lookup_kind,
            method_kind,
        );

        FkOperationTokens::new(signature, implementation)
    }

    fn create_method_signature(
        method_name: &Ident,
        return_ty: &Path,
        ret_ty: ReturnTypeTokens,
        lookup_kind: FkLookupKind<'_>,
        method_kind: CanyonMethodKind,
    ) -> TokenStream {
        let method_generics_and_args = method_generics_and_args(lookup_kind, method_kind);
        let where_clause = get_where_clause(method_kind);
        let documentation = method_documentation(return_ty, lookup_kind, method_kind);

        quote! {
            #[doc = #documentation]
            pub async fn #method_name #method_generics_and_args
                -> canyon_sql::CanyonResult<#ret_ty<#return_ty>>
                #where_clause
        }
    }

    fn method_documentation(
        return_ty: &Path,
        lookup_kind: FkLookupKind<'_>,
        method_kind: CanyonMethodKind,
    ) -> String {
        let return_ty = quote!(#return_ty).to_string();
        let datasource = match method_kind {
            CanyonMethodKind::Default => "the default datasource",
            CanyonMethodKind::WithInput => "the supplied connection or datasource",
        };

        match lookup_kind {
            FkLookupKind::Parent => {
                format!("Loads the related `{return_ty}` through {datasource}.")
            }
            FkLookupKind::Child { .. } => {
                format!("Loads all related `{return_ty}` rows through {datasource}.")
            }
        }
    }

    fn method_generics_and_args(
        lookup_kind: FkLookupKind<'_>,
        method_kind: CanyonMethodKind,
    ) -> TokenStream {
        match (lookup_kind, method_kind) {
            (FkLookupKind::Parent, CanyonMethodKind::Default) => quote! { (&self) },
            (FkLookupKind::Parent, CanyonMethodKind::WithInput) => {
                quote! { <I>(&self, input: I) }
            }
            (FkLookupKind::Child { parent_ty }, CanyonMethodKind::Default) => {
                quote! { (value: &#parent_ty) }
            }
            (FkLookupKind::Child { parent_ty }, CanyonMethodKind::WithInput) => {
                quote! { <I>(value: &#parent_ty, input: I) }
            }
        }
    }

    fn get_where_clause(method_kind: CanyonMethodKind) -> TokenStream {
        match method_kind {
            CanyonMethodKind::Default => quote! {},
            CanyonMethodKind::WithInput => quote! {
                where I: canyon_sql::connection::DbConnection + Send
            },
        }
    }

    fn connection_binding(method_kind: CanyonMethodKind) -> TokenStream {
        match method_kind {
            CanyonMethodKind::Default => quote! {
                let db_conn = canyon_sql::core::Canyon::instance()?
                    .get_default_connection()?;
            },
            CanyonMethodKind::WithInput => quote! {
                let db_conn = input;
            },
        }
    }

    fn query_builder_stmt(query_source: LookupQuerySource) -> TokenStream {
        match query_source {
            LookupQuerySource::Parent {
                entity,
                predicate_field,
            } => {
                let predicate_column = database_field_name(predicate_field);
                quote! {
                    let db_type = db_conn.get_database_type()?;
                    let stmt = canyon_sql::query::querybuilder::SelectQueryBuilder::new(
                        <#entity as canyon_sql::query::bounds::EntityRuntimeInfo>::TABLE,
                        db_type,
                    )
                    .r#where(#predicate_column, canyon_sql::query::operators::Operator::Eq)
                    .build()?;
                }
            }
            LookupQuerySource::Child {
                table,
                predicate_column,
            } => quote! {
                let db_type = db_conn.get_database_type()?;
                let stmt = canyon_sql::query::querybuilder::SelectQueryBuilder::new(#table, db_type)
                    .r#where(#predicate_column, canyon_sql::query::operators::Operator::Eq)
                    .build()?;
            },
        }
    }

    fn lookup_value_tokens(lookup_value_source: LookupValueSource<'_>) -> TokenStream {
        match lookup_value_source {
            LookupValueSource::SelfField(field_ident) => quote! {
                &self.#field_ident as &dyn canyon_sql::query::QueryParameter
            },
            LookupValueSource::ParentField(field) => quote! {
                &value.#field as &dyn canyon_sql::query::QueryParameter
            },
        }
    }

    fn query_execution_tokens(
        lookup_value_source: LookupValueSource<'_>,
        return_ty: &Path,
        lookup_kind: FkLookupKind<'_>,
    ) -> TokenStream {
        let lookup_value = lookup_value_tokens(lookup_value_source);

        match lookup_kind {
            FkLookupKind::Parent => quote! {
                db_conn
                    .query_one::<#return_ty>(
                        stmt.sql(),
                        &[#lookup_value],
                    )
                    .await
            },
            FkLookupKind::Child { .. } => quote! {
                db_conn
                    .query::<&str, #return_ty>(
                        stmt.sql(),
                        &[#lookup_value],
                    )
                    .await
            },
        }
    }

    fn generate_method_implementation_body(
        query_source: LookupQuerySource<'_>,
        lookup_value_source: LookupValueSource<'_>,
        return_ty: &Path,
        lookup_kind: FkLookupKind<'_>,
        method_kind: CanyonMethodKind,
    ) -> TokenStream {
        let connection_binding = connection_binding(method_kind);
        let query_builder_stmt = query_builder_stmt(query_source);
        let query_execution = query_execution_tokens(lookup_value_source, return_ty, lookup_kind);

        quote! {
            {
                use canyon_sql::connection::DbConnection;
                use canyon_sql::query::querybuilder::{QueryBuilderExt, SelectQueryBuilderExt};

                #connection_binding
                #query_builder_stmt
                #query_execution
            }
        }
    }
}

#[derive(Debug)]
struct FkOperationTokens {
    implementation: TokenStream,
}

impl FkOperationTokens {
    fn new(signature: TokenStream, implementation: TokenStream) -> Self {
        let method_implementation = quote! {
            #signature #implementation
        };
        Self {
            implementation: method_implementation,
        }
    }

    fn implementation(&self) -> &TokenStream {
        &self.implementation
    }
}

#[cfg(test)]
mod tests {
    use super::generate_find_by_fk_ops;
    use crate::utils::macro_tokens::MacroTokens;
    use syn::DeriveInput;

    fn generated_tokens(source: &str) -> String {
        let ast = syn::parse_str::<DeriveInput>(source).unwrap();
        let macro_data = MacroTokens::new(&ast).unwrap();

        generate_find_by_fk_ops(&macro_data, "players")
            .unwrap()
            .to_string()
            .replace(' ', "")
    }

    #[test]
    fn generates_relation_oriented_names_from_the_field() {
        let tokens = generated_tokens(
            r#"
                struct Player {
                    #[foreign_key(references = Team::id)]
                    team_id: i64,
                }
            "#,
        );

        assert!(tokens.contains("implPlayer"));
        assert!(tokens.contains("pubasyncfnfind_team(&self)"));
        assert!(tokens.contains("pubasyncfnfind_team_with<I>(&self,input:I)"));
        assert!(tokens.contains("pubasyncfnfind_all_by_team(value:&Team)"));
        assert!(tokens.contains("pubasyncfnfind_all_by_team_with<I>(value:&Team,input:I)"));
        assert!(tokens.contains("<Teamascanyon_sql::query::bounds::EntityRuntimeInfo>::TABLE"));
        assert!(tokens.contains("&value.idas&dyncanyon_sql::query::QueryParameter"));
        assert!(!tokens.contains("traitPlayerRelations"));
        assert!(!tokens.contains("FkOperations"));
        assert!(!tokens.contains("childrens"));
    }

    #[test]
    fn relation_names_do_not_depend_on_the_physical_table_name() {
        let tokens = generated_tokens(
            r#"
                struct AuditEntry {
                    #[foreign_key(references = crate::models::User::id)]
                    created_by_id: i64,
                }
            "#,
        );

        assert!(tokens.contains("pubasyncfnfind_created_by(&self)"));
        assert!(tokens.contains("CanyonResult<Option<crate::models::User>>"));
        assert!(tokens.contains("pubasyncfnfind_all_by_created_by(value:&crate::models::User)"));
        assert!(!tokens.contains("pubasyncfnfind_users"));
        assert!(!tokens.contains("pubasyncfnfind_all_by_users"));
    }

    #[test]
    fn raw_identifier_fields_generate_valid_method_names() {
        let tokens = generated_tokens(
            r#"
                struct Event {
                    #[foreign_key(references = EventType::id)]
                    r#type: i64,
                }
            "#,
        );

        assert!(tokens.contains("pubasyncfnfind_type(&self)"));
        assert!(tokens.contains("pubasyncfnfind_all_by_type(value:&EventType)"));
    }

    #[test]
    fn rejects_relationship_names_that_collide_after_normalization() {
        let ast = syn::parse_str::<DeriveInput>(
            r#"
                struct Player {
                    #[foreign_key(references = Team::id)]
                    team: i64,
                    #[foreign_key(references = Team::id)]
                    team_id: i64,
                }
            "#,
        )
        .unwrap();
        let macro_data = MacroTokens::new(&ast).unwrap();

        let error = generate_find_by_fk_ops(&macro_data, "players").unwrap_err();

        assert!(
            error
                .to_string()
                .contains("duplicate Canyon relationship name `team`")
        );
    }
}
