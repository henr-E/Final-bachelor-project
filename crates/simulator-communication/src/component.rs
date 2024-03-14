//! The building blocks of the Graph data structure.
//!
//! These traits need to be implemented on every rust type you want to use as a component in the
//! simulation.

use prost_types::value::Kind;

use crate::{component_structure::ComponentStructure, ComponentSpecification, Value};

/// Trait for a component to be used in the simulation
///
/// You will likely not want to implement this trait yourself but use the derive macro
/// with the same name: [`Component`](derive@crate::Component).
///
/// # Example:
/// ```
/// # use simulator_communication::{Component, ComponentPiece};
///
/// #[derive(ComponentPiece, Component)]
/// #[component(name = "transmission-line", ty = "edge")]
/// struct TransmissionLine {
///     above_ground: bool,
///     length: f64,
/// }
/// ```
pub trait Component: ComponentPiece {
    /// The name of this component, will be used while communicating with the manager.
    fn get_name() -> String;

    /// The "shape" of this component. i.e. what fields and types it has.
    /// Together with the type of the component.
    fn get_spec() -> ComponentSpecification;
}

/// Trait for a part of component.
///
/// This trait is useful if you want to use a struct as part of [`Component`] or [`ComponentPiece`].
/// But don't want it to be a component on its own.
///
/// You will likely not want to implement this trait yourself but use the derive macro
/// with the same name: [`ComponentPiece`](derive@crate::ComponentPiece).
///
/// # Example:
/// ```
/// # use simulator_communication::{Component, ComponentPiece};
///
/// #[derive(ComponentPiece)]
/// struct ExampleSingleValueComp(f64);
///
/// #[derive(ComponentPiece)]
/// struct SharedElecData {
///     max_volt: f64,
///     current_volt: f64,
/// }
///
/// #[derive(ComponentPiece, Component)]
/// #[component(name = "transmission-line", ty = "edge")]
/// struct TransmissionLine {
///     shared_elec_data: SharedElecData,
///     length: f64,
/// }
/// ```
pub trait ComponentPiece: Sized + Send + Sync + 'static {
    /// The "shape" of this component. i.e. what fields and types it has.
    fn get_structure() -> ComponentStructure;

    /// Convert a [`Value`] into this type.
    fn from_value(value: Value) -> Option<Self>;
    /// Convert this type into a [`Value`].
    fn to_value(&self) -> Value;
}

macro_rules! impl_primitive_component_piece {
    ($rty:ty, $cty:ident) => {
        impl ComponentPiece for $rty {
            fn get_structure() -> ComponentStructure {
                ComponentStructure::Primitive(crate::proto::ComponentPrimitive::$cty.into())
            }

            fn from_value(value: Value) -> Option<Self> {
                match value.kind? {
                    prost_types::value::Kind::NumberValue(n) => Some(n as $rty),
                    _ => None,
                }
            }

            fn to_value(&self) -> Value {
                Value {
                    kind: Some(prost_types::value::Kind::NumberValue(*self as f64)),
                }
            }
        }
    };
}

impl_primitive_component_piece!(u8, U8);
impl_primitive_component_piece!(u16, U16);
impl_primitive_component_piece!(u32, U32);
impl_primitive_component_piece!(u64, U64);
impl_primitive_component_piece!(u128, U128);

impl_primitive_component_piece!(i8, I8);
impl_primitive_component_piece!(i16, I16);
impl_primitive_component_piece!(i32, I32);
impl_primitive_component_piece!(i64, I64);
impl_primitive_component_piece!(i128, I128);

impl_primitive_component_piece!(f32, F32);
impl_primitive_component_piece!(f64, F64);

impl ComponentPiece for bool {
    fn get_structure() -> ComponentStructure {
        ComponentStructure::Primitive(crate::proto::ComponentPrimitive::Bool.into())
    }

    fn from_value(value: Value) -> Option<Self> {
        match value.kind? {
            Kind::BoolValue(b) => Some(b),
            _ => None,
        }
    }

    fn to_value(&self) -> Value {
        Value {
            kind: Some(Kind::BoolValue(*self)),
        }
    }
}

impl ComponentPiece for String {
    fn get_structure() -> ComponentStructure {
        ComponentStructure::Primitive(crate::proto::ComponentPrimitive::String.into())
    }

    fn from_value(value: Value) -> Option<Self> {
        match value.kind? {
            Kind::StringValue(s) => Some(s),
            _ => None,
        }
    }

    fn to_value(&self) -> Value {
        Value {
            kind: Some(Kind::StringValue(self.into())),
        }
    }
}

