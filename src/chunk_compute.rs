use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_asset::RenderAssets;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::{Render, RenderApp, RenderSet, render_resource::*};

use crate::{rendering, world};

pub struct ChunkComputePlugin;

fn chunk_compute(
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<SdfComputePipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut query: Query<(&mut ComputeChunk, &world::Chunk)>,
) {
    let pso = pipeline_cache
        .get_compute_pipeline(pipeline.pipeline)
        .expect("exists");

    let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("SDF compute encoder"),
    });

    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_pipeline(pso);

        for (mut compute_chunk, world_chunk) in &mut query {
            if world_chunk.version == compute_chunk.last_uploaded_version {
                continue;
            }

            pass.set_bind_group(0, &compute_chunk.bind_group, &[]);
            pass.dispatch_workgroups(4, 4, 4);
            compute_chunk.last_uploaded_version = world_chunk.version;
        }
    }

    render_queue.submit(Some(encoder.finish()));
}

impl Plugin for ChunkComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<world::Chunk>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                prepare_bind_groups.in_set(RenderSet::PrepareBindGroups),
                chunk_compute.in_set(RenderSet::Render),
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<SdfComputePipeline>();
    }
}

#[derive(Resource)]
struct SdfComputePipeline {
    pipeline: CachedComputePipelineId,
    layout: BindGroupLayout,
}

impl FromWorld for SdfComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let shader = asset_server.load("shaders/chunk_compute.wgsl");

        let layout = render_device.create_bind_group_layout(
            Some("chunk_compute_bgl"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D3,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(12),
                    },
                    count: None,
                },
            ],
        );

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("chunk_compute_pipeline".into()),
            layout: vec![layout.clone()],
            shader,
            shader_defs: vec![],
            entry_point: "main".into(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        });

        Self {
            pipeline,
            layout,
        }
    }
}

fn prepare_bind_groups(
    mut commands: Commands,
    pipeline: Res<SdfComputePipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    chunks: Query<(&world::Chunk, Entity), Without<ComputeChunk>>,
    voxel_texture: Res<rendering::WorldSpaceTextureHandle>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    for (chunk, entity) in chunks.iter() {
        let texture = gpu_images
            .get(&voxel_texture.0)
            .expect("exists");

        let mut uniform_buffer = UniformBuffer::from(chunk.idx.to_vec());
        uniform_buffer.write_buffer(&render_device, &queue);

        let bind_group = render_device.create_bind_group(
            None,
            &pipeline.layout,
            &BindGroupEntries::sequential((&texture.texture_view, &uniform_buffer)),
        );

        commands.entity(entity).insert(ComputeChunk {
            bind_group,
            last_uploaded_version: 0,
        });
    }
}

#[derive(Component)]
struct ComputeChunk {
    bind_group: BindGroup,
    last_uploaded_version: u64,
}
