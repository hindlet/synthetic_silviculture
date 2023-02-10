#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use image::{GenericImageView};
use crate::{
    vector_three::Vector3,
    bounding_sphere::BoundingSphere,
    bounding_box::BoundingBox,
    plant::*,
    branch::*,
    branch_prototypes::BranchPrototypes,
};
use std::f32::consts::PI;



///////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////// Plants  ///////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod plant_bounds_tests {

    use super::{World, PlantBundle, PlantTag, Vector3, BoundingBox,
        StageLabel, SystemStage, Schedule, update_plant_intersections,
        Stage, With, Query, BranchBundle, BoundingSphere,
        BranchTag, BranchData, update_branch_intersections,
        update_plant_bounds, PlantData, BranchConnectionData
    };


    #[test]
    fn intersections_test() {
        let mut test_world = World::new();

        test_world.spawn(PlantBundle {
            tag: PlantTag,
            bounds: BoundingBox {
                pos: Vector3::new(),
                width: 5.0,
                height: 7.0,
                depth: 3.0,
            },
            data: PlantData::new(),
        });

        test_world.spawn(PlantBundle {
            tag: PlantTag,
            bounds: BoundingBox {
                pos: Vector3 {x: 2.0, y: 5.0, z: 2.0},
                width: 5.0,
                height: 7.0,
                depth: 3.0,
            },
            data: PlantData::new(),
        });

        test_world.spawn(PlantBundle {
            tag: PlantTag,
            bounds: BoundingBox {
                pos: Vector3 {x: 6.0, y: 5.0, z: 2.0},
                width: 5.0,
                height: 7.0,
                depth: 3.0,
            },
            data: PlantData::new(),
        });

        #[derive(StageLabel)]
        pub struct TestRunLabel;


        

        let mut test_schedule = Schedule::default();

        test_schedule.add_stage(TestRunLabel, SystemStage::parallel());
        
        test_schedule.add_system_to_stage(TestRunLabel, update_plant_intersections);
        
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
            tag: BranchTag,
            bounds: BoundingSphere {
                centre: Vector3::new(),
                radius: 5.0,
            },
            data: BranchData::default(),
            connections: BranchConnectionData::default(),
        })
        .id();

        let branch_two = test_world.spawn(BranchBundle {
            tag: BranchTag,
            bounds: BoundingSphere {
                centre: Vector3{x: 5.0, y: 2.0, z: 7.0},
                radius: 5.0,
            },
            data: BranchData::default(),
            connections: BranchConnectionData {
                children: (Some(branch_one), None),
                parent: None,
            }
        })
        .id();

        let branch_three = test_world.spawn(BranchBundle {
            tag: BranchTag,
            bounds: BoundingSphere {
                centre: Vector3{x: 12.0, y: 3.0, z: 15.0},
                radius: 6.0,
            },
            data: BranchData::default(),
            connections: BranchConnectionData::default(),
        })
        .id();


        test_world.spawn(PlantBundle {
            tag: PlantTag,
            bounds: BoundingBox::new(),
            data: PlantData {
                root_node: Some(branch_two),
                ..Default::default()
            },
        });

        test_world.spawn(PlantBundle {
            tag: PlantTag,
            bounds: BoundingBox::new(),
            data: PlantData {
                root_node: Some(branch_three),
                ..Default::default()
            },
        });



        let mut test_schedule = Schedule::default();

        #[derive(StageLabel)]
        pub struct StageOne;
        #[derive(StageLabel)]
        pub struct StageTwo;
        #[derive(StageLabel)]
        pub struct StageThree;


        test_schedule.add_stage(StageOne, SystemStage::parallel());
        test_schedule.add_stage(StageTwo, SystemStage::parallel());
        test_schedule.add_stage(StageThree, SystemStage::parallel());
        
        test_schedule.add_system_to_stage(StageOne, update_plant_bounds);
        test_schedule.add_system_to_stage(StageTwo, update_plant_intersections);
        test_schedule.add_system_to_stage(StageThree, update_branch_intersections);
        
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

    use super::{BranchPrototypes, BranchBundle, BranchConnectionData,
    PlantBundle, Query, BranchData, BoundingSphere, With, BranchTag, Entity,
    calculate_branch_light_exposure, Vector3, World, Schedule,
    StageLabel, SystemStage, Stage, calculate_branch_intersection_volumes, PI,
    QueryState, calculate_growth_vigor, PlantData};

    /// this function is for testing purposes,
    /// it checks every branch intersecting with every other branch
    /// this means it's super slow at large scales
    fn testing_branch_intersections(
        mut branch_query: Query<(&mut BranchData, &BoundingSphere, Entity), With<BranchTag>>,
    ) {
        // reset intersections list
        for (mut data, sphere, entity) in &mut branch_query {
            data.intersection_list = Vec::new();
        }

        // check intersections
        let mut combinations = branch_query.iter_combinations_mut();
        while let Some([mut branch_one, branch_two]) = combinations.fetch_next() {
            if branch_one.1.is_intersecting_sphere(&branch_two.1) {
                branch_one.0.intersection_list.push(branch_two.2);
            }
        }
    }

    #[test]
    fn intersection_volume_test() {
        let mut test_world = World::new();

        let branch_one_id = test_world.spawn(BranchBundle {
            tag: BranchTag,
            bounds: BoundingSphere {
                centre: Vector3::new(),
                radius: 2.0,
            },
            data: BranchData::default(),
            connections: BranchConnectionData::default(),
        })
        .id();

        let branch_two_id = test_world.spawn(BranchBundle {
            tag: BranchTag,
            bounds: BoundingSphere {
                centre: Vector3{x: -2.0, y: 2.0, z: -1.0},
                radius: 2.0,
            },
            data: BranchData::default(),
            connections: BranchConnectionData::default()
        })
        .id();

        let branch_three_id = test_world.spawn(BranchBundle {
            tag: BranchTag,
            bounds: BoundingSphere {
                centre: Vector3{x: -4.0, y: 4.0, z: -2.0},
                radius: 2.0,
            },
            data: BranchData::default(),
            connections: BranchConnectionData::default(),
        })
        .id();

        let mut test_schedule = Schedule::default();
        #[derive(StageLabel)]
        pub struct StageOne;
        test_schedule.add_stage(StageOne, SystemStage::parallel());
        #[derive(StageLabel)]
        pub struct StageTwo;
        test_schedule.add_stage(StageTwo, SystemStage::parallel());

        test_schedule.add_system_to_stage(StageOne, testing_branch_intersections);
        test_schedule.add_system_to_stage(StageTwo, calculate_branch_intersection_volumes);
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
            data: BranchData {
                light_exposure: 5.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_six_id = test_world.spawn(BranchBundle {
            data: BranchData {
                light_exposure: 90.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_five_id = test_world.spawn(BranchBundle {
            data: BranchData {
                light_exposure: 7.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_four_id = test_world.spawn(BranchBundle {
            data: BranchData {
                light_exposure: 8.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_three_id = test_world.spawn(BranchBundle {
            data: BranchData {
                light_exposure: 24.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_two_id = test_world.spawn(BranchBundle {
            data: BranchData {
                light_exposure: 19.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let branch_one_id = test_world.spawn(BranchBundle {
            data: BranchData {
                light_exposure: 3.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();
        

        let plant_id = test_world.spawn(PlantBundle {
            data: PlantData {
                root_node: Some(branch_one_id),
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
        #[derive(StageLabel)]
        pub struct StageOne;
        test_schedule.add_stage(StageOne, SystemStage::parallel());
        #[derive(StageLabel)]
        pub struct StageTwo;
        test_schedule.add_stage(StageTwo, SystemStage::parallel());

        test_schedule.add_system_to_stage(StageOne, calculate_growth_vigor);
        test_schedule.run(&mut test_world);

        let mut branch_query = test_world.query::<&BranchData>();
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

    
}

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// Branch Prototypes ///////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod branch_prototype_tests {
    use super::{BranchPrototypes, BranchBundle, GenericImageView};

    #[test]
    fn sampling_test() {
        let mut prototypes = BranchPrototypes::new();
        prototypes.setup(vec![
            (BranchBundle::default(), [0, 200, 0], 0.4, 0.4),
            (BranchBundle::default(), [200, 0, 0], 0.0, 0.0),
            (BranchBundle::default(), [0, 0, 200], 0.8, 0.8),
            (BranchBundle::default(), [150, 0, 150], 0.4, 0.3)
        ]);

        let sample = prototypes.voronoi.get_pixel(50, 50);
        assert_eq!(sample, image::Rgba([150, 0, 150, 255]))
    }
}