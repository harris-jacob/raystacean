use std::sync::Arc;

use bevy::prelude::*;

use crate::{
    controls, events, geometry,
    global_id::{self, GlobalId},
    node_id, selection,
};

#[derive(Resource, Default, Debug)]
pub struct OperationsForest {
    pub roots: Vec<Node>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Node {
    Geometry(node_id::NodeId),
    Union(Union),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Union {
    pub id: node_id::NodeId,
    pub left: Arc<Node>,
    pub right: Arc<Node>,
}

pub struct OperationsPlugin;

impl Plugin for OperationsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OperationsForest::default())
            .add_systems(Update, perform_union)
            .add_observer(on_geometry_added);
    }
}

fn on_geometry_added(
    trigger: Trigger<events::GeometryAdded>,
    mut operations: ResMut<OperationsForest>,
) {
    operations.roots.push(Node::Geometry(trigger.id));
}

fn perform_union(
    control_mode: Res<controls::ControlMode>,
    selected: Query<(&geometry::BoxGeometry, Entity), With<selection::Selected>>,
    mut operations: ResMut<OperationsForest>,
    mut new_id: ResMut<global_id::GlobalId>,
    mut commands: Commands,
) {
    if *control_mode != controls::ControlMode::UnionSelect {
        return;
    }

    if selected.iter().len() != 2 {
        return;
    }

    let left = operations
        .take_root_of(&selected.iter().next().expect("exists").0.id)
        .expect("exists");
    // TODO: could be that the two selected items belong to the same root, which means
    // they are already part of a heirarchy of CSG operations, this is not valid but I need
    // to figure out how to handle it.
    let right = operations
        .take_root_of(&selected.iter().nth(1).expect("exists").0.id)
        .expect("exists");

    let node = Node::Union(Union {
        id: node_id::NodeId::new(new_id.next()),
        left: Arc::new(left),
        right: Arc::new(right),
    });

    operations.insert_root(node);

    dbg!(&operations);

    // TODO: get working with event
    for (_, entity) in selected.iter() {
        commands.entity(entity).remove::<selection::Selected>();
    }
}

impl OperationsForest {
    fn take_root_of(&mut self, target: &node_id::NodeId) -> Option<Node> {
        if let Some(idx) = self.roots.iter().position(|node| node.contains(target)) {
            Some(self.roots.remove(idx))
        } else {
            None
        }
    }

    fn insert_root(&mut self, node: Node) {
        self.roots.push(node);
    }
}

impl Node {
    fn contains(&self, id: &node_id::NodeId) -> bool {
        match self {
            Node::Geometry(node_id) => node_id == id,
            Node::Union(union) => {
                let left = union.left.contains(id);
                let right = union.right.contains(id);

                left | right
            }
        }
    }
}
