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

/// Parses the content of a &str to get the related identifier of a type
pub fn database_table_name_to_struct_ident(name: &str) -> Ident {
    let mut struct_name: String = String::new();

    let mut first_iteration = true;
    let mut previous_was_underscore = false;

    for char in name.chars() {
        if first_iteration {
            struct_name.push(char.to_ascii_uppercase());
            first_iteration = false;
        } else {
            match char {
                '_' => {
                    previous_was_underscore = true;
                }
                char if char.is_ascii_lowercase() => {
                    if previous_was_underscore {
                        struct_name.push(char.to_ascii_lowercase())
                    } else {
                        struct_name.push(char)
                    }
                }
                _ => panic!("Detected wrong format or broken convention for database table names"),
            }
        }
    }

    Ident::new(&struct_name, Span::call_site())
}

#[cfg(test)]
mod default_table_name_from_entity_name_tests {
    use crate::helpers::default_database_table_name_from_entity_name;

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
}
