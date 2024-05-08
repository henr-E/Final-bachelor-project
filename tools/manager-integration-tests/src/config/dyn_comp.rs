use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, bail, Context};

use super::{
    state_helpers::{parse_path_start, PathStart},
    ComponentInfo, ComponentStructure, TestConfigInitialState,
};

/// A component, or a part of it.
#[derive(Debug, Clone)]
pub(crate) enum Component {
    Int(i128),
    Float(f64),
    String(String),
    Bool(bool),
    OptionInt(Option<i128>),
    OptionFloat(Option<f64>),
    OptionString(Option<String>),
    OptionBool(Option<bool>),
    ListInt(Vec<i128>),
    ListFloat(Vec<f64>),
    ListString(Vec<String>),
    ListBool(Vec<bool>),
    Struct(HashMap<String, Component>),
}

impl Component {
    /// User facing name of this component part.
    fn type_str(&self) -> &'static str {
        match self {
            Component::Int(_) => "int",
            Component::Float(_) => "float",
            Component::String(_) => "string",
            Component::Bool(_) => "bool",
            Component::OptionInt(_) => "optionint",
            Component::OptionFloat(_) => "optionfloat",
            Component::OptionString(_) => "optionstring",
            Component::OptionBool(_) => "optionbool",
            Component::ListInt(_) => "listint",
            Component::ListFloat(_) => "listfloat",
            Component::ListString(_) => "liststring",
            Component::ListBool(_) => "listbool",
            Component::Struct(_) => "struct",
        }
    }

    /// Delete all fields that are not set by the fill checker.
    ///
    /// Returns true if any field is kept.
    fn keep_filled(&mut self, fill_comp: &ComponentFillChecker) -> bool {
        match (self, fill_comp) {
            (Component::Struct(map), ComponentFillChecker::Struct(fill_map)) => {
                *map = map
                    .drain()
                    .filter_map(|(name, mut comp)| {
                        let fill_comp = fill_map.get(&name)?;
                        if !comp.keep_filled(fill_comp) {
                            return None;
                        }
                        Some((name, comp))
                    })
                    .collect();
                !map.is_empty()
            }
            (_, ComponentFillChecker::Value(filled)) => *filled,
            _ => unreachable!("invalid fill_comp"),
        }
    }

    /// Sets the values in this component with the values from the `value`. An error is returned if
    /// the shape of `self` and `value` don't match.
    fn set_from_value(&mut self, value: toml::Value) -> anyhow::Result<()> {
        match (self, value) {
            (Component::String(s), toml::Value::String(ns)) => *s = ns,
            (Component::Int(i), toml::Value::Integer(ni)) => *i = ni.into(),
            (Component::Float(f), toml::Value::Float(nf)) => *f = nf,
            (Component::Bool(b), toml::Value::Boolean(nb)) => *b = nb,

            (Component::OptionString(s), toml::Value::String(ns)) => *s = Some(ns),
            (Component::OptionInt(i), toml::Value::Integer(ni)) => *i = Some(ni.into()),
            (Component::OptionFloat(f), toml::Value::Float(nf)) => *f = Some(nf),
            (Component::OptionBool(b), toml::Value::Boolean(nb)) => *b = Some(nb),

            (Component::ListInt(i), toml::Value::Array(ni)) => {
                *i = ni
                    .into_iter()
                    .map(|v| -> anyhow::Result<i128> {
                        match v {
                            toml::Value::Integer(i) => Ok(i.into()),
                            _ => Err(anyhow!("wrong format")),
                        }
                    })
                    .collect::<Result<_, _>>()?;
            }
            (Component::ListFloat(i), toml::Value::Array(ni)) => {
                *i = ni
                    .into_iter()
                    .map(|v| -> anyhow::Result<f64> {
                        match v {
                            toml::Value::Float(f) => Ok(f),
                            _ => Err(anyhow!("wrong format")),
                        }
                    })
                    .collect::<Result<_, _>>()?;
            }
            (Component::ListBool(i), toml::Value::Array(ni)) => {
                *i = ni
                    .into_iter()
                    .map(|v| -> anyhow::Result<bool> {
                        match v {
                            toml::Value::Boolean(b) => Ok(b),
                            _ => Err(anyhow!("wrong format")),
                        }
                    })
                    .collect::<Result<_, _>>()?;
            }
            (Component::ListString(i), toml::Value::Array(ni)) => {
                *i = ni
                    .into_iter()
                    .map(|v| -> anyhow::Result<String> {
                        match v {
                            toml::Value::String(s) => Ok(s),
                            _ => Err(anyhow!("wrong format")),
                        }
                    })
                    .collect::<Result<_, _>>()?;
            }

            (Component::Struct(map), toml::Value::Table(t)) => {
                for (name, value) in t {
                    if let Some(comp) = map.get_mut(&name) {
                        comp.set_from_value(value)
                            .with_context(|| format!("in field `{name}`"))?;
                    }
                }
            }
            (expected, got) => {
                return Err(anyhow!(
                    "wrong format. Expected {}, got {}",
                    expected.type_str(),
                    got.type_str()
                ))
            }
        }
        Ok(())
    }

    /// Test if the `self` matches the given `value` form proto. Return `Ok` if they do, and an
    // error with an explanation if they don't.
    fn eq_proto_value(&self, value: &prost_types::Value) -> anyhow::Result<()> {
        let Some(k) = &value.kind else {
            bail!("invalid value");
        };

        use prost_types::value::Kind as K;
        use Component as C;
        match (self, k) {
            (C::Int(s), K::NumberValue(v)) => {
                if *s != v.round() as i128 {
                    bail!("got {} expected {s}", v.round() as i128);
                }
            }
            (C::Float(s), K::NumberValue(v)) => {
                if (s - v).abs() >= 0.00001 {
                    bail!("got {v} expected {s}");
                }
            }
            (C::Bool(s), K::BoolValue(v)) => {
                if s != v {
                    bail!("got {v} expected {s}");
                }
            }
            (C::String(s), K::StringValue(v)) => {
                if s != v {
                    bail!("got \"{v}\" expected \"{s}\"");
                }
            }
            (C::Struct(s), K::StructValue(v)) => {
                for (name, comp) in s {
                    let Some(got_comp) = v.fields.get(name) else {
                        bail!("expected a field with name `{}`", name);
                    };
                    comp.eq_proto_value(got_comp)
                        .with_context(|| format!("in field `{name}`"))?;
                }
            }
            (
                C::OptionInt(_)
                | C::OptionFloat(_)
                | C::OptionString(_)
                | C::OptionBool(_)
                | C::ListInt(_)
                | C::ListFloat(_)
                | C::ListString(_)
                | C::ListBool(_),
                _,
            ) => bail!("unsuported field type: {}", self.type_str()),
            _ => {
                bail!("unexpected type");
            }
        };
        Ok(())
    }

    /// Sets the field specified by the `path` with a value parsed from `record`.
    /// Errors if the value can't be parsed. The path should have been checked by a
    /// [`ComponentFillChecker`] for better error messages first.
    fn set(&mut self, path: &[String], record: &str) -> anyhow::Result<()> {
        match path.split_first() {
            Some((name, path)) => {
                let Component::Struct(map) = self else {
                    bail!("ComponentFillChecker should have made shure paths are correct");
                };
                let Some(c) = map.get_mut(name) else {
                    bail!("ComponentFillChecker should have made shure paths are correct");
                };

                c.set(path, record)
                    .with_context(|| format!("in field `{name}`"))?;
            }
            None => match self {
                Component::Int(v) => *v = record.parse()?,
                Component::Float(v) => *v = record.parse()?,
                Component::String(v) => *v = record.parse()?,
                Component::Bool(v) => *v = record.parse()?,
                Component::OptionInt(_)
                | Component::OptionFloat(_)
                | Component::OptionString(_)
                | Component::OptionBool(_)
                | Component::ListInt(_)
                | Component::ListFloat(_)
                | Component::ListString(_)
                | Component::ListBool(_) => bail!("unsuported field type: {}", self.type_str()),
                Component::Struct(_) => {
                    bail!("ComponentFillChecker should have made shure paths are correct");
                }
            },
        }
        Ok(())
    }

    /// Convert this component into a proto value.
    ///
    /// # Panics
    /// Panics on Optional or List values
    fn into_proto_state(self) -> prost_types::Value {
        use prost_types::value::Kind as K;
        let k = match self {
            Component::Int(i) => K::NumberValue(i as f64),
            Component::Float(f) => K::NumberValue(f),
            Component::String(s) => K::StringValue(s),
            Component::Bool(b) => K::BoolValue(b),
            Component::OptionInt(_) => todo!(),
            Component::OptionFloat(_) => todo!(),
            Component::OptionString(_) => todo!(),
            Component::OptionBool(_) => todo!(),
            Component::ListInt(_) => todo!(),
            Component::ListFloat(_) => todo!(),
            Component::ListString(_) => todo!(),
            Component::ListBool(_) => todo!(),
            Component::Struct(s) => {
                let fields = s
                    .into_iter()
                    .map(|(name, comp)| (name, comp.into_proto_state()))
                    .collect();
                K::StructValue(prost_types::Struct { fields })
            }
        };
        prost_types::Value { kind: Some(k) }
    }
}

