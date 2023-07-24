use bevy::{
    asset::{
        load_internal_asset,
        HandleUntyped,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_resource::{
            ExtractResource,
            ExtractResourcePlugin,
        },
        render_asset::RenderAssets,
        renderer::{
            RenderDevice,
            RenderQueue,
        },
        render_resource::{
            BindGroup,
            BindGroupDescriptor,
            BindGroupEntry,
            BindGroupLayout,
            BindGroupLayoutDescriptor,
            BindGroupLayoutEntry,
            BindingResource,
            BindingType,
            BufferBindingType,
            Extent3d,
            ShaderStages,
            ShaderType,
            StorageTextureAccess,
            TextureDimension,
            TextureFormat,
            TextureUsages,
            TextureViewDimension,
            UniformBuffer,
        },
        Render,
        RenderApp,
        RenderSet,
    },
};


const AUTOMATA_SHADER_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 6712956732940);


#[derive(Default)]
pub struct AutomataPlugin;

impl Plugin for AutomataPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            AUTOMATA_SHADER_HANDLE,
            "automata.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins((
            ExtractResourcePlugin::<AutomataField>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                prepare_automata_uniforms.in_set(RenderSet::Prepare),
                queue_automata_bind_group.in_set(RenderSet::Queue),
            )
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<AutomataPipeline>();
        render_app.init_resource::<AutomataUniformBuffer>();
    }
}


// TODO: set max number of steps /w completion event
#[derive(Resource, Clone, ExtractResource)]
pub struct AutomataField {
    pub edges: Handle<Image>,
    pub nodes: Handle<Image>,
    edge_count: u32,
    max_radius: f32,
    max_edge_weight: f32,
    seed: f32,
    width: u32,
    height: u32,
}

// TODO: use default implementation
impl AutomataField {
    // TODO: from_graph implementation
    pub fn new(
        field_size: Extent3d,
        edge_count: u32,
        images: &mut ResMut<Assets<Image>>,
    ) -> Self {
        let mut nodes = Image::new_fill(
            field_size,
            TextureDimension::D2,
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            TextureFormat::Rgba32Float,
        );
        nodes.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
        let nodes = images.add(nodes);


        // 2D to assist cache locality
        let edges_size = Extent3d {
            width: field_size.width,
            height: field_size.height,
            depth_or_array_layers: field_size.depth_or_array_layers * edge_count,
        };

        let mut edges: Image = Image::new_fill(
            edges_size,
            TextureDimension::D2,
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            TextureFormat::Rgba32Float,
        );
        edges.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
        let edges = images.add(edges);

        Self {
            edges,
            nodes,
            edge_count,
            max_radius: 15.0,
            max_edge_weight: 1.0,
            seed: 1.0,
            width: field_size.width,
            height: field_size.height,
        }
    }
}


#[derive(Clone, Default, ShaderType)]
struct AutomataUniform {
    edge_count: u32,
    max_radius: f32,
    max_edge_weight: f32,
    seed: f32,
    width: u32,
    height: u32,
}

#[derive(Resource, Default)]
struct AutomataUniformBuffer {
    buffer: UniformBuffer<AutomataUniform>,
}

fn prepare_automata_uniforms(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut uniform_buffer: ResMut<AutomataUniformBuffer>,
    automata: Res<AutomataField>,
) {
    let buffer = uniform_buffer.buffer.get_mut();

    buffer.edge_count = automata.edge_count;
    buffer.max_radius = automata.max_radius;
    buffer.max_edge_weight = automata.max_edge_weight;
    buffer.seed = automata.seed;
    buffer.width = automata.width;
    buffer.height = automata.height;

    uniform_buffer.buffer.write_buffer(&render_device, &render_queue);
}

#[derive(Resource)]
pub struct AutomataBindGroup(pub BindGroup);

fn queue_automata_bind_group(
    mut commands: Commands,
    mut pipeline: ResMut<AutomataPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    automata: Res<AutomataField>,
    render_device: Res<RenderDevice>,
    uniform_buffer: ResMut<AutomataUniformBuffer>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(
                    &gpu_images[&automata.edges].texture_view
                ),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(
                    &gpu_images[&automata.nodes].texture_view
                ),
            },
            BindGroupEntry {
                binding: 2,
                resource: uniform_buffer.buffer.binding().unwrap(),
            },
        ],
    });

    commands.insert_resource(AutomataBindGroup(bind_group));

    pipeline.width = automata.width;
    pipeline.height = automata.height;
}


// TODO: internal step counter, validate step count in render world equals main world
#[derive(Resource)]
pub struct AutomataPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub width: u32,
    pub height: u32,
    // TODO: allow dynamic number of fields (workgroup depth)
}

impl FromWorld for AutomataPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("automata bind group layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::Rgba32Float,
                                view_dimension: TextureViewDimension::D2Array,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::Rgba32Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        AutomataPipeline {
            bind_group_layout,
            width: 0,
            height: 0,
        }
    }
}
