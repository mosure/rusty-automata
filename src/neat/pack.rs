use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d,
        TextureDimension,
        TextureFormat,
        TextureUsages,
    },
};


pub struct NeatUafActivation {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
}

pub struct NeatEdge {
    pub weight: f32,
    pub source: usize,
    pub input: usize,
}

pub struct NeatNode {
    pub activation: NeatUafActivation,
    pub source_edges: Vec<NeatEdge>,
}

pub struct NeatGraph {
    pub nodes: Vec<NeatNode>,
}

pub struct NeatPopulation {
    pub graphs: Vec<NeatGraph>,
    pub max_edge_count: u32,
}

pub struct NeatTextures {
    pub activations: Image,
    pub edges: Image,
    pub nodes: Image,
}

fn to_byte_slice<'a>(floats: &'a [f32]) -> &'a [u8] {
    unsafe {
        std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4)
    }
}

fn pack_2d(elements: u32) -> (u32, u32) {
    let square_size = (elements as f32).sqrt().ceil() as u32;
    let excess_area = (square_size.pow(2) - elements) as u32;
    let rows_to_remove = excess_area / square_size;
    let width = square_size;
    let height = square_size - rows_to_remove;
    (width, height)
}

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

    let mut edge_texture_data: Vec<f32> = Vec::new();
    edge_texture_data.resize((field_size.width * field_size.height * population.max_edge_count) as usize * 4, 0.0);

    let mut node_texture_data: Vec<f32> = Vec::new();
    node_texture_data.resize((field_size.width * field_size.height) as usize * 4, 0.0);

    for (i, graph) in population.graphs.iter().enumerate() {
        let x = (i as u32 % population_width) * population_width;
        let y = (i as u32 / population_width) * agent_field_height;

        for (j, node) in graph.nodes.iter().enumerate() {
            let node_x = x + (j as u32 % agent_field_width);
            let node_y = y + (j as u32 / agent_field_width);

            let node_index = (node_y * field_size.width + node_x) as usize * 4;

            // node_texture_data[node_index + 0] = initial_state;

            activation_texture_data[node_index + 0] = node.activation.a;
            activation_texture_data[node_index + 1] = node.activation.b;
            activation_texture_data[node_index + 2] = node.activation.c;
            activation_texture_data[node_index + 3] = node.activation.d;
            //activation_texture_data[node_index] = node.activation.e; // TODO: bind bias

            for (k, edge) in node.source_edges.iter().enumerate() {
                let edge_index = node_index + (field_size.width * field_size.height * 4) as usize * k;

                let edge = NeatEdge {
                    weight: edge.weight,
                    source: edge.source,
                    input: edge.input,
                };

                let parent_x = x + (edge.source as u32 % agent_field_width);
                let parent_y = y + (edge.source as u32 / agent_field_width);

                edge_texture_data[edge_index + 0] = parent_x as f32;
                edge_texture_data[edge_index + 1] = parent_y as f32;
                edge_texture_data[edge_index + 2] = edge.weight;
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
        depth_or_array_layers: field_size.depth_or_array_layers * population.max_edge_count,
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
