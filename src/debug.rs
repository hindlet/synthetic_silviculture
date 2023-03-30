use bevy_ecs::prelude::*;


fn debug_log_branches(branch_query: Query<(&BranchData, &BranchGrowthData, &BranchBounds), With<BranchTag>>, node_query: Query<&BranchNodeData, With<BranchNodeTag>>) {
    for branch in branch_query.iter() {
        println!("{:?}, \n{:?}, \n{:?} \n{:?} \n", branch.0, branch.1, branch.2, node_query.get(branch.0.root_node.unwrap()).unwrap().position);
    }
    println!("\n \n");
}

fn debug_log_nodes(node_query: Query<(Entity, &BranchNodeData, &BranchNodeConnectionData), With<BranchNodeTag>>) {
    for node in node_query.iter() {
        println!("{:?}, \n{:?} \n{:?}\n", node.0, node.1, node.2);
    }
    println!("\n \n")
}

fn debug_log_nodes_and_branches(branch_query: Query<(Entity, &BranchData, &BranchGrowthData, &BranchBounds), With<BranchTag>>, node_query: Query<&BranchNodeData, With<BranchNodeTag>>, node_connections_query: Query<&BranchNodeConnectionData, With<BranchNodeTag>>) {
    for branch in branch_query.iter() {
        println!("{:?}, \n{:?}, \n{:?} \n{:?} \n{:?} \n\n", branch.0, branch.1, branch.2, branch.3, node_query.get(branch.1.root_node.unwrap()).unwrap().position);
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