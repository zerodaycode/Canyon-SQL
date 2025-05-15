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
