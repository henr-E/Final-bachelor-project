use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Error, Fields, Path, TypeParamBound};

struct DerivedFunctions {
    get_structure: TokenStream,
    from_value: TokenStream,
    to_value: TokenStream,
}

pub fn derive_component_piece(
    item: proc_macro::TokenStream,
    crate_name: Path,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let name = item.ident;

    let trait_name: TypeParamBound = parse_quote!(#crate_name::component::ComponentPiece);

    // Add a bound `T: ComponentPirce` to every type parameter T.
    let generics = crate::add_trait_bounds(item.generics, trait_name.clone());

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let DerivedFunctions {
        get_structure,
        from_value,
        to_value,
    } = match get_structure(&item.data, trait_name.clone(), crate_name.clone()) {
        Ok(deriven) => deriven,
        Err(error) => return error.into_compile_error().into(),
    };

    let expanded = quote! {
        impl #impl_generics #trait_name for #name #ty_generics #where_clause {
            fn get_structure() -> #crate_name::component_structure::ComponentStructure {
                #get_structure
            }

            fn from_value(value: #crate_name::Value) -> Option<Self> {
                #from_value
            }

            fn to_value(&self) -> #crate_name::Value {
                #to_value
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

fn get_structure(
    data: &Data,
    trait_name: TypeParamBound,
    crate_name: Path,
) -> Result<DerivedFunctions, Error> {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let mut get_structure_parts = Vec::new();
                let mut from_value_parts = Vec::new();
                let mut to_value_parts = Vec::new();

                for f in &fields.named {
                    let name = f.ident.as_ref().unwrap();
                    let name_str = name.to_string();
                    let ty = &f.ty;

                    let get_structure = quote_spanned! {ty.span()=>
                        (
                            #name_str.to_owned(),
                            #crate_name::proto::ComponentStructure {
                                component_structure: Some(
                                    <#ty as #trait_name>::get_structure()
                                ),
                            },
                        )
                    };
                    let from_value = quote_spanned! {ty.span()=>
                        #name: #trait_name::from_value(fields.remove(#name_str)?)?
                    };
                    let to_value = quote_spanned! {ty.span()=>
                        (
                            #name_str.to_owned(),
                            #trait_name::to_value(&self.#name),
                        )
                    };

                    get_structure_parts.push(get_structure);
                    from_value_parts.push(from_value);
                    to_value_parts.push(to_value);
                }

                let get_structure = quote! {
                    #crate_name::component_structure::ComponentStructure::Struct(#crate_name::proto::ComponentStruct {
                        data: std::collections::HashMap::from([
                            #(#get_structure_parts),*
                        ]),
                    })
                };
                let from_value = quote! {
                    match value.kind? {
                        #crate_name::prost_types::value::Kind::StructValue(#crate_name::prost_types::Struct { mut fields }) => Some(Self {
                            #(#from_value_parts),*
                        }),
                        _ => None,
                    }
                };
                let to_value = quote! {
                    #crate_name::Value {
                        kind: Some(#crate_name::prost_types::value::Kind::StructValue(#crate_name::prost_types::Struct {
                            fields: std::collections::BTreeMap::from([
                                #(#to_value_parts),*
                            ]),
                        })),
                    }
                };

                Ok(DerivedFunctions {
                    get_structure,
                    from_value,
                    to_value,
                })
            }
            Fields::Unnamed(ref fields) => {
                if fields.unnamed.len() != 1 {
                    return Err(Error::new_spanned(
                        data.struct_token,
                        "Can't impl ComponentPiece on structs with more then one unnamed field",
                    ));
                }
                let field = fields.unnamed.first().unwrap();
                let ty = &field.ty;

                let get_structure = quote_spanned! {ty.span()=>
                    <#ty as #trait_name>::get_structure()
                };
                let from_value = quote_spanned! {ty.span()=>
                    Some(Self(<#ty as #trait_name>::from_value(value)?))
                };
                let to_value = quote_spanned! {ty.span()=>
                    #trait_name::to_value(&self.0)
                };

                Ok(DerivedFunctions {
                    get_structure,
                    from_value,
                    to_value,
                })
            }
            Fields::Unit => Err(Error::new_spanned(
                &data.fields,
                "Can't impl ComponentPiece on unit structs",
            )),
        },
        Data::Enum(ref e) => Err(Error::new_spanned(
            e.enum_token,
            "Can't impl ComponentPiece on enums",
        )),
        Data::Union(ref u) => Err(Error::new_spanned(
            u.union_token,
            "Can't impl ComponentPiece on unions",
        )),
    }
}