/// Can be used to check/measure what fields of a component are set by a set of path/proto values.
#[derive(Debug)]
pub(crate) enum ComponentFillChecker {
    Value(bool),
    Struct(HashMap<String, ComponentFillChecker>),
}

impl ComponentFillChecker {
    /// Mark a new path as set. Returns an error if the path is invalid.
    fn add(&mut self, path: &[String]) -> anyhow::Result<()> {
        match self {
            ComponentFillChecker::Value(v) => {
                if path.is_empty() {
                    if *v {
                        bail!("specified twice");
                    }
                    *v = true;
                } else {
                    bail!("not a struct");
                }
            }
            ComponentFillChecker::Struct(map) => match path.split_first() {
                Some((name, path)) => match map.get_mut(name) {
                    Some(c) => c.add(path).with_context(|| format!("in field `{name}`"))?,
                    None => return Err(anyhow!("no field `{name}`")),
                },
                None => return Err(anyhow!("is a struct")),
            },
        }
        Ok(())
    }

    /// Marks all the fields that exist in the value as set, returning an error if the value is
    /// not a subset of self.
    fn set_from_value(&mut self, value: &toml::Value) -> anyhow::Result<()> {
        match (self, value) {
            (ComponentFillChecker::Value(_), toml::Value::Table(_)) => {
                Err(anyhow!("expected value"))
            }
            (ComponentFillChecker::Value(v), _) => {
                *v = true;
                Ok(())
            }
            (ComponentFillChecker::Struct(map), toml::Value::Table(v)) => {
                for (name, value) in v {
                    let Some(comp) = map.get_mut(name) else {
                        bail!("no field with name `{name}`");
                    };
                    comp.set_from_value(value)
                        .with_context(|| format!("in filed `{name}`"))?;
                }
                Ok(())
            }
            (ComponentFillChecker::Struct(_), _) => Err(anyhow!("expected struct")),
        }
    }

