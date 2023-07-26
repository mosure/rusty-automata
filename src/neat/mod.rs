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
            RenderContext,
            RenderDevice,
        },
        render_graph::{
            self,
            RenderGraph,
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
            CachedComputePipelineId,
            CachedPipelineState,
            ComputePassDescriptor,
            ComputePipelineDescriptor,
            Extent3d,
            PipelineCache,
            ShaderStages,
            StorageTextureAccess,
            TextureDimension,
            TextureFormat,
            TextureUsages,
            TextureViewDimension,
        },
        Render,
        RenderApp,
        RenderSet,
    },
};

use super::{
    automata::{
        AutomataBindGroup,
        AutomataPipeline,
    },
    uaf::UafPlugin,
};

use std::borrow::Cow;


const NEAT_SHADER_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 21533341678341);
const WORKGROUP_SIZE: u32 = 4;


// TODO: streaming IO
// #[derive(Resource, Clone, ExtractResource)]
// struct NeatIO {
//     handle: Handle<Image>,
//     size: (u32, u32),
//     current_index: u32,
// }


#[derive(Default)]
pub struct NeatPlugin;

impl Plugin for NeatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<NeatField>::default(),
            UafPlugin,
        ));

        load_internal_asset!(
            app,
            NEAT_SHADER_HANDLE,
            "neat.wgsl",
            Shader::from_wgsl
        );

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                queue_neat_bind_group.in_set(RenderSet::Queue),
            )
        );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("neat", NeatNode::default());
        render_graph.add_node_edge(
            "neat",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );

        // TODO: automata node should feed into neat node :D
        //       output of automata node is pre_activation, can swap pipelines for different behaviors

        // TODO: register UI editable types
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<NeatPipeline>();
    }
}


#[derive(Resource, Clone, ExtractResource)]
pub struct NeatField {
    pub uaf_activations: Handle<Image>,
}

impl NeatField {
    pub fn new(
        field_size: Extent3d,
        images: &mut ResMut<Assets<Image>>,
    ) -> Self {
        let mut uaf_activations = Image::new_fill(
            field_size,
            TextureDimension::D2,
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            TextureFormat::Rgba32Float,
        );
        uaf_activations.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
        let uaf_activations = images.add(uaf_activations);

        Self {
            uaf_activations,
        }
    }
}


#[derive(Resource)]
pub struct NeatBindGroup(pub BindGroup);

fn queue_neat_bind_group(
    mut commands: Commands,
    pipeline: Res<NeatPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    neat_field: Res<NeatField>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(
                    &gpu_images[&neat_field.uaf_activations].texture_view
                ),
            },
        ],
    });

    commands.insert_resource(NeatBindGroup(bind_group));
}

#[derive(Resource)]
pub struct NeatPipeline {
    pub bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for NeatPipeline {
    fn from_world(world: &mut World) -> Self {
        let automata = world.resource::<AutomataPipeline>();

        let bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("neat bind group layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::Rgba32Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_cache = world.resource::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![automata.bind_group_layout.clone(), bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: NEAT_SHADER_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![automata.bind_group_layout.clone(), bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: NEAT_SHADER_HANDLE.typed(),
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        NeatPipeline {
            bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}


enum NeatState {
    Loading,
    Init,
    Update,
}

struct NeatNode {
    state: NeatState,
}

impl Default for NeatNode {
    fn default() -> Self {
        Self {
            state: NeatState::Loading,
        }
    }
}

impl render_graph::Node for NeatNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<NeatPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            NeatState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = NeatState::Init;
                }
            }
            NeatState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = NeatState::Update;
                }
            }
            NeatState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let automata_bind_group = world.resource::<AutomataBindGroup>();
        let neat_bind_group = world.resource::<NeatBindGroup>();

        let pipeline_cache = world.resource::<PipelineCache>();
        let automata_pipeline = world.resource::<AutomataPipeline>();
        let pipeline = world.resource::<NeatPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &automata_bind_group.0, &[]);
        pass.set_bind_group(1, &neat_bind_group.0, &[]);

        match self.state {
            NeatState::Loading => {}
            NeatState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(automata_pipeline.width / WORKGROUP_SIZE, automata_pipeline.height / WORKGROUP_SIZE, 1);
            }
            NeatState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(automata_pipeline.width / WORKGROUP_SIZE, automata_pipeline.height / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}
