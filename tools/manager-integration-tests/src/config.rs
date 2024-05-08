#![allow(clippy::match_like_matches_macro)]

mod dyn_comp;
mod state_helpers;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{bail, Context};
use serde::Deserialize;
use tracing::info;

use self::{dyn_comp::State, state_helpers::get_from_state_as_str};

/// The type of a component. ie. Node, Edge or Global
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ComponentType {
    Node,
    Edge,
    Global,
}

impl ComponentType {
    /// Compare with the proto version of this type.
    fn eq_proto(&self, other: &proto::simulation::ComponentType) -> bool {
        use proto::simulation::ComponentType as CT;
        match (self, other) {
            (ComponentType::Node, CT::Node)
            | (ComponentType::Edge, CT::Edge)
            | (ComponentType::Global, CT::Global) => true,
            _ => false,
        }
    }

    /// Convert into the proto version of this type.
    fn to_proto(&self) -> proto::simulation::ComponentType {
        use proto::simulation::ComponentType as CT;
        match self {
            ComponentType::Node => CT::Node,
            ComponentType::Edge => CT::Edge,
            ComponentType::Global => CT::Global,
        }
    }
}

/// A component structure, designed to be nice to use in toml
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ComponentStructure {
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    String,
    F32,
    F64,
    #[serde(rename = "Option")]
    Option(Box<ComponentStructure>),
    #[serde(rename = "List")]
    List(Box<ComponentStructure>),
    #[serde(untagged)]
    Struct(HashMap<String, ComponentStructure>),
}

