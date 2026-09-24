/// Generates the default snake-case table name for a Rust entity type.
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