    /// Are `Ok` if all fields are marked as set. The error otherwise has a human readable value to
    /// the field that has not been set.
    fn is_filled(&self) -> anyhow::Result<()> {
        match self {
            ComponentFillChecker::Value(true) => Ok(()),
            ComponentFillChecker::Value(false) => Err(anyhow!("not set")),
            ComponentFillChecker::Struct(map) => {
                for (name, c) in map {
                    c.is_filled()
                        .with_context(|| format!("in field `{name}`"))?;
                }
                Ok(())
            }
        }
    }

    /// Returns all paths needed to fill this component.
    /// Warning: Paths are reversed order as an optimization.
    fn paths(&self) -> Box<dyn Iterator<Item = Vec<String>> + '_> {
        match self {
            ComponentFillChecker::Value(_) => Box::new(std::iter::once(Vec::new())),
            ComponentFillChecker::Struct(s) => Box::new(s.iter().flat_map(|(name, comp)| {
                comp.paths().map(|mut p| {
                    p.push(name.clone());
                    p
                })
            })),
        }
    }
}

/// Fill checker for a whole frame of simulation. See [`ComponentFillChecker`] for mere information
/// what this is used fore.
#[derive(Debug)]
pub(crate) struct StateFillChecker {
    global: HashMap<String, ComponentFillChecker>,
    nodes: HashMap<u64, HashMap<String, ComponentFillChecker>>,
    edges: HashMap<u64, (String, ComponentFillChecker)>,
}

