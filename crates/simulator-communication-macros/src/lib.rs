mod component;
mod component_piece;

use syn::{parse_quote, GenericParam, Generics, TypeParamBound};

fn add_trait_bounds(mut generics: Generics, trait_name: TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(trait_name.clone());
        }
    }
    generics
}

#[proc_macro_derive(Component, attributes(component))]
pub fn derive_component(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    component::derive_component(item, parse_quote!(::simulator_communication))
}

#[proc_macro_derive(ComponentPiece)]
pub fn derive_component_piece(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    component_piece::derive_component_piece(item, parse_quote!(::simulator_communication))
}
