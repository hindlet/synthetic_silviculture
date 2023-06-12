use bevy_ecs::prelude::*;

use super::branches::{
    branch::{BranchData, BranchGrowthData, BranchBounds, BranchTag, BranchConnectionData},
    branch_node::{BranchNodeData, BranchNodeTag, BranchNodeConnectionData, get_nodes_base_to_tip}
};


pub fn debug_log_branches(branch_query: Query<(Entity, &BranchData, &BranchGrowthData, &BranchBounds, &BranchConnectionData), With<BranchTag>>) {
    for branch in branch_query.iter() {
        println!("{:?}, \n{:?}, \n{:?} \n{:?} \n{:?} \n", branch.0, branch.1, branch.2, branch.3, branch.4);
    }
    println!("\n \n");
}

pub fn debug_log_nodes(node_query: Query<(Entity, &BranchNodeData, &BranchNodeConnectionData), With<BranchNodeTag>>) {
    for node in node_query.iter() {
        println!("{:?}, \n{:?} \n{:?}\n", node.0, node.1, node.2);
    }
    println!("\n \n")
}

pub fn debug_log_nodes_and_branches(branch_query: Query<(Entity, &BranchData, &BranchGrowthData, &BranchBounds, &BranchConnectionData), With<BranchTag>>, node_query: Query<&BranchNodeData, With<BranchNodeTag>>, node_connections_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>) {
    for branch in branch_query.iter() {
        println!("{:?}, \n{:?}, \n{:?} \n{:?} \n{:?} \n\n", branch.0, branch.1, branch.2, branch.3, branch.4);
        for id in get_nodes_base_to_tip(&node_connections_query, branch.1.root_node.unwrap()) {
            println!("{:?}", id);
            if let Ok(node) = node_query.get(id) {
                println!("{:?}", node);
            }
            if let Ok(node) = node_connections_query.get(id) {
                println!("{:?} \n", node)
            }
        }
        println!("\n \n \n")
    }
    println!("\n\n\n\n\n")
}