use bevy::prelude::*;

use crate::geometry;

pub struct BvhPlugin;

impl Plugin for BvhPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BVHTree::default());
        app.add_systems(Update, update_bvh);
    }
}

fn update_bvh(query: Query<&geometry::BoxGeometry>, mut bvh_tree: ResMut<BVHTree>) {
    // TODO: no clones if possible
    let boxes: Vec<geometry::BoxGeometry> = query.iter().cloned().collect();

    let mut indices: Vec<usize> = (0..boxes.len()).collect();

    bvh_tree.root = build_bvh_recursive(&boxes, &mut indices);
}

#[derive(Resource)]
pub struct BVHTree {
    pub root: BVHNode,
}

impl Default for BVHTree {
    fn default() -> Self {
        Self {
            root: BVHNode::empty(),
        }
    }
}

#[derive(Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Debug)]
pub enum BVHNode {
    Internal {
        bounds: AABB,
        left: Box<BVHNode>,
        right: Box<BVHNode>,
    },
    Leaf {
        bounds: AABB,
        prim_indices: Vec<usize>,
    },
}

impl BVHNode {
    fn empty() -> Self {
        BVHNode::Leaf {
            bounds: AABB::empty(),
            prim_indices: Vec::new(),
        }
    }
}

impl AABB {
    fn empty() -> Self {
        Self {
            min: Vec3::INFINITY,
            max: Vec3::NEG_INFINITY,
        }
    }

    fn grow(&mut self, other: AABB) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    fn extent(&self) -> Vec3 {
        self.max - self.min
    }

    fn largest_axis(&self) -> usize {
        let extent = self.extent();

        if extent.x > extent.y && extent.x > extent.z {
            0
        } else if extent.y > extent.z {
            1
        } else {
            2
        }
    }
}

const MAX_LEAF_SIZE: usize = 4;

fn compute_bounds(geometry: &[geometry::BoxGeometry], indices: &[usize]) -> AABB {
    let mut b = AABB::empty();
    for &i in indices {
        b.grow(geometry[i].bounds());
    }
    b
}

fn build_bvh_recursive(geometry: &[geometry::BoxGeometry], indices: &mut [usize]) -> BVHNode {
    let bounds = compute_bounds(geometry, indices);

    if indices.len() <= MAX_LEAF_SIZE {
        return BVHNode::Leaf {
            bounds,
            prim_indices: indices.to_vec(),
        };
    }

    let axis = bounds.largest_axis();

    indices.sort_by(|&a, &b| {
        let ca = geometry[a].position;
        let cb = geometry[b].position;

        let va = match axis {
            0 => ca.x,
            1 => ca.y,
            _ => ca.z,
        };
        let vb = match axis {
            0 => cb.x,
            1 => cb.y,
            _ => cb.z,
        };
        va.partial_cmp(&vb).unwrap()
    });

    let mid = indices.len() / 2;
    let (left_indices, right_indices) = indices.split_at_mut(mid);

    let left = build_bvh_recursive(geometry, left_indices);
    let right = build_bvh_recursive(geometry, right_indices);

    BVHNode::Internal {
        bounds,
        left: Box::new(left),
        right: Box::new(right),
    }
}
