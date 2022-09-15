use proc_macro2::Ident;


/// Parses a syn::Identifier to get a snake case database name from the type identifier
/// TODO: #[macro(table_name = 'user_defined_db_table_name)]' 
pub fn database_table_name_from_struct(ty: &Ident) -> String {

    let struct_name: String = String::from(ty.to_string());
    let mut table_name: String = String::new();
    
    let mut index = 0;
    for char in struct_name.chars() {
        if index < 1 {
            table_name.push(char.to_ascii_lowercase());
            index += 1;
        } else {
            match char {
                n if n.is_ascii_uppercase() => {
                    table_name.push('_');
                    table_name.push(n.to_ascii_lowercase()); 
                }
                _ => table_name.push(char)
            }
        }   
    }

    table_name
}

/// Parses a syn::Identifier to get a snake case database name from the type identifier
/// TODO: #[macro(table_name = 'user_defined_db_table_name)]' 
pub fn database_table_name_from_entity_name(ty: &str) -> String {

    let struct_name: String = String::from(ty.to_string());
    let mut table_name: String = String::new();
    
    let mut index = 0;
    for char in struct_name.chars() {
        if index < 1 {
            table_name.push(char.to_ascii_lowercase());
            index += 1;
        } else {
            match char {
                n if n.is_ascii_uppercase() => {
                    table_name.push('_');
                    table_name.push(n.to_ascii_lowercase()); 
                }
                _ => table_name.push(char)
            }
        }   
    }

    table_name
}

/// Parses the content of an &str to get the related identifier of a type
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
                n if n == '_' => {
                    previous_was_underscore = true;
                },
                char if char.is_ascii_lowercase() => {
                    if previous_was_underscore {
                        struct_name.push(char.to_ascii_lowercase())
                    } else { struct_name.push(char) }
                },
                _ => panic!("Detected wrong format or broken convention for database table names")
            }
        }   
    }

    Ident::new(
        &struct_name,
        proc_macro2::Span::call_site()
    )
}