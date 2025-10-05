use bevy::prelude::*;

use crate::{controls, events, geometry, global_id, node_id, selection};

#[derive(Resource, Default, Debug)]
pub struct OperationsForest {
    pub roots: Vec<Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Geometry(node_id::NodeId),
    Union(Union),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Union {
    pub id: node_id::NodeId,
    pub left: Box<Node>,
    pub right: Box<Node>,
    pub blend: f32,
    pub color: [f32; 3],
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
    mut control_mode: ResMut<controls::ControlMode>,
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

    let first = selected.iter().next().expect("exists").0;
    let second = selected.iter().nth(1).expect("exists").0;
    let left = operations
        .find_root_index(&first.id)
        .expect("Node does not exists in tree");
    let right = operations
        .find_root_index(&second.id)
        .expect("Node does not exist in tree");

    // The Nodes already belong to the same root (union operation doesn't make
    // sense)
    if left == right {
        commands.trigger(events::UnionOperationErrored);
        *control_mode = controls::ControlMode::Select;
        return;
    }

    let left = operations.take_root(left);

    // TODO: avoid duplicate operation
    let right = operations
        .find_root_index(&selected.iter().nth(1).expect("exists").0.id)
        .expect("Node does not exist in tree");

    let right = operations.take_root(right);

    let node = Node::Union(Union {
        id: node_id::NodeId::new(new_id.next()),
        left: Box::new(left),
        right: Box::new(right),
        // Unions take the color of the first operation used to create them
        color: first.color,
        blend: 0.0,
    });

    operations.insert_root(node);
    commands.trigger(events::UnionOperationPerformed);

    *control_mode = controls::ControlMode::Select;
}

impl OperationsForest {
    pub fn find_root_mut(&mut self, target: &node_id::NodeId) -> Option<&mut Node> {
        self.roots.iter_mut().find(|node| node.contains(target))
    }
    fn take_root(&mut self, idx: usize) -> Node {
        self.roots.remove(idx)
    }

    fn find_root_index(&self, target: &node_id::NodeId) -> Option<usize> {
        self.roots.iter().position(|node| node.contains(target))
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
