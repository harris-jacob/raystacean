use bevy::prelude::*;

use crate::{events, node_id};

#[derive(Resource, Default)]
struct OperationsForest {
    roots: Vec<Node>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Node {
    Geometry(node_id::NodeId),
    Union,
}

struct Union {
    id: node_id::NodeId,
    left: node_id::NodeId,
    right: node_id::NodeId,
}

pub struct OperationsPlugin;

impl Plugin for OperationsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OperationsForest::default())
            .add_observer(on_geometry_added);
    }
}

fn on_geometry_added(
    trigger: Trigger<events::GeometryAdded>,
    mut operations: ResMut<OperationsForest>,
) {
    operations.roots.push(Node::Geometry(trigger.id));
}