impl StateFillChecker {
    /// Mark a new path as set. Returns an error if the path is invalid.
    pub(crate) fn add(&mut self, path: &[String]) -> anyhow::Result<()> {
        let mut path = path.iter();
        let path_start = parse_path_start(&mut path)?;

        let name = path_start.get_name();
        let comp = match path_start {
            PathStart::Global(n) => self
                .global
                .get_mut(n)
                .with_context(|| format!("no global component `{n}`"))?,

            PathStart::Node(idx, n) => self
                .nodes
                .get_mut(&idx)
                .with_context(|| format!("no node with id {idx}"))?
                .get_mut(n)
                .with_context(|| format!("no component `{n}` in node {idx}"))?,

            PathStart::Edge(idx) => {
                &mut self
                    .edges
                    .get_mut(&idx)
                    .with_context(|| format!("no edge with index {idx}"))?
                    .1
            }
        };

        comp.add(path.as_slice())
            .with_context(|| format!("in {name}"))
    }

    /// Marks all the fields that exist in the value as set, returning an error if the value is
    /// not a subset of self.
    pub(crate) fn set_from_config(
        &mut self,
        initial_state: &super::TestConfigInitialState,
    ) -> anyhow::Result<()> {
        for (name, value) in initial_state.global.iter() {
            let Some(comp) = self.global.get_mut(name) else {
                bail!("no component with name `{name}`");
            };
            comp.set_from_value(value)
                .with_context(|| format!("in component `{name}`"))?;
        }

        for config_node in &initial_state.nodes {
            let node = self
                .nodes
                .get_mut(&config_node.id)
                .with_context(|| format!("no node with id {}", config_node.id))?;

            for (name, value) in config_node.components.iter().flatten() {
                let Some(comp) = node.get_mut(name) else {
                    bail!(
                        "no component with name `{name}`, in node {}",
                        config_node.id
                    );
                };
                comp.set_from_value(value).with_context(|| {
                    format!("in component `{name}`, in node {}", config_node.id)
                })?;
            }
        }
        for (id, config_edge) in initial_state.edges.iter().enumerate() {
            let edge = &mut self
                .edges
                .get_mut(&(id as u64))
                .with_context(|| format!("no edge with id {}", id))?
                .1;

            edge.set_from_value(&config_edge.component_data)
                .with_context(|| format!("in edge {}", id))?;
        }
        Ok(())
    }

    /// Are `Ok` if all fields in all components are marked as set. The error otherwise has a human
    /// readable value to the field that has not been set.
    pub(crate) fn is_filled(&self) -> anyhow::Result<()> {
        for (name, c) in &self.global {
            c.is_filled()
                .with_context(|| format!("in global component `{name}`"))?;
        }
        for (id, n) in &self.nodes {
            for (name, c) in n {
                c.is_filled()
                    .with_context(|| format!("in node {id}, component `{name}`"))?;
            }
        }
        for (id, (_, c)) in &self.edges {
            c.is_filled().with_context(|| format!("in edge {id}"))?;
        }
        Ok(())
    }

    /// Returns all paths needed to fill this state.
    /// Warning: Paths are reversed order as an optimization.
    pub(crate) fn paths(&self) -> impl Iterator<Item = String> + '_ {
        let globals = self.global.iter().flat_map(|(name, comp)| {
            comp.paths().map(|mut p| {
                p.push(name.clone());
                p.push("global".to_owned());
                p
            })
        });

        let nodes = self.nodes.iter().flat_map(|(id, comps)| {
            comps
                .iter()
                .flat_map(|(name, comp)| {
                    comp.paths().map(|mut p| {
                        p.push(name.clone());
                        p
                    })
                })
                .map(|mut p| {
                    p.push(id.to_string());
                    p.push("node".to_owned());
                    p
                })
        });

        let edges = self.edges.iter().flat_map(|(id, (_, comp))| {
            comp.paths().map(|mut p| {
                p.push(id.to_string());
                p.push("edge".to_owned());
                p
            })
        });

        globals.chain(nodes).chain(edges).map(|rev_path| {
            let path: Vec<_> = rev_path.into_iter().rev().collect();
            path.join(".")
        })
    }
}

/// Single node
#[derive(Debug, Clone)]
pub(crate) struct Node {
    longitude: f64,
    latitude: f64,
    id: u64,
    components: HashMap<String, Component>,
}

/// Single edge
#[derive(Debug, Clone)]
pub(crate) struct Edge {
    from: u64,
    to: u64,
    id: u64,
    component_name: String,
    component_data: Component,
}

