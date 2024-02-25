use darling::{FromAttributes, FromMeta};
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, Path, TypeParamBound};

#[derive(FromMeta)]
enum ComponentType {
    Edge,
    Node,
    Global,
}

#[derive(FromAttributes)]
#[darling(attributes(component))]
struct ComponentAttr {
    name: String,
    ty: ComponentType,
}

pub fn derive_component(
    item: proc_macro::TokenStream,
    crate_name: Path,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let name = item.ident;

    let trait_name: TypeParamBound = parse_quote!(#crate_name::component::Component);

    // Add a bound `T: Component` to every type parameter T.
    let generics = crate::add_trait_bounds(item.generics, trait_name.clone());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let attribute = match ComponentAttr::from_attributes(&item.attrs) {
        Ok(a) => a,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let component_name = attribute.name;

    let component_type = match attribute.ty {
        ComponentType::Edge => quote!(Edge),
        ComponentType::Node => quote!(Node),
        ComponentType::Global => quote!(Global),
    };

    let expanded = quote! {
        impl #impl_generics #trait_name for #name #ty_generics #where_clause {
            fn get_name() -> String {
                #component_name.to_string()
            }

            fn get_spec() -> #crate_name::ComponentSpecification {
                #crate_name::ComponentSpecification {
                    r#type: #crate_name::proto::ComponentType::#component_type.into(),
                    structure: Some(#crate_name::ComponentStructure {
                        component_structure: Some(<Self as #crate_name::component::ComponentPiece>::get_structure()),
                    }),
                }
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
