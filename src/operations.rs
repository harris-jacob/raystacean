use bevy::prelude::*;

use crate::{controls, events, geometry, global_id, node_id, selection};

#[derive(Resource, Default, Debug)]
pub struct OperationsForest {
    pub roots: Vec<Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Geometry(node_id::NodeId),
    Union(Operation),
    Subtract(Operation),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Operation {
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
            .add_systems(Update, (perform_union, perform_subtract))
            .add_observer(on_geometry_added);
    }
}

fn on_geometry_added(
    trigger: Trigger<events::GeometryAdded>,
    mut operations: ResMut<OperationsForest>,
) {
    operations.roots.push(Node::Geometry(trigger.id));
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CsgOperation {
    Union,
    Subtract,
}

fn perform_subtract(
    control_mode: ResMut<controls::ControlMode>,
    selected: Query<(&geometry::BoxGeometry, Entity), With<selection::Selected>>,
    operations: ResMut<OperationsForest>,
    new_id: ResMut<global_id::GlobalId>,
    commands: Commands,
) {
    perform_csg_operation(
        control_mode,
        selected,
        operations,
        new_id,
        commands,
        CsgOperation::Subtract,
    )
}

fn perform_union(
    control_mode: ResMut<controls::ControlMode>,
    selected: Query<(&geometry::BoxGeometry, Entity), With<selection::Selected>>,
    operations: ResMut<OperationsForest>,
    new_id: ResMut<global_id::GlobalId>,
    commands: Commands,
) {
    perform_csg_operation(
        control_mode,
        selected,
        operations,
        new_id,
        commands,
        CsgOperation::Union,
    )
}

fn perform_csg_operation(
    mut control_mode: ResMut<controls::ControlMode>,
    selected: Query<(&geometry::BoxGeometry, Entity), With<selection::Selected>>,
    mut operations: ResMut<OperationsForest>,
    mut new_id: ResMut<global_id::GlobalId>,
    mut commands: Commands,
    operation_type: CsgOperation,
) {
    if !is_in_expected_control_mode(operation_type, &control_mode) {
        return;
    }

    let mut selected = selected.iter();

    if selected.len() != 2 {
        return;
    }

    let first_primative = selected.next().expect("exists").0;
    let second_primative = selected.next().expect("exists").0;

    let first = operations.find_root(&first_primative.id).expect("exists");
    let second = operations.find_root(&second_primative.id).expect("exists");

    // The Nodes already belong to the same root (operation doesn't make
    // sense)
    if first == second {
        trigger_operation_error_event(operation_type, &mut commands);
        *control_mode = controls::ControlMode::Select;
        return;
    }

    let first_id = first.id();
    let second_id = second.id();

    let left = operations
        .take_root(&first_id)
        .expect("Node does not exists in tree");

    let right = operations
        .take_root(&second_id)
        .expect("Node does not exist in tree");

    let color = match &left {
        Node::Geometry(_) => first_primative.color,
        Node::Union(operation) => operation.color,
        Node::Subtract(operation) => operation.color,
    };

    let operation = Operation {
        id: node_id::NodeId::new(new_id.next()),
        left: Box::new(left),
        right: Box::new(right),
        color,
        blend: 0.0,
    };

    insert_operation(operation_type, operation, operations);
    trigger_operation_performed_event(operation_type, &mut commands);

    *control_mode = controls::ControlMode::Select;
}

fn is_in_expected_control_mode(
    operation_type: CsgOperation,
    control_mode: &controls::ControlMode,
) -> bool {
    let expected_mode = match operation_type {
        CsgOperation::Union => controls::ControlMode::UnionSelect,
        CsgOperation::Subtract => controls::ControlMode::SubtractSelect,
    };

    *control_mode == expected_mode
}

fn insert_operation(
    operation_type: CsgOperation,
    operation: Operation,
    mut operations: ResMut<OperationsForest>,
) {
    let node = match operation_type {
        CsgOperation::Union => Node::Union(operation),
        CsgOperation::Subtract => Node::Subtract(operation),
    };

    operations.insert_root(node);
}

fn trigger_operation_performed_event(operation_type: CsgOperation, commands: &mut Commands) {
    match operation_type {
        CsgOperation::Union => commands.trigger(events::UnionOperationPerformed),
        CsgOperation::Subtract => commands.trigger(events::SubtractOperationPerformed),
    };
}

fn trigger_operation_error_event(operation_type: CsgOperation, commands: &mut Commands) {
    match operation_type {
        CsgOperation::Union => commands.trigger(events::UnionOperationErrored),
        CsgOperation::Subtract => commands.trigger(events::SubtractOperationErrored),
    };
}

impl OperationsForest {
    pub fn find_root_mut(&mut self, target: &node_id::NodeId) -> Option<&mut Node> {
        self.roots.iter_mut().find(|node| node.contains(target))
    }

    /// Find the root of a node and take it out of the tree
    fn find_root(&self, target: &node_id::NodeId) -> Option<&Node> {
        self.roots.iter().find(|node| node.contains(target))
    }

    fn take_root(&mut self, target: &node_id::NodeId) -> Option<Node> {
        let idx = self.roots.iter().position(|node| node.id() == *target)?;

        Some(self.roots.remove(idx))
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
            Node::Subtract(subtract) => {
                let left = subtract.left.contains(id);
                let right = subtract.right.contains(id);

                left | right
            }
        }
    }

    fn id(&self) -> node_id::NodeId {
        match self {
            Node::Geometry(node_id) => *node_id,
            Node::Union(union) => union.id,
            Node::Subtract(subract) => subract.id,
        }
    }
}