/// Frame of simulation
#[derive(Debug, Clone)]
pub(crate) struct State {
    pub(crate) global: HashMap<String, Component>,
    pub(crate) nodes: HashMap<u64, Node>,
    pub(crate) edges: HashMap<u64, Edge>,
}

impl State {
    /// Sets the field specified by the `path` with a value parsed from `record`.
    /// Errors if the value can't be parsed. The path should have been checked by a
    /// [`ComponentFillChecker`] for better error messages first.
    pub(crate) fn set(&mut self, path: &[String], record: &str) -> anyhow::Result<()> {
        let mut path = path.iter();
        let path_start = parse_path_start(&mut path)?;

        let name = path_start.get_name();
        let comp = match path_start {
            PathStart::Global(n) => self
                .global
                .get_mut(n)
                .with_context(|| format!("no global component `{n}`"))?,

            PathStart::Node(idx, n) => self
                .nodes
                .get_mut(&idx)
                .with_context(|| format!("no node with id {idx}"))?
                .components
                .get_mut(n)
                .with_context(|| format!("no component `{n}` in node {idx}"))?,

            PathStart::Edge(idx) => {
                &mut self
                    .edges
                    .get_mut(&idx)
                    .with_context(|| format!("no edge with index {idx}"))?
                    .component_data
            }
        };

        comp.set(path.as_slice(), record)
            .context(format!("in component `{name}`"))
    }

    /// Sets values using the values in the config. Ignoring any components/fields that do not
    /// exist.
    ///
    /// Use the fillchecker if you want to make sure the fields match.
    pub(crate) fn set_from_config(
        &mut self,
        initial_state: super::TestConfigInitialState,
    ) -> anyhow::Result<()> {
        for (name, value) in initial_state.global {
            if let Some(comp) = self.global.get_mut(&name) {
                comp.set_from_value(value)
                    .with_context(|| format!("in component `{name}`"))?;
            }
        }

        for config_node in initial_state.nodes {
            let Some(node) = self.nodes.get_mut(&config_node.id) else {
                continue;
            };

            for (name, value) in config_node.components.into_iter().flatten() {
                if let Some(comp) = node.components.get_mut(&name) {
                    comp.set_from_value(value).with_context(|| {
                        format!("in component `{name}`, in node {}", config_node.id)
                    })?;
                }
            }
        }
        for (id, config_edge) in initial_state.edges.into_iter().enumerate() {
            if let Some(edge) = self.edges.get_mut(&(id as u64)) {
                edge.component_data
                    .set_from_value(config_edge.component_data)
                    .with_context(|| format!("in edge {}", id))?;
            }
        }
        Ok(())
    }

    /// Remove all fields except those that are marked as filled in the fill_checker
    pub(crate) fn keep_filled(&mut self, fill_checker: &StateFillChecker) {
        self.global = self
            .global
            .drain()
            .filter_map(|(name, mut comp)| {
                let fill_comp = fill_checker.global.get(&name)?;
                if !comp.keep_filled(fill_comp) {
                    return None;
                }
                Some((name, comp))
            })
            .collect();

        self.nodes = self
            .nodes
            .drain()
            .filter_map(|(name, mut node)| {
                let fill_node = fill_checker.nodes.get(&node.id)?;

                node.components = node
                    .components
                    .drain()
                    .filter_map(|(name, mut comp)| {
                        let fill_comp = fill_node.get(&name)?;
                        if !comp.keep_filled(fill_comp) {
                            return None;
                        }
                        Some((name, comp))
                    })
                    .collect();
                if node.components.is_empty() {
                    return None;
                }

                Some((name, node))
            })
            .collect();

        self.edges = self
            .edges
            .drain()
            .filter_map(|(name, mut edge)| {
                let fill_edge = fill_checker.edges.get(&edge.id)?;
                if !edge.component_data.keep_filled(&fill_edge.1) {
                    return None;
                }
                Some((name, edge))
            })
            .collect();
    }

