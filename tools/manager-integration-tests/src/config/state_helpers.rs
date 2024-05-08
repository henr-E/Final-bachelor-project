use anyhow::{anyhow, bail, Context};
use proto::simulation::State;

/// The start of a parsed path
pub(crate) enum PathStart<'a> {
    /// A path to a global component with a name.
    Global(&'a str),
    /// A path to a component with an id from the first value, an a component name from the second.
    Node(u64, &'a str),
    /// A path to an edge with the given id.
    Edge(u64),
}

impl<'a> PathStart<'a> {
    /// Get a human readable name for a the thing the path references.
    pub(crate) fn get_name(&self) -> String {
        match self {
            PathStart::Global(n) => format!("global component `{n}`"),
            PathStart::Node(idx, n) => format!("node id {idx}, component `{n}`"),
            PathStart::Edge(idx) => format!("edge index {idx}"),
        }
    }
}

/// Parse the first few items from the iterator into a PathStart. If this return an error the
/// iterator is left in an unknown state.
pub(crate) fn parse_path_start<'a, 'b>(
    path: &'b mut impl Iterator<Item = &'a String>,
) -> anyhow::Result<PathStart<'a>> {
    let Some(ty) = path.next() else {
        bail!("expected path")
    };
    match ty.as_str() {
        "" => Err(anyhow!("expected path")),
        "global" => {
            let component_name = path.next().context("expected global component name")?;
            Ok(PathStart::Global(component_name))
        }
        "node" => {
            let idx = path.next().context("expected node index")?;
            let idx = idx.parse().context("node index not a number")?;
            let component_name = path.next().context("expected global component name")?;
            Ok(PathStart::Node(idx, component_name))
        }
        "edge" => {
            let idx = path.next().context("expected edge index")?;
            let idx = idx.parse().context("edge index not a number")?;
            Ok(PathStart::Edge(idx))
        }
        _ => Err(anyhow!(
            "first part of path needs to be `g`, `node`, or `edge`"
        )),
    }
}

/// Follow the path over the fields of the value, and return the final field as a string.
fn get_filed_form_value_as_str(
    value: &prost_types::Value,
    path: &[String],
) -> Result<String, anyhow::Error> {
    let value = value.kind.as_ref().context("invalid value")?;

    match path.split_first() {
        Some((field, rest_of_path)) => {
            let prost_types::value::Kind::StructValue(prost_types::Struct { fields }) = value
            else {
                bail!("invalid value")
            };

            let value = fields.get(field).context("invalid value")?;

            get_filed_form_value_as_str(value, rest_of_path)
        }
        None => match value {
            prost_types::value::Kind::NullValue(_) => Ok(String::new()),
            prost_types::value::Kind::NumberValue(v) => Ok(v.to_string()),
            prost_types::value::Kind::StringValue(v) => Ok(v.to_string()),
            prost_types::value::Kind::BoolValue(v) => Ok(v.to_string()),
            prost_types::value::Kind::ListValue(_) => todo!(),
            prost_types::value::Kind::StructValue(_) => Err(anyhow!("invalid value")),
        },
    }
}

/// Follow the path over graph and the fields of the component, and return the final field as a
/// string.
pub(crate) fn get_from_state_as_str(state: &State, path: &[String]) -> anyhow::Result<String> {
    let mut path = path.iter();
    let path_start = parse_path_start(&mut path)?;

    let comp = match path_start {
        PathStart::Global(n) => state
            .global_components
            .get(n)
            .with_context(|| format!("no global component `{n}`"))?,

        PathStart::Node(idx, n) => state
            .graph
            .as_ref()
            .context("invalid state")?
            .nodes
            .iter()
            .find(|n| n.id == idx)
            .with_context(|| format!("no node with id {idx}"))?
            .components
            .get(n)
            .with_context(|| format!("no component `{n}` in node {idx}"))?,

        PathStart::Edge(idx) => state
            .graph
            .as_ref()
            .context("invalid state")?
            .edge
            .iter()
            .find(|e| e.id == idx)
            .with_context(|| format!("no edge with index {idx}"))?
            .component_data
            .as_ref()
            .context("invalid state")?,
    };

    get_filed_form_value_as_str(comp, path.as_slice())
}
