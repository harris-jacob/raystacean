use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_asset::RenderAssets;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::storage::ShaderStorageBuffer;
use bevy::render::texture::GpuImage;
use bevy::render::{Render, RenderApp, RenderSet, render_resource::*};

use crate::{geometry, rendering, world};

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

fn setup_primatives_buffer(
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut commands: Commands,
) {
    let primatives = buffers.add(ShaderStorageBuffer::default());
    commands.insert_resource(PrimativesBufferHandle(primatives));
}

#[repr(C)]
#[derive(Clone, ShaderType, Default, Debug)]
pub struct GpuPrimative {
    pub position: [f32; 3],
    pub is_subtract: u32,
    pub scale: [f32; 3],
    pub blend: f32,
    pub color: [f32; 3],
    pub rounding_radius: f32,
    pub logical_color: [f32; 3],
    _pad1: f32,
}

fn boxes_to_gpu(
    boxes: Query<&geometry::BoxGeometry>,
    buffer_handle: Res<PrimativesBufferHandle>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let buffer = buffer_handle.get_mut(&mut buffers);

    let gpu_data: Vec<GpuPrimative> = boxes
        .iter()
        // Sorted by ID to ensure stable operation ordering seen by the shader
        .sort_by::<&geometry::BoxGeometry>(|a, b| a.id.cmp(&b.id))
        .map(|b| GpuPrimative {
            position: b.position.into(),
            scale: b.scale.into(),
            color: b.color,
            blend: b.blend,
            rounding_radius: b.rounding_radius(),
            logical_color: b.id.to_color(),
            is_subtract: if b.is_subtract { 1 } else { 0 },
            ..default()
        })
        .collect();

    buffer.set_data(gpu_data);
}

#[derive(Resource, ExtractResource, Clone)]
pub struct PrimativesBufferHandle(Handle<ShaderStorageBuffer>);

impl PrimativesBufferHandle {
    pub fn get_mut<'a>(
        &self,
        assets: &'a mut Assets<ShaderStorageBuffer>,
    ) -> &'a mut ShaderStorageBuffer {
        assets
            .get_mut(&self.0)
            .expect("ShaderStorageBuffer should exist")
    }
}

impl Plugin for ChunkComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<world::Chunk>::default());
        app.add_plugins(ExtractResourcePlugin::<PrimativesBufferHandle>::default());
        app.add_plugins(ExtractResourcePlugin::<ExtractedPrimatives>::default());

        app.add_systems(
            Startup,
            (setup_primatives_buffer, setup_extracted_primatives),
        )
        .add_systems(Update, (boxes_to_gpu, extract_boxes));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                upload_prims_to_gpu.in_set(RenderSet::PrepareResources),
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

// Extract CPU boxes -> render-world Vec<GpuPrimative>
#[derive(Resource, ExtractResource, Clone, Default)]
pub struct ExtractedPrimatives(pub Vec<GpuPrimative>);

fn setup_extracted_primatives(mut commands: Commands) {
    commands.insert_resource(ExtractedPrimatives::default())
}

// Main world: build the vec (reuse your existing code)
fn extract_boxes(mut out: ResMut<ExtractedPrimatives>, boxes: Query<&geometry::BoxGeometry>) {
    out.0.clear();
    for b in boxes
        .iter()
        .sort_by::<&geometry::BoxGeometry>(|a, b| a.id.cmp(&b.id))
    {
        out.0.push(GpuPrimative {
            position: b.position.into(),
            scale: b.scale.into(),
            color: b.color,
            blend: b.blend,
            rounding_radius: b.rounding_radius(),
            logical_color: b.id.to_color(),
            is_subtract: u32::from(b.is_subtract),
            ..Default::default()
        });
    }
}

// Render world: write into the persistent StorageBuffer
fn upload_prims_to_gpu(
    mut pipeline: ResMut<SdfComputePipeline>,
    prims: Res<ExtractedPrimatives>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    pipeline.primative_buffer.set(prims.0.clone());
    pipeline
        .primative_buffer
        .write_buffer(&render_device, &queue);

    pipeline
        .primative_meta
        .set(UVec4::new(prims.0.len() as u32, 0, 0, 0));
    pipeline.primative_meta.write_buffer(&render_device, &queue);
}

#[derive(Resource)]
struct SdfComputePipeline {
    pipeline: CachedComputePipelineId,
    layout: BindGroupLayout,
    primative_buffer: StorageBuffer<Vec<GpuPrimative>>,
    primative_meta: UniformBuffer<UVec4>,
}

impl FromWorld for SdfComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();
        let queue = world.resource::<RenderQueue>();

        let shader = asset_server.load("shaders/chunk_compute.wgsl");

        let mut primative_buffer = StorageBuffer::from(vec![GpuPrimative::default(); 100]);

        let mut primative_meta = UniformBuffer::from(UVec4::default());

        primative_buffer.write_buffer(render_device, queue);
        primative_meta.write_buffer(render_device, queue);

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
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(16),
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
            primative_buffer,
            primative_meta,
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
        let texture = gpu_images.get(&voxel_texture.0).expect("exists");

        let mut uniform_buffer = UniformBuffer::from(chunk.idx.to_vec());
        uniform_buffer.write_buffer(&render_device, &queue);

        let bind_group = render_device.create_bind_group(
            None,
            &pipeline.layout,
            &BindGroupEntries::sequential((
                &texture.texture_view,
                &uniform_buffer,
                &pipeline.primative_buffer,
                &pipeline.primative_meta,
            )),
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