    /// Turn this state into a proto state.
    pub(crate) fn into_proto_state(self) -> proto::simulation::State {
        proto::simulation::State {
            graph: Some(proto::simulation::Graph {
                nodes: self
                    .nodes
                    .into_values()
                    .map(|n| proto::simulation::Node {
                        longitude: n.longitude,
                        latitude: n.latitude,
                        components: n
                            .components
                            .into_iter()
                            .map(|(n, c)| (n, c.into_proto_state()))
                            .collect(),
                        id: n.id,
                    })
                    .collect(),
                edge: self
                    .edges
                    .into_values()
                    .map(|e| proto::simulation::Edge {
                        from: e.from,
                        to: e.to,
                        component_type: e.component_name,
                        component_data: Some(e.component_data.into_proto_state()),
                        id: e.id,
                    })
                    .collect(),
            }),
            global_components: self
                .global
                .into_iter()
                .map(|(name, comp)| (name, comp.into_proto_state()))
                .collect(),
        }
    }

    /// Test if the `self` matches the given `value` form proto. Return `Ok` if they do, and an
    // error with an explanation if they don't.
    pub(crate) fn eq_proto_state(&self, got: proto::simulation::State) -> anyhow::Result<()> {
        for (name, comp) in &self.global {
            let Some(got_comp) = got.global_components.get(name) else {
                bail!("Expected a global components with name `{}`", name);
            };
            comp.eq_proto_value(got_comp)
                .with_context(|| format!("in component `{name}`"))?;
        }

        for (id, node) in &self.nodes {
            let Some(got_node) = got
                .graph
                .as_ref()
                .context("bad proto")?
                .nodes
                .iter()
                .find(|gn| gn.id == *id)
            else {
                bail!("Expected a node with id `{}`", id);
            };

            for (name, comp) in &node.components {
                let Some(got_comp) = got_node.components.get(name) else {
                    bail!(
                        "Expected components with name `{}` in node with id {id}",
                        name
                    );
                };
                comp.eq_proto_value(got_comp)
                    .with_context(|| format!("in component `{name}` in node with id {id}"))?;
            }
        }

        for (id, edge) in &self.edges {
            let Some(got_edge) = got
                .graph
                .as_ref()
                .context("bad proto")?
                .edge
                .iter()
                .find(|gn| gn.id == *id)
            else {
                bail!("Expected a edge with id `{}`", id);
            };

            if edge.component_name != got_edge.component_type {
                bail!(
                    "edge {} component names don't match.\nExpected: `{}`\n     Got: `{}`",
                    id,
                    edge.component_name,
                    got_edge.component_type
                );
            }
            edge.component_data
                .eq_proto_value(got_edge.component_data.as_ref().context("bad proto")?)
                .with_context(|| format!("in edge with id {id}"))?;
        }

        Ok(())
    }
}

/// Creates a [`ComponentFillChecker`] and a [`Component`] from a config [`ComponentStructure`].
/// If preset_options is set all optional fields will be marked as set from the start.
fn from_structure(
    structure: &ComponentStructure,
    preset_options: bool,
) -> anyhow::Result<(ComponentFillChecker, Component)> {
    use ComponentStructure as C;
    match structure {
        C::U8 | C::U16 | C::U32 | C::U64 | C::U128 | C::I8 | C::I16 | C::I32 | C::I64 | C::I128 => {
            Ok((ComponentFillChecker::Value(false), Component::Int(0)))
        }
        C::F32 | C::F64 => Ok((ComponentFillChecker::Value(false), Component::Float(0.0))),
        C::String => Ok((
            ComponentFillChecker::Value(false),
            Component::String(String::new()),
        )),
        C::Bool => Ok((ComponentFillChecker::Value(false), Component::Bool(false))),
        C::List(i) => {
            let i = match **i {
                C::U8
                | C::U16
                | C::U32
                | C::U64
                | C::U128
                | C::I8
                | C::I16
                | C::I32
                | C::I64
                | C::I128 => Component::ListInt(Vec::new()),
                C::Bool => Component::ListBool(Vec::new()),
                C::String => Component::ListString(Vec::new()),
                C::F32 | C::F64 => Component::ListFloat(Vec::new()),
                C::Option(_) => return Err(anyhow!("lists of options is not suported atm")),
                C::List(_) => return Err(anyhow!("lists of lists is not suported atm")),
                C::Struct(_) => return Err(anyhow!("lists of structs is not suported atm")),
            };
            Ok((ComponentFillChecker::Value(false), i))
        }
        C::Option(i) => {
            let i = match **i {
                C::U8
                | C::U16
                | C::U32
                | C::U64
                | C::U128
                | C::I8
                | C::I16
                | C::I32
                | C::I64
                | C::I128 => Component::OptionInt(None),
                C::Bool => Component::OptionBool(None),
                C::String => Component::OptionString(None),
                C::F32 | C::F64 => Component::OptionFloat(None),
                C::Option(_) => return Err(anyhow!("option of option is not suported atm")),
                C::List(_) => return Err(anyhow!("option of list is not suported atm")),
                C::Struct(_) => return Err(anyhow!("option of structs is not suported atm")),
            };
            Ok((ComponentFillChecker::Value(preset_options), i))
        }
        C::Struct(map) => {
            let (fillers, comps) = map
                .iter()
                .map(|(name, s)| -> anyhow::Result<_> {
                    let (filler, comp) =
                        from_structure(s, preset_options).context(format!("in field `{name}`"))?;
                    Ok(((name.clone(), filler), (name.clone(), comp)))
                })
                .collect::<anyhow::Result<Vec<_>>>()?
                .into_iter()
                .unzip();

            Ok((
                ComponentFillChecker::Struct(fillers),
                Component::Struct(comps),
            ))
        }
    }
}

