use bevy::{prelude::*, render::extract_component::ExtractComponent};

use crate::{events, geometry};

const WORLD_BOUNDS: f32 = 128.0;
const CHUNK_SIZE: f32 = 16.0;
pub const RESOLUTION: u32 = 32;

pub const CHUNKS_PER_AXIS: i32 = ((WORLD_BOUNDS * 2.0) / CHUNK_SIZE) as i32;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_observer(on_place_geometry);
    }
}

fn setup(mut commands: Commands) {
    for x in -CHUNKS_PER_AXIS / 2..CHUNKS_PER_AXIS / 2 {
        for y in -CHUNKS_PER_AXIS / 2..CHUNKS_PER_AXIS / 2 {
            for z in -CHUNKS_PER_AXIS / 2..CHUNKS_PER_AXIS / 2 {
                let idx = ChunkIdx { x, y, z };
                commands.spawn(Chunk::new(idx));
            }
        }
    }
}

fn on_place_geometry(
    trigger: Trigger<events::GeometryAdded>,
    chunks: Query<&mut Chunk>,
    primatives: Query<&mut geometry::BoxGeometry>,
) {
    let primative = primatives.get(trigger.entity).expect("exists");

    mark_dirty_chunks(chunks, primative);
}

/// Given a primative change let the renderer know that a chunk needs
/// rerendering.
fn mark_dirty_chunks(mut chunks: Query<&mut Chunk>, primative: &geometry::BoxGeometry) {
    let min = primative.position - primative.scale;
    let max = primative.position + primative.scale;

    for mut chunk in &mut chunks {
        let origin = chunk.idx.world_origin();
        let chunk_min = origin;
        let chunk_max = origin + Vec3::splat(CHUNK_SIZE);

        let has_overlap = (min.x <= chunk_max.x && max.x >= chunk_min.x)
            && (min.y <= chunk_max.y && max.y >= chunk_min.y)
            && (min.z <= chunk_max.z && max.z >= chunk_min.z);

        if has_overlap {
            chunk.mark_dirty();
        }
    }
}

#[derive(Component, ExtractComponent, Clone, Debug)]
pub struct Chunk {
    pub idx: ChunkIdx,
    pub version: u64,
}

impl Chunk {
    pub fn new(idx: ChunkIdx) -> Self {
        Self { idx, version: 1 }
    }

    fn mark_dirty(&mut self) {
        self.version += 1;
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChunkIdx {
    x: i32,
    y: i32,
    z: i32,
}

impl ChunkIdx {
    /// pos must be within world bounds.
    pub fn from_world_pos(pos: Vec3) -> Self {
        Self {
            x: (pos.x / CHUNK_SIZE).floor() as i32,
            y: (pos.y / CHUNK_SIZE).floor() as i32,
            z: (pos.z / CHUNK_SIZE).floor() as i32,
        }
    }

    pub fn world_origin(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * CHUNK_SIZE,
            self.y as f32 * CHUNK_SIZE,
            self.z as f32 * CHUNK_SIZE,
        )
    }

    pub fn to_vec(&self) -> UVec3 {
        UVec3::new(
            (self.x + 8) as u32,
            (self.y + 8) as u32,
            (self.z + 8) as u32,
        )
    }
}
