#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use image::{GenericImageView};
use crate::{
    vector_three::Vector3,
    bounding_sphere::BoundingSphere,
    bounding_box::BoundingBox,
    plant::*,
    branch::*,
    branch_prototypes::*,
    branch_node::*,
};
use std::f32::consts::PI;



///////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////// Plants  ///////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod plant_bounds_tests {

    use bevy_ecs::schedule::IntoSystemConfig;

    use super::{World, PlantBundle, PlantTag, Vector3, BoundingBox,
        Schedule, update_plant_intersections,
        With, Query, BranchBundle, BoundingSphere,
        BranchTag, BranchData, update_branch_intersections,
        update_plant_bounds, PlantData, BranchConnectionData,
        PlantGrowthControlFactors, PlantBounds, BranchBounds
    };


    #[test]
    fn intersections_test() {
        let mut test_world = World::new();

        test_world.spawn(PlantBundle {
            tag: PlantTag,
            bounds: PlantBounds::from(BoundingBox {
                least_corner: Vector3::ZERO(),
                width: 5.0,
                height: 7.0,
                depth: 3.0,
            }),
            data: PlantData::default(),
            growth_factors: PlantGrowthControlFactors::default(),
        });

        test_world.spawn(PlantBundle {
            tag: PlantTag,
            bounds: PlantBounds::from(BoundingBox {
                least_corner: Vector3::new(2.0, 5.0, 2.0),
                width: 5.0,
                height: 7.0,
                depth: 3.0,
            }),
            data: PlantData::default(),
            growth_factors: PlantGrowthControlFactors::default(),
        });

        test_world.spawn(PlantBundle {
            tag: PlantTag,
            bounds: PlantBounds::from(BoundingBox {
                least_corner: Vector3::new(6.0, 5.0, 2.0),
                width: 5.0,
                height: 7.0,
                depth: 3.0,
            }),
            data: PlantData::default(),
            growth_factors: PlantGrowthControlFactors::default(),
        });

        let mut test_schedule = Schedule::default();
        
        test_schedule.add_system(update_plant_intersections);
        
        test_schedule.run(&mut test_world);

        let mut intersection_count: usize = 0;
        let mut query = test_world.query::<&PlantData>();
        for intersections in query.iter(&test_world) {
            intersection_count += intersections.intersection_list.len();
        }

        assert_eq!(intersection_count, 2);
    }

    #[test]
    fn plant_calculated_bounds_test() {
        let mut test_world = World::new();

        let branch_one = test_world.spawn(BranchBundle {
            bounds: BranchBounds::from(BoundingSphere {
                centre: Vector3::ZERO(),
                radius: 5.0,
            }),
            ..Default::default()
        })
        .id();

        let branch_two = test_world.spawn(BranchBundle {
            bounds: BranchBounds::from(BoundingSphere {
                centre: Vector3{x: 5.0, y: 2.0, z: 7.0},
                radius: 5.0,
            }),
            connections: BranchConnectionData {
                children: (Some(branch_one), None),
                parent: None,
            },
            ..Default::default()
        })
        .id();

        let branch_three = test_world.spawn(BranchBundle {
            bounds: BranchBounds::from(BoundingSphere {
                centre: Vector3{x: 12.0, y: 3.0, z: 15.0},
                radius: 6.0,
            }),
            ..Default::default()
        })
        .id();


        test_world.spawn(PlantBundle {
            tag: PlantTag,
            bounds: PlantBounds::default(),
            data: PlantData {
                root_node: Some(branch_two),
                ..Default::default()
            },
            growth_factors: PlantGrowthControlFactors::default(),
        });

        test_world.spawn(PlantBundle {
            tag: PlantTag,
            bounds: PlantBounds::default(),
            data: PlantData {
                root_node: Some(branch_three),
                ..Default::default()
            },
            growth_factors: PlantGrowthControlFactors::default(),
        });



        let mut test_schedule = Schedule::default();

        test_schedule.add_systems((update_plant_bounds, update_plant_intersections.after(update_plant_bounds), update_branch_intersections.after(update_plant_intersections)));
        
        test_schedule.run(&mut test_world);

        let mut plant_intersection_count: usize = 0;
        let mut plant_query = test_world.query::<&PlantData>();
        for intersections in plant_query.iter(&test_world) {
            plant_intersection_count += intersections.intersection_list.len();
        }

        let mut branch_intersection_count: usize = 0;
        let mut branch_query = test_world.query::<&BranchData>();
        for intersections in branch_query.iter(&test_world) {
            branch_intersection_count += intersections.intersection_list.len();
        }

        assert_eq!(plant_intersection_count, 1);
        assert_eq!(branch_intersection_count, 2);

    }

}

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////// Vigor ///////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod vigor_and_light_exposure_tests {

    use bevy_ecs::schedule::IntoSystemConfig;

    use super::{BranchBundle, BranchConnectionData,
    PlantBundle, Query, BranchData, BoundingSphere, With, BranchTag, Entity,
    calculate_branch_light_exposure, Vector3, World, Schedule,
    calculate_branch_intersection_volumes, PI,
    QueryState, calculate_growth_vigor, PlantData, PlantGrowthControlFactors,
    BranchGrowthData, BranchBounds};

    /// this function is for testing purposes,
    /// it checks every branch intersecting with every other branch
    /// this means it's super slow at large scales
    fn testing_branch_intersections(
        mut branch_query: Query<(&mut BranchData, &BranchBounds, Entity), With<BranchTag>>,
    ) {
        // reset intersections list
        for (mut data, sphere, entity) in &mut branch_query {
            data.intersection_list = Vec::new();
        }

        // check intersections
        let mut combinations = branch_query.iter_combinations_mut();
        while let Some([mut branch_one, branch_two]) = combinations.fetch_next() {
            if branch_one.1.bounds.is_intersecting_sphere(&branch_two.1.bounds) {
                branch_one.0.intersection_list.push(branch_two.2);
            }
        }
    }

    #[test]
    fn intersection_volume_test() {
        let mut test_world = World::new();

        let branch_one_id = test_world.spawn(BranchBundle {
            bounds: BranchBounds::from(BoundingSphere {
                centre: Vector3::ZERO(),
                radius: 2.0,
            }),
            ..Default::default()
        })
        .id();

        let branch_two_id = test_world.spawn(BranchBundle {
            bounds: BranchBounds::from(BoundingSphere {
                centre: Vector3{x: -2.0, y: 2.0, z: -1.0},
                radius: 2.0,
            }),
            ..Default::default()
        })
        .id();

        let branch_three_id = test_world.spawn(BranchBundle {
            bounds: BranchBounds::from(BoundingSphere {
                centre: Vector3{x: -4.0, y: 4.0, z: -2.0},
                radius: 2.0,
            }),
            ..Default::default()
        })
        .id();

        let mut test_schedule = Schedule::default();

        test_schedule.add_systems((testing_branch_intersections, calculate_branch_intersection_volumes.after(testing_branch_intersections)));
        test_schedule.run(&mut test_world);

        let mut total_intersection_count: usize = 0;
        let mut branch_query = test_world.query::<&BranchData>();
        for branch in branch_query.iter(&test_world) {
            total_intersection_count += branch.intersection_list.len();
        }

        assert_eq!(total_intersection_count, 2);

        if let Ok(branch_one) = branch_query.get(&test_world, branch_one_id) {
            assert_eq!(branch_one.intersections_volume, PI * 11.0 / 12.0);
        }
        if let Ok(branch_two) = branch_query.get(&test_world, branch_two_id) {
            assert_eq!(branch_two.intersections_volume, PI * 22.0 / 12.0);
        }
        if let Ok(branch_three) = branch_query.get(&test_world, branch_three_id) {
            assert_eq!(branch_three.intersections_volume, PI * 11.0 / 12.0);
        }
    }


    fn test_add_branch_child(
        connections_query: &mut QueryState<&mut BranchConnectionData>,
        parent: Entity,
        new_child: Entity,
        world: &mut World,
    ) -> bool {
        if let Ok(mut parent_connections) = connections_query.get_mut(world, parent) {
            if parent_connections.children.0.is_none() {
                parent_connections.children.0 = Some(new_child);
            }
            else if parent_connections.children.1.is_none() {
                parent_connections.children.1 = Some(new_child);
            } else {return false;}
        }
    
        if let Ok(mut child_connections) = connections_query.get_mut(world, new_child) {
            child_connections.parent = Some(parent);
        }
    
        true
    }


    #[test]
    fn light_and_vigor_distribution_test() {
        let mut test_world = World::new();

        let branch_seven_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 5.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_six_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 90.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_five_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 7.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_four_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 8.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_three_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 24.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_two_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 19.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_one_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 3.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();
        

        let plant_id = test_world.spawn(PlantBundle {
            data: PlantData {
                root_node: Some(branch_one_id),
                ..Default::default()
            },
            growth_factors: PlantGrowthControlFactors {
                apical_control: 0.7,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let mut connections_query = test_world.query::<&mut BranchConnectionData>();
        test_add_branch_child(&mut connections_query, branch_one_id, branch_two_id, &mut test_world);
        test_add_branch_child(&mut connections_query, branch_one_id, branch_three_id, &mut test_world);
        test_add_branch_child(&mut connections_query, branch_two_id, branch_four_id, &mut test_world);
        test_add_branch_child(&mut connections_query, branch_three_id, branch_five_id, &mut test_world);
        test_add_branch_child(&mut connections_query, branch_three_id, branch_six_id, &mut test_world);
        test_add_branch_child(&mut connections_query, branch_six_id, branch_seven_id, &mut test_world);

        let mut test_schedule = Schedule::default();

        test_schedule.add_system(calculate_growth_vigor);
        test_schedule.run(&mut test_world);

        let mut branch_query = test_world.query::<&BranchGrowthData>();
        if let Ok(branch_one) = branch_query.get(&test_world, branch_one_id) {
            assert_eq!(branch_one.light_exposure, 20.0);
            assert_eq!(branch_one.growth_vigor, 20.0);
        }
        if let Ok(branch_two) = branch_query.get(&test_world, branch_two_id) {
            assert_eq!(branch_two.light_exposure, 8.0);
            assert_eq!(branch_two.growth_vigor, 40.0 / 9.0);
        }
        if let Ok(branch_three) = branch_query.get(&test_world, branch_three_id) {
            assert_eq!(branch_three.light_exposure, 12.0);
            assert_eq!(branch_three.growth_vigor, 140.0 / 9.0);
        }
        if let Ok(branch_four) = branch_query.get(&test_world, branch_four_id) {
            assert_eq!(branch_four.light_exposure, 8.0);
            assert_eq!(branch_four.growth_vigor, 40.0/ 9.0);
        }
        if let Ok(branch_five) = branch_query.get(&test_world, branch_five_id) {
            assert_eq!(branch_five.light_exposure, 7.0);
            assert_eq!(branch_five.growth_vigor, 1715.0 / 144.0);
        }
        if let Ok(branch_six) = branch_query.get(&test_world, branch_six_id) {
            assert_eq!(branch_six.light_exposure, 5.0);
            assert_eq!(branch_six.growth_vigor, (175.0 / 48.0) - 0.0000003); // the subract here is just due to a higher precision in the direct division, it's still correct
        }
        if let Ok(branch_seven) = branch_query.get(&test_world, branch_seven_id) {
            assert_eq!(branch_seven.light_exposure, 5.0);
            assert_eq!(branch_seven.growth_vigor, (175.0 / 48.0) - 0.0000003);
        }
        
    }

    fn test_get_terminal_branches(
        connections_query: &mut QueryState<&BranchConnectionData>,
        root_branch: Entity,
        world: &mut World,
    ) -> Vec<Entity> {
    
        let mut list: Vec<Entity> = vec![root_branch];

        let mut i = 0;
        loop {
            if i >= list.len() {break;}
            if let Ok(branch_connections) = connections_query.get(world, list[i]) {
                if branch_connections.children.0.is_none() {
                    i += 1;
                    continue;
                }
                list.push(branch_connections.children.0.unwrap());
                if branch_connections.children.1.is_some() {
                    list.push(branch_connections.children.1.unwrap());
                }
                list.remove(i);
            }
            
        }

        list
    }

    #[test]
    fn terminal_branches_test() {
        let mut test_world = World::new();

        let branch_seven_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 5.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_six_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 90.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_five_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 7.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_four_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 8.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_three_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 24.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_two_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 19.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_one_id = test_world.spawn(BranchBundle {
            growth_data: BranchGrowthData {
                light_exposure: 3.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();
        

        let plant_id = test_world.spawn(PlantBundle {
            data: PlantData {
                root_node: Some(branch_one_id),
                ..Default::default()
            },
            growth_factors: PlantGrowthControlFactors {
                apical_control: 0.7,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let mut connections_query = test_world.query::<&mut BranchConnectionData>();
        test_add_branch_child(&mut connections_query, branch_one_id, branch_two_id, &mut test_world);
        test_add_branch_child(&mut connections_query, branch_one_id, branch_three_id, &mut test_world);
        test_add_branch_child(&mut connections_query, branch_two_id, branch_four_id, &mut test_world);
        test_add_branch_child(&mut connections_query, branch_three_id, branch_five_id, &mut test_world);
        test_add_branch_child(&mut connections_query, branch_three_id, branch_six_id, &mut test_world);
        test_add_branch_child(&mut connections_query, branch_six_id, branch_seven_id, &mut test_world);

        let mut connections_query = test_world.query::<&BranchConnectionData>();
        let terminal_nodes = test_get_terminal_branches(&mut connections_query, branch_one_id, &mut test_world);

        assert!(terminal_nodes.contains(&branch_seven_id));
        assert!(terminal_nodes.contains(&branch_five_id));
        assert!(terminal_nodes.contains(&branch_four_id));
        assert!(!terminal_nodes.contains(&branch_three_id));
        assert!(!terminal_nodes.contains(&branch_six_id));
        assert!(!terminal_nodes.contains(&branch_one_id));
        assert!(!terminal_nodes.contains(&branch_two_id));
    }

    
}

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// Branch Nodes ////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod branch_nodes_tests {
    use super::*;

    fn test_add_node_child(
        connections_query: &mut QueryState<&mut BranchNodeConnectionData>,
        parent: Entity,
        new_child: Entity,
        world: &mut World,
    ){
        if let Ok(mut parent_connections) = connections_query.get_mut(world, parent) {
            parent_connections.children.push(new_child);
        }
    
        if let Ok(mut child_connections) = connections_query.get_mut(world, new_child) {
            child_connections.parent = Some(parent);
        }
    }

    fn test_get_terminal_nodes(
        connections_query: &mut QueryState<&BranchNodeConnectionData>,
        root_node: Entity,
        world: &mut World,
    ) -> Vec<Entity> {
    
        let mut list: Vec<Entity> = vec![root_node];
    
        let mut i = 0;
        loop {
            if i >= list.len() {break;}
            if let Ok(node_connections) = connections_query.get(world, list[i]) {
                if node_connections.children.len() == 0 {
                    i += 1;
                    continue;
                }
                for child_node_id in node_connections.children.iter() {
                    list.push(*child_node_id);
                }
                list.remove(i);
            }
            
        }
    
        list
    }
    

    #[test]
    fn terminal_nodes_test() {
        let mut test_world = World::new();

        let node_1_id = test_world.spawn(BranchNodeBundle::default()).id();
        let node_2_id = test_world.spawn(BranchNodeBundle::default()).id();
        let node_3_id = test_world.spawn(BranchNodeBundle::default()).id();
        let node_4_id = test_world.spawn(BranchNodeBundle::default()).id();
        let node_5_id = test_world.spawn(BranchNodeBundle::default()).id();
        let node_6_id = test_world.spawn(BranchNodeBundle::default()).id();
        let node_7_id = test_world.spawn(BranchNodeBundle::default()).id();
        let node_8_id = test_world.spawn(BranchNodeBundle::default()).id();
        let node_9_id = test_world.spawn(BranchNodeBundle::default()).id();
        let node_10_id = test_world.spawn(BranchNodeBundle::default()).id();


        let mut connections_query = test_world.query::<&mut BranchNodeConnectionData>();
        test_add_node_child(&mut connections_query, node_1_id, node_2_id, &mut test_world);
        test_add_node_child(&mut connections_query, node_1_id, node_3_id, &mut test_world);
        test_add_node_child(&mut connections_query, node_2_id, node_4_id, &mut test_world);
        test_add_node_child(&mut connections_query, node_3_id, node_5_id, &mut test_world);
        test_add_node_child(&mut connections_query, node_3_id, node_6_id, &mut test_world);
        test_add_node_child(&mut connections_query, node_6_id, node_7_id, &mut test_world);
        test_add_node_child(&mut connections_query, node_6_id, node_8_id, &mut test_world);
        test_add_node_child(&mut connections_query, node_6_id, node_9_id, &mut test_world);
        test_add_node_child(&mut connections_query, node_9_id, node_10_id, &mut test_world);

        let mut connections_query = test_world.query::<&BranchNodeConnectionData>();
        let terminal_nodes = test_get_terminal_nodes(&mut connections_query, node_1_id, &mut test_world);

        assert!(terminal_nodes.contains(&node_4_id));
        assert!(terminal_nodes.contains(&node_5_id));
        assert!(terminal_nodes.contains(&node_7_id));
        assert!(terminal_nodes.contains(&node_8_id));
        assert!(terminal_nodes.contains(&node_10_id));
        assert!(!terminal_nodes.contains(&node_1_id));
        assert!(!terminal_nodes.contains(&node_2_id));
        assert!(!terminal_nodes.contains(&node_3_id));
        assert!(!terminal_nodes.contains(&node_6_id));
        assert!(!terminal_nodes.contains(&node_9_id));
    }
}


///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// Branch Prototypes ///////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod branch_prototype_tests {
    use super::{BranchPrototypesSampler, BranchPrototypeRef, GenericImageView,
    World, BranchNodeTag};

    #[test]
    fn sampling_test() {
        let mut test_world = World::new();
        let random_entity = test_world.spawn(BranchNodeTag).id();

        let prototypes = BranchPrototypesSampler::create(
            vec![(BranchPrototypeRef::new(random_entity), [0, 200, 0], 0.4, 0.4),
            (BranchPrototypeRef::new(random_entity), [200, 0, 0], 0.0, 0.0),
            (BranchPrototypeRef::new(random_entity), [0, 0, 200], 0.8, 0.8),
            (BranchPrototypeRef::new(random_entity), [150, 0, 150], 0.4, 0.3)],
            (200, 200),
            1.0,
            1.0
        );

        let sample = prototypes.voronoi.get_pixel(50, 50);
        assert_eq!(sample, image::Rgba([150, 0, 150, 255]))
    }
}