/// Creates a [`ComponentFillChecker`] and a [`Component`]. Uses the `expected_components` the get
/// the shape of the components. `initial_state` is used to decide what nodes and edges should exist
/// and what components they should have, but not the shape of the components. Any component name
/// in `filter_components` will not be included in the final output. Lastly if preset_options is
/// true all optional fields will be marked as set from the start.
pub(crate) fn from_file_config(
    expected_components: &HashMap<String, ComponentInfo>,
    initial_state: &TestConfigInitialState,
    filter_components: &HashSet<String>,
    preset_options: bool,
) -> anyhow::Result<(StateFillChecker, State)> {
    let (global_fillers, global_comps) = initial_state
        .global
        .keys()
        .filter(|name| !filter_components.contains(*name))
        .map(|name| {
            let comp = expected_components
                .get(name)
                .context("node component has no coresponding expected component")?;
            let (filler, comps) = from_structure(&comp.structure, preset_options)
                .with_context(|| format!("in component `{name}`"))?;

            Ok(((name.clone(), filler), (name.clone(), comps)))
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .unzip();

    let (node_fillers, node_comps) = initial_state
        .nodes
        .iter()
        .map(|n| {
            (
                (n.id, n.longitude, n.latitude),
                n.components.iter().flat_map(|c| c.keys()),
            )
        })
        .map(
            |((id, longitude, latitude), components)| -> anyhow::Result<_> {
                let (fillers, comps): (HashMap<_, _>, HashMap<_, _>) = components
                    .filter(|name| !filter_components.contains(*name))
                    .map(|name| -> anyhow::Result<_> {
                        let comp = expected_components
                            .get(name)
                            .context("node component has no coresponding expected component")?;
                        let (filler, comps) = from_structure(&comp.structure, preset_options)
                            .with_context(|| format!("in component `{name}`"))?;

                        Ok(((name.clone(), filler), (name.clone(), comps)))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()
                    .with_context(|| format!("in node {id}"))?
                    .into_iter()
                    .unzip();

                Ok((
                    (id, fillers),
                    (
                        id,
                        Node {
                            longitude,
                            latitude,
                            id,
                            components: comps,
                        },
                    ),
                ))
            },
        )
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .unzip();

    let (edge_fillers, edge_comps) = initial_state
        .edges
        .iter()
        .enumerate()
        .filter(|(_, e)| !filter_components.contains(&e.component_type))
        .map(|(id, e)| -> anyhow::Result<_> {
            let id = id as u64;

            let comp = expected_components
                .get(&e.component_type)
                .context("edge component has no coresponding expected component")?;
            let (filler, comp) = from_structure(&comp.structure, preset_options)
                .with_context(|| format!("in component `{}`", &e.component_type))?;

            Ok((
                (id, (e.component_type.clone(), filler)),
                (
                    id,
                    Edge {
                        from: e.from,
                        to: e.to,
                        id,
                        component_name: e.component_type.clone(),
                        component_data: comp,
                    },
                ),
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .unzip();

    Ok((
        StateFillChecker {
            global: global_fillers,
            nodes: node_fillers,
            edges: edge_fillers,
        },
        State {
            global: global_comps,
            nodes: node_comps,
            edges: edge_comps,
        },
    ))
}
