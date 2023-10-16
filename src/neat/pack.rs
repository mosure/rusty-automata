use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d,
        TextureDimension,
        TextureFormat,
        TextureUsages,
    },
};
use pyo3::prelude::*;


#[pyclass]
#[derive(Clone)]
pub struct NeatUafActivation {
    #[pyo3(get)]
    pub a: f32,
    #[pyo3(get)]
    pub b: f32,
    #[pyo3(get)]
    pub c: f32,
    #[pyo3(get)]
    pub d: f32,
    #[pyo3(get)]
    pub e: f32,
}
#[pymethods]
impl NeatUafActivation {
    #[new]
    fn new(
        a: f32,
        b: f32,
        c: f32,
        d: f32,
        e: f32,
    ) -> Self {
        Self {
            a,
            b,
            c,
            d,
            e,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct NeatEdge {
    #[pyo3(get)]
    pub weight: f32,
    #[pyo3(get)]
    pub source: usize,
}
#[pymethods]
impl NeatEdge {
    #[new]
    fn new(
        weight: f32,
        source: usize,
    ) -> Self {
        Self {
            weight,
            source,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct NeatNode {
    #[pyo3(get)]
    pub activation: NeatUafActivation,
    #[pyo3(get)]
    pub source_edges: Vec<NeatEdge>,
}
#[pymethods]
impl NeatNode {
    #[new]
    fn new(
        activation: NeatUafActivation,
        source_edges: Vec<NeatEdge>,
    ) -> Self {
        Self {
            activation,
            source_edges,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct NeatGraph {
    #[pyo3(get)]
    pub nodes: Vec<NeatNode>,
}
#[pymethods]
impl NeatGraph {
    #[new]
    fn new(
        nodes: Vec<NeatNode>,
    ) -> Self {
        Self {
            nodes,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct NeatPopulation {
    pub graphs: Vec<NeatGraph>,
    //pub max_steps: usize,
}
#[pymethods]
impl NeatPopulation {
    #[new]
    fn new(
        graphs: Vec<NeatGraph>,
        //max_steps: usize,
    ) -> Self {
        Self {
            graphs,
            //max_steps,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct NeatTextures {
    pub activations: Image,
    pub edges: Image,
    pub input: Image,
    pub nodes: Image,
    pub output: Image,
}

#[pymethods]
impl NeatTextures {
    fn node_data(&mut self) -> &[u8] {
        self.nodes.data.as_slice()
    }

    fn set_input(&mut self, input: Vec<u8>) {
        self.input.data = input;
        // TODO: invalidate texture handle
    }

    fn output_data(&mut self) -> &[u8] {
        self.edges.data.as_slice()
    }
}


fn to_byte_slice(floats: &[f32]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4)
    }
}

fn pack_2d(elements: u32) -> (u32, u32) {
    let square_size = (elements as f32).sqrt().ceil() as u32;
    let excess_area = square_size.pow(2) - elements;
    let rows_to_remove = excess_area / square_size;
    let width = square_size;
    let height = square_size - rows_to_remove;
    (width, height)
}

#[pyfunction]
pub fn population_to_textures(population: &NeatPopulation) -> NeatTextures {
    let population_size = population.graphs.len();
    let (population_width, population_height) = pack_2d(population_size as u32);

    let max_node_count = population.graphs.iter().map(|g| g.nodes.len()).max().unwrap();
    let (agent_field_width, agent_field_height) = pack_2d(max_node_count as u32);

    let field_size = Extent3d {
        width: population_width * agent_field_width,
        height: population_height * agent_field_height,
        depth_or_array_layers: 1,
    };

    let mut activation_texture_data: Vec<f32> = Vec::new();
    activation_texture_data.resize((field_size.width * field_size.height) as usize * 4, 0.0);

    let max_edge_count = population.graphs.iter().map(|g| g.nodes.iter().map(|n| n.source_edges.len()).max().unwrap()).max().unwrap() as u32;
    let mut edge_texture_data: Vec<f32> = Vec::new();
    edge_texture_data.resize((field_size.width * field_size.height * max_edge_count) as usize * 4, 0.0);

    let mut node_texture_data: Vec<f32> = Vec::new();
    node_texture_data.resize((field_size.width * field_size.height) as usize * 4, 0.0);

    for (i, graph) in population.graphs.iter().enumerate() {
        let x = (i as u32 % population_width) * agent_field_width;
        let y = (i as u32 / population_width) * agent_field_height;

        for (j, node) in graph.nodes.iter().enumerate() {
            let node_x = x + (j as u32 % agent_field_width);
            let node_y = y + (j as u32 / agent_field_width);

            let node_index = (node_y * field_size.width + node_x) as usize * 4;

            // node_texture_data[node_index] = initial_state;

            activation_texture_data[node_index    ] = node.activation.a;
            activation_texture_data[node_index + 1] = node.activation.b;
            activation_texture_data[node_index + 2] = node.activation.c;
            activation_texture_data[node_index + 3] = node.activation.d;
            //activation_texture_data[node_index] = node.activation.e; // TODO: bind bias

            for (k, edge) in node.source_edges.iter().enumerate() {
                let edge_index = node_index + (field_size.width * field_size.height * 4) as usize * k;

                let edge = NeatEdge {
                    weight: edge.weight,
                    source: edge.source,
                };

                let parent_x = x + (edge.source as u32 % agent_field_width);
                let parent_y = y + (edge.source as u32 / agent_field_width);

                edge_texture_data[edge_index    ] = parent_x as f32;
                edge_texture_data[edge_index + 1] = parent_y as f32;
                edge_texture_data[edge_index + 2] = edge.weight;
                edge_texture_data[edge_index + 3] = edge.weight;
            }
        }
    }

    let mut activations = Image::new(
        field_size,
        TextureDimension::D2,
        to_byte_slice(&activation_texture_data).to_vec(),
        TextureFormat::Rgba32Float,
    );
    activations.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let edges_size = Extent3d {
        width: field_size.width,
        height: field_size.height,
        depth_or_array_layers: field_size.depth_or_array_layers * max_edge_count,
    };
    let mut edges = Image::new(
        edges_size,
        TextureDimension::D2,
        to_byte_slice(&edge_texture_data).to_vec(),
        TextureFormat::Rgba32Float,
    );
    edges.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let mut nodes = Image::new(
        field_size,
        TextureDimension::D2,
        to_byte_slice(&node_texture_data).to_vec(),
        TextureFormat::Rgba32Float,
    );
    nodes.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    NeatTextures {
        activations,
        edges,
        nodes
    }
}


#[pymodule]
fn _rusty_automata(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<NeatUafActivation>()?;
    m.add_class::<NeatEdge>()?;
    m.add_class::<NeatNode>()?;
    m.add_class::<NeatGraph>()?;
    m.add_class::<NeatPopulation>()?;
    m.add_function(wrap_pyfunction!(population_to_textures, m)?).unwrap();
    Ok(())
}
