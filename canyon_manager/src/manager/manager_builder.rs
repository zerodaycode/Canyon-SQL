use proc_macro2::TokenStream;
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