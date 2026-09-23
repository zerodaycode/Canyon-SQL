use proc_macro2::{Ident, Span};

/// Autogenerates a default table name for an entity given their struct name
/// TODO: This is duplicated from the macro's crate. We should be able to join both crates in
/// one later, but now, for developing purposes, we need to maintain here for a while this here
pub fn default_database_table_name_from_entity_name(ty: &str) -> String {
    let mut table_name: String = String::new();

    let mut index = 0;
    for char in ty.chars() {
        if index < 1 {
            table_name.push(char.to_ascii_lowercase());
            index += 1;
        } else {
            match char {
                n if n.is_ascii_uppercase() => {
                    table_name.push('_');
                    table_name.push(n.to_ascii_lowercase());
                }
                _ => table_name.push(char),
            }
        }
    }

    table_name
}

/// Converts a snake-case database table name into its Rust type identifier.
///
/// Invalid names are reported as macro diagnostics instead of panicking in the
/// compiler process.
pub fn database_table_name_to_struct_ident(name: &str) -> syn::Result<Ident> {
    let mut struct_name = String::new();

    if name.is_empty() {
        return Err(invalid_table_name(name));
    }

    for segment in name.split('_') {
        let mut chars = segment.chars();
        let Some(first) = chars.next() else {
            return Err(invalid_table_name(name));
        };

        if !first.is_ascii_alphabetic() || !chars.clone().all(|char| char.is_ascii_alphanumeric()) {
            return Err(invalid_table_name(name));
        }

        struct_name.push(first.to_ascii_uppercase());
        struct_name.extend(chars);
    }

    syn::parse_str(&struct_name).map_err(|_| invalid_table_name(name))
}

fn invalid_table_name(name: &str) -> syn::Error {
    syn::Error::new(
        Span::call_site(),
        format!(
            "invalid database table name `{name}`: expected snake_case ASCII letters and digits"
        ),
    )
}

#[cfg(test)]
mod default_table_name_from_entity_name_tests {
    use crate::helpers::{
        database_table_name_to_struct_ident, default_database_table_name_from_entity_name,
    };

    #[test]
    #[cfg(not(target_env = "msvc"))]
    fn test_entity_database_name_defaulter() {
        assert_eq!(
            default_database_table_name_from_entity_name("League"),
            "league".to_owned()
        );
        assert_eq!(
            default_database_table_name_from_entity_name("MajorLeague"),
            "major_league".to_owned()
        );
        assert_eq!(
            default_database_table_name_from_entity_name("MajorLeagueTournament"),
            "major_league_tournament".to_owned()
        );

        assert_ne!(
            default_database_table_name_from_entity_name("MajorLeague"),
            "majorleague".to_owned()
        );
        assert_ne!(
            default_database_table_name_from_entity_name("MajorLeague"),
            "MajorLeague".to_owned()
        );
    }

    #[test]
    fn converts_compound_table_names_to_pascal_case() {
        assert_eq!(
            database_table_name_to_struct_ident("team_members")
                .unwrap()
                .to_string(),
            "TeamMembers"
        );
        assert_eq!(
            database_table_name_to_struct_ident("league_region2")
                .unwrap()
                .to_string(),
            "LeagueRegion2"
        );
    }

    #[test]
    fn rejects_invalid_table_names_without_panicking() {
        for table_name in ["", "_teams", "teams_", "team__members", "team-members"] {
            assert!(database_table_name_to_struct_ident(table_name).is_err());
        }
    }
}