impl ComponentStructure {
    /// Compare with the proto version of the type
    fn eq_proto(&self, other: &proto::simulation::ComponentStructure) -> bool {
        let Some(other) = &other.component_structure else {
            return false;
        };

        use proto::simulation::component_structure::ComponentStructure as CSO;
        use proto::simulation::ComponentPrimitive as CP;
        use ComponentStructure as CSS;
        match (self, other) {
            (CSS::Option(s), CSO::Option(o)) => s.eq_proto(o),
            (CSS::List(s), CSO::List(o)) => s.eq_proto(o),
            (CSS::Struct(s), CSO::Struct(o)) => {
                let o = &o.data;
                if s.len() != o.len() {
                    return false;
                }
                for (name, s) in s {
                    let Some(o) = o.get(name) else {
                        return false;
                    };
                    if !s.eq_proto(o) {
                        return false;
                    }
                }
                true
            }
            (p, CSO::Primitive(o)) => {
                let Ok(o): Result<CP, _> = (*o).try_into() else {
                    return false;
                };
                match (p, o) {
                    (CSS::Bool, CP::Bool)
                    | (CSS::U8, CP::U8)
                    | (CSS::U16, CP::U16)
                    | (CSS::U32, CP::U32)
                    | (CSS::U64, CP::U64)
                    | (CSS::U128, CP::U128)
                    | (CSS::I8, CP::I8)
                    | (CSS::I16, CP::I16)
                    | (CSS::I32, CP::I32)
                    | (CSS::I64, CP::I64)
                    | (CSS::I128, CP::I128)
                    | (CSS::String, CP::String)
                    | (CSS::F32, CP::F32)
                    | (CSS::F64, CP::F64) => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Convert into the proto version of this type.
    fn to_proto(&self) -> proto::simulation::ComponentStructure {
        use proto::simulation::component_structure::ComponentStructure as CS;
        use proto::simulation::ComponentPrimitive as CP;
        let is = match self {
            ComponentStructure::Bool => CS::Primitive(CP::Bool.into()),
            ComponentStructure::U8 => CS::Primitive(CP::U8.into()),
            ComponentStructure::U16 => CS::Primitive(CP::U16.into()),
            ComponentStructure::U32 => CS::Primitive(CP::U32.into()),
            ComponentStructure::U64 => CS::Primitive(CP::U64.into()),
            ComponentStructure::U128 => CS::Primitive(CP::U128.into()),
            ComponentStructure::I8 => CS::Primitive(CP::I8.into()),
            ComponentStructure::I16 => CS::Primitive(CP::I16.into()),
            ComponentStructure::I32 => CS::Primitive(CP::I32.into()),
            ComponentStructure::I64 => CS::Primitive(CP::I64.into()),
            ComponentStructure::I128 => CS::Primitive(CP::I128.into()),
            ComponentStructure::String => CS::Primitive(CP::String.into()),
            ComponentStructure::F32 => CS::Primitive(CP::F32.into()),
            ComponentStructure::F64 => CS::Primitive(CP::F64.into()),
            ComponentStructure::Option(i) => CS::Option(Box::new(i.to_proto())),
            ComponentStructure::List(i) => CS::List(Box::new(i.to_proto())),
            ComponentStructure::Struct(i) => CS::Struct(proto::simulation::ComponentStruct {
                data: i
                    .iter()
                    .map(|(name, comp)| (name.clone(), comp.to_proto()))
                    .collect(),
            }),
        };

        proto::simulation::ComponentStructure {
            component_structure: Some(is),
        }
    }
}

/// Everything you need to know about a Component
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ComponentInfo {
    ty: ComponentType,
    structure: ComponentStructure,
}

impl ComponentInfo {
    /// Compare with the proto version of the type.
    pub(crate) fn eq_proto(&self, other: &proto::simulation::ComponentSpecification) -> bool {
        ({
            let t: &Result<proto::simulation::ComponentType, _> = &other.r#type.try_into();
            match t {
                Ok(t) => self.ty.eq_proto(t),
                Err(_) => false,
            }
        }) && ({
            match &other.structure {
                Some(structure) => self.structure.eq_proto(structure),
                None => false,
            }
        })
    }

    /// Convert into the proto version of this type.
    pub(crate) fn to_proto(&self) -> proto::simulation::ComponentSpecification {
        proto::simulation::ComponentSpecification {
            r#type: self.ty.to_proto().into(),
            structure: Some(self.structure.to_proto()),
        }
    }
}

/// A simulation node definition designed to be nice to parse from toml
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct TestConfigNode {
    longitude: f64,
    latitude: f64,
    id: u64,
    // Wrapping in a vec to improve the toml format, as only arrays can have internal new lines
    components: Vec<HashMap<String, toml::Value>>,
}

/// A simulation edge definition designed to be nice to parse from toml
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct TestConfigEdge {
    from: u64,
    to: u64,
    component_type: String,
    component_data: toml::Value,
}

/// The initial configuration of a simulation designed to be nice to parse from toml
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct TestConfigInitialState {
    global: HashMap<String, toml::Value>,
    nodes: Vec<TestConfigNode>,
    edges: Vec<TestConfigEdge>,
}

/// Settings for the mock simulation designed to be nice to parse from toml
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct MockSimulatorFile {
    output_components: HashSet<String>,
    data_file: PathBuf,
}

/// A single test case designed to be nice to parse from toml
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct TestConfigFile {
    used_simulators: HashSet<String>,
    expected_components: HashMap<String, ComponentInfo>,
    mock_simulator: Option<MockSimulatorFile>,

    #[serde(default)]
    ignored_components: HashSet<String>,

    expected_output_file: PathBuf,
    amount_of_timesteps: u32,
    timestep_delta_seconds: f64,
    initial_state: TestConfigInitialState,
}

impl TestConfigFile {
    /// Reads a list of component data from a csv file
    ///
    /// If a component specified in `filter_components` is given in the file an error is returned.
    /// The output will also not have any components from the filter.
    /// If `fill_with_initial_state` is set, the initial state is used to supply the missing
    /// values. Otherwise only the fields given in the csv file will be included in the output.
    fn read_csv(
        &self,
        path: &Path,
        filter_components: &HashSet<String>,
        fill_with_initial_state: bool,
    ) -> anyhow::Result<Vec<State>> {
        let f = std::fs::OpenOptions::new()
            .read(true)
            .open(path)
            .with_context(|| format!("error opening `{}`", path.display()))?;

        let mut file_reader = csv::Reader::from_reader(f);

        let (mut fill_checker, mut components) = dyn_comp::from_file_config(
            &self.expected_components,
            &self.initial_state,
            filter_components,
            false,
        )?;

        let mut paths = Vec::new();
        for (i, col) in file_reader
            .headers()
            .context("need a header with paths to component fields")?
            .iter()
            .enumerate()
        {
            let path = col
                .trim()
                .split('.')
                .map(|s| s.to_owned())
                .collect::<Vec<_>>();
            fill_checker
                .add(&path)
                .with_context(|| format!("in header {i}"))?;
            paths.push(path);
        }

        if fill_with_initial_state {
            components.set_from_config(self.initial_state.clone())?;
        } else {
            components.keep_filled(&fill_checker);
        }

        let mut steps = Vec::new();
        for (i, record) in file_reader.records().enumerate() {
            let record = record.with_context(|| format!("in row {}", i + 1))?;

            let mut components = components.clone();

            for (path, record) in paths.iter().zip(record.iter()) {
                components
                    .set(path, record.trim())
                    .with_context(|| format!("in row {i}"))?;
            }

            steps.push(components);
        }

        Ok(steps)
    }

    /// Converts into a [`TestConfig`].
    ///
    /// This will also reads any csv files necessary using the `base_path` as a root path.
    pub(crate) fn into_config(self, base_path: &Path) -> anyhow::Result<TestConfig> {
        let mut expected_output_file_path = base_path.to_path_buf();
        expected_output_file_path.extend(&self.expected_output_file);
        let expected_output_frames = self
            .read_csv(&expected_output_file_path, &HashSet::new(), false)
            .context("error while reading expected output file")?;

        let mock_simulator = match &self.mock_simulator {
            Some(mock_simulator) => {
                // Filter out all components except for the ones specified as needing to be mocked.
                let filter_components = self
                    .expected_components
                    .keys()
                    .filter(|name| !mock_simulator.output_components.contains(*name))
                    .cloned()
                    .collect();

                let mut mock_data_file_path = base_path.to_path_buf();
                mock_data_file_path.extend(&mock_simulator.data_file);
                let mock_data = self
                    .read_csv(&mock_data_file_path, &filter_components, true)
                    .context("error while reading mock data file")?;

                if mock_data.len() != self.amount_of_timesteps as usize {
                    bail!(
                        "Mock data has {} frames but there are {} timesteps",
                        mock_data.len(),
                        self.amount_of_timesteps
                    );
                }

                let output_components = mock_simulator
                    .output_components
                    .iter()
                    .map(|name| -> anyhow::Result<_> {
                        let comp = self
                            .expected_components
                            .get(name)
                            .with_context(|| {
                                format!("component `{name}` not found in expected components")
                            })?
                            .clone();
                        Ok((name.clone(), comp))
                    })
                    .collect::<Result<_, _>>()
                    .context("in mock simulation")?;

                Some(MockSimulatorConfig {
                    output_components,
                    data: mock_data,
                })
            }
            None => None,
        };

        let (mut filler, mut components) = dyn_comp::from_file_config(
            &self.expected_components,
            &self.initial_state,
            &HashSet::new(),
            true,
        )?;
        filler.set_from_config(&self.initial_state)?;
        filler.is_filled()?;
        components.set_from_config(self.initial_state)?;

        // Build the CSV reader and iterate over each record.
        Ok(TestConfig {
            used_simulators: self.used_simulators,
            expected_components: self.expected_components,
            ignored_components: self.ignored_components,
            mock_simulator,
            expected_output_frames,
            amount_of_timesteps: self.amount_of_timesteps,
            timestep_delta: Duration::from_secs_f64(self.timestep_delta_seconds),
            initial_state: components,
        })
    }

    /// Writes the given `frames` into the csv file specified in `self`.
    pub(crate) fn write_frames(
        &self,
        base_path: &Path,
        frames: Vec<proto::simulation::State>,
    ) -> anyhow::Result<()> {
        let mut expected_output_file_path = base_path.to_path_buf();
        expected_output_file_path.extend(&self.expected_output_file);

        let mut file_reader = csv::Reader::from_path(&expected_output_file_path)
            .context("opening expected output file")?;

        let mut paths = Vec::new();
        for col in file_reader
            .headers()
            .context("need a header with paths to component fields")?
            .iter()
        {
            let path = col
                .trim()
                .split('.')
                .map(|s| s.to_owned())
                .collect::<Vec<_>>();
            paths.push(path);
        }
        // Dropping file reader to close file descriptor
        drop(file_reader);

        let mut file_writer = csv::Writer::from_path(&expected_output_file_path)
            .context("opening expected output file")?;

        file_writer.write_record(paths.iter().map(|path| path.join(".")))?;

        for frame in frames {
            file_writer.write_record(
                paths
                    .iter()
                    .map(|path| get_from_state_as_str(&frame, path))
                    .collect::<Result<Vec<_>, _>>()?,
            )?;
        }

        Ok(())
    }

    /// Writes a csv file to the expected output file with all the paths that have to be set
    pub(crate) fn generate_output_file(&self, base_path: &Path) -> anyhow::Result<()> {
        let mut expected_output_file_path = base_path.to_path_buf();
        expected_output_file_path.extend(&self.expected_output_file);

        let (filler, _) = dyn_comp::from_file_config(
            &self.expected_components,
            &self.initial_state,
            &HashSet::new(),
            false,
        )?;

        let mut wrt = csv::Writer::from_path(&expected_output_file_path)
            .context("error while opening expected output file")?;
        wrt.write_record(filler.paths())
            .context("writing to file")?;

        info!("Wrote to {}", expected_output_file_path.display());

        Ok(())
    }
}

/// A mock simulator config with the frames read from the csv.
#[derive(Debug, Clone)]
pub(crate) struct MockSimulatorConfig {
    pub(crate) output_components: HashMap<String, ComponentInfo>,
    pub(crate) data: Vec<State>,
}

/// A single test case designed to be easy to use in the test runner.
#[derive(Debug, Clone)]
pub(crate) struct TestConfig {
    pub(crate) used_simulators: HashSet<String>,
    pub(crate) expected_components: HashMap<String, ComponentInfo>,
    pub(crate) ignored_components: HashSet<String>,
    pub(crate) expected_output_frames: Vec<State>,
    pub(crate) mock_simulator: Option<MockSimulatorConfig>,
    pub(crate) amount_of_timesteps: u32,
    pub(crate) timestep_delta: Duration,
    pub(crate) initial_state: State,
}