impl<T: ComponentPiece> ComponentPiece for Option<T> {
    fn get_structure() -> ComponentStructure {
        ComponentStructure::Option(Box::new(crate::proto::ComponentStructure {
            component_structure: Some(T::get_structure()),
        }))
    }

    fn from_value(value: Value) -> Option<Self> {
        match value.kind? {
            Kind::NullValue(_) => Some(None),
            kind => Some(T::from_value(Value { kind: Some(kind) })),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Some(t) => t.to_value(),
            None => Value {
                kind: Some(Kind::NullValue(0)),
            },
        }
    }
}

impl<T: ComponentPiece> ComponentPiece for Vec<T> {
    fn get_structure() -> ComponentStructure {
        ComponentStructure::List(Box::new(crate::proto::ComponentStructure {
            component_structure: Some(T::get_structure()),
        }))
    }

    fn from_value(value: Value) -> Option<Self> {
        match value.kind? {
            Kind::ListValue(l) => l.values.into_iter().map(|v| T::from_value(v)).collect(),
            _ => None,
        }
    }

    fn to_value(&self) -> Value {
        Value {
            kind: Some(Kind::ListValue(prost_types::ListValue {
                values: self.iter().map(|t| t.to_value()).collect(),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use super::*;
    use crate::{proto, Component, ComponentPiece};

    #[derive(ComponentPiece, Component, PartialEq, Debug)]
    #[component(name = "transmision-line", ty = "global")]
    struct TransLine {
        elec: ElecInfo,
        length: f64,
    }

    #[derive(ComponentPiece, PartialEq, Debug)]
    struct ElecInfo(u32);

    #[test]
    fn component_derive() {
        #[derive(ComponentPiece, Component)]
        #[component(name = "node-example", ty = "node")]
        struct NodeEx {
            elec: ElecInfo,
            length: f64,
        }

        assert_eq!(NodeEx::get_name(), "node-example".to_owned());
        assert_eq!(
            NodeEx::get_spec().r#type,
            Into::<i32>::into(proto::ComponentType::Node)
        );

        #[derive(ComponentPiece, Component)]
        #[component(name = "edge-example", ty = "edge")]
        struct EdgeEx {
            elec: ElecInfo,
            length: f64,
        }

        assert_eq!(EdgeEx::get_name(), "edge-example".to_owned());
        assert_eq!(
            EdgeEx::get_spec().r#type,
            Into::<i32>::into(proto::ComponentType::Edge)
        );

        #[derive(ComponentPiece, Component)]
        #[component(name = "global-example", ty = "global")]
        struct GlobalEx {
            elec: ElecInfo,
            length: f64,
        }

        assert_eq!(GlobalEx::get_name(), "global-example".to_owned());
        assert_eq!(
            GlobalEx::get_spec().r#type,
            Into::<i32>::into(proto::ComponentType::Global)
        );
    }

    #[test]
    fn component_piece_derive() {
        assert_eq!(
            TransLine::get_structure(),
            ComponentStructure::Struct(proto::ComponentStruct {
                data: HashMap::from([
                    (
                        "elec".to_owned(),
                        proto::ComponentStructure {
                            component_structure: Some(ComponentStructure::Primitive(
                                proto::ComponentPrimitive::U32.into()
                            ))
                        }
                    ),
                    (
                        "length".to_owned(),
                        proto::ComponentStructure {
                            component_structure: Some(ComponentStructure::Primitive(
                                proto::ComponentPrimitive::F64.into()
                            ))
                        }
                    )
                ])
            })
        );

        assert_eq!(
            TransLine::from_value(Value {
                kind: Some(Kind::StructValue(prost_types::Struct {
                    fields: BTreeMap::from([
                        (
                            "elec".to_owned(),
                            Value {
                                kind: Some(Kind::NumberValue(123.0))
                            }
                        ),
                        (
                            "length".to_owned(),
                            Value {
                                kind: Some(Kind::NumberValue(1.2))
                            }
                        ),
                    ])
                }))
            }),
            Some(TransLine {
                elec: ElecInfo(123),
                length: 1.2
            })
        );

        assert_eq!(
            TransLine {
                elec: ElecInfo(123),
                length: 1.2
            }
            .to_value(),
            Value {
                kind: Some(Kind::StructValue(prost_types::Struct {
                    fields: BTreeMap::from([
                        (
                            "elec".to_owned(),
                            Value {
                                kind: Some(Kind::NumberValue(123.0))
                            }
                        ),
                        (
                            "length".to_owned(),
                            Value {
                                kind: Some(Kind::NumberValue(1.2))
                            }
                        ),
                    ])
                }))
            }
        )
    }
}
