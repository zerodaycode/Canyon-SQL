use proc_macro2::{TokenStream, Ident, Span};
use quote::quote;

use super::{
    entity::CanyonEntity,
    field_annotation::EntityFieldAnnotation
};

/// Builds the TokenStream that contains the struct definition code
pub fn generate_data_struct(canyon_entity: &CanyonEntity) -> TokenStream {
    let fields = &canyon_entity.get_attrs_as_token_stream();

    let struct_name = &canyon_entity.struct_name;
    let struct_visibility = &canyon_entity.vis;
    let struct_generics = &canyon_entity.generics;

    quote! {
        #struct_visibility struct #struct_name #struct_generics {
            #(#fields),*
        }
    }
}

pub fn generate_fields_names_for_enum(canyon_entity: &CanyonEntity) -> TokenStream {
    let enum_name = Ident::new(
        // TODO Convert it to CamelCase
        (canyon_entity.struct_name.to_string() + "Fields").as_str(),
        Span::call_site()
    );

    let fields_names = &canyon_entity.get_fields_as_enum_variants();
    let match_arms = &canyon_entity.create_match_arm_for_relate_field(&enum_name);

    let visibility = &canyon_entity.vis;
    let generics = &canyon_entity.generics;

    quote! {
        #[derive(Debug)]
        #visibility enum #enum_name #generics {
            #(#fields_names),*
        }

        impl canyon_sql::bounds::FieldIdentifier for #generics #enum_name #generics {
            fn value(self) -> String {
                match self {
                    #(#match_arms),*
                }
            }
        }

        impl #generics std::fmt::Display for #enum_name #generics {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "") // Pa ir tirando
            }
        }
    }
}


/// Builds the tokens for generate the code that implements the TryFrom<&CanyonEntity>
/// for the macro annotated struct
pub fn get_field_attr(metrics_struct: &CanyonEntity) -> () {
    let _field_attributes = metrics_struct
        .attributes
        .iter()
        .map(|field| {
            match field.attribute {
                Some(EntityFieldAnnotation::ForeignKey) => {
                    println!("Annotation ForeignKey found in field: {} for {} entity", 
                        &field.name, &metrics_struct.struct_name
                    );
                },
                _ => {
                    println!("No annotation found for field: {} in {} entity", 
                        &field.name, &metrics_struct.struct_name
                    );
                },
            };
        })
        .collect::<Vec<_>>();

        ()
}