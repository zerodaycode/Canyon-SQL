use proc_macro2::TokenStream;
use quote::quote;

use super::{
    entity::CanyonEntity,
    field_annotation::EntityFieldAnnotation
};

/// Builds the TokenStream that contains the struct definition code
pub fn generate_data_struct(canyon_entity: &CanyonEntity) -> TokenStream {
    let fields = &canyon_entity
        .attributes
        .iter()
        .map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            quote!{ pub #name: #ty }
        })
        .collect::<Vec<_>>();

    let struct_name = &canyon_entity.struct_name;

    quote! {
        pub struct #struct_name {
            #(#fields),*
        }
    }
}


/// Builds the tokens for generate the code that implements the TryFrom<&CanyonEntity>
/// for the macro annotated struct
pub fn get_field_attr(metrics_struct: &CanyonEntity) -> () {
    let field_attributes = metrics_struct
        .attributes
        .iter()
        .map(|field| {
            match field.attribute_type {
                EntityFieldAnnotation::None => {
                    println!("No annotation found for field: {} in {} entity", 
                        &field.name, &metrics_struct.struct_name
                    );
                },
                EntityFieldAnnotation::ForeignKey => {
                    println!("Annotation ForeignKey found in field: {} for {} entity", 
                        &field.name, &metrics_struct.struct_name
                    );
                },
            };
        })
        .collect::<Vec<_>>();

        ()
}