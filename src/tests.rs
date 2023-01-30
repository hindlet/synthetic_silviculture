#![allow(dead_code, unused_variables, unused_imports)]
use bevy_ecs::prelude::*;
use crate::*;
use std::f32::consts::PI;

///////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////// Bounding Sphere  //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod bounding_sphere_tests {
    use super::{Vector3, BoundingSphere, PI};

    #[test]
    fn axis_furthest_point_test() {
        let sphere = BoundingSphere{
            centre: Vector3 { x: 0.0, y: 5.0, z: 0.0 },
            radius: 2.5,
        };
        let point = Vector3 {x: 0.0, y: 0.0, z: 0.0};

        let furthest_point = sphere.furthest_point_from_point(&point);

        assert_eq!(furthest_point, Vector3 {x: 0.0, y: 7.5, z: 0.0})
    }

    #[test]
    fn non_axis_furthest_point_test() {
        let sphere = BoundingSphere{
            centre: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            radius: 2.5,
        };
        let point = Vector3 {x: -2.5, y: 0.0, z: 0.0};

        let furthest_point = sphere.furthest_point_from_point(&point);

        assert_eq!(furthest_point, Vector3 {x: 2.5, y: 0.0, z: 0.0})
    }

    #[test]
    fn zero_points_test() {
        let points: Vec<Vector3> = vec![];

        assert_eq!(BoundingSphere::from_points(&points), BoundingSphere::new())
    }

    #[test]
    fn defined_bounds_test() {
        let points: Vec<Vector3> = vec![
            Vector3{x: 0.0, y: 0.0, z: 0.0},
            Vector3{x: 0.0, y: 2.5, z:0.0},
            Vector3{x: 0.0, y: 1.5, z: 1.5},
        ];
        let mut bounds = BoundingSphere::new();
        bounds.radius = 2.5;
        assert_eq!(bounds.contains_points(&points), true)
    }

    #[test]
    fn calculated_bounds_test() {
        let points: Vec<Vector3> = vec![
            Vector3{x: 0.0, y: 0.0, z: 0.0},
            Vector3{x: 0.0, y: 2.5, z:0.0},
            Vector3{x: 0.0, y: 1.5, z: 1.5},
        ];
        let bounds = BoundingSphere::from_points(&points);
        assert_eq!(bounds.contains_points(&points), true)
    }

    #[test]
    fn intersection_test() {
        let sphere_one = BoundingSphere {
            centre: Vector3::new(),
            radius: 5.0,
        };
        let sphere_two = BoundingSphere {
            centre: Vector3 {x: 5.0, y: 0.0, z: 0.0},
            radius: 2.0,
        };

        assert_eq!(sphere_one.is_intersecting_sphere(&sphere_two), true)
    }

    #[test]
    fn touching_test() {
        let sphere_one = BoundingSphere {
            centre: Vector3::new(),
            radius: 5.0,
        };
        let sphere_two = BoundingSphere {
            centre: Vector3 {x: 10.0, y: 0.0, z: 0.0},
            radius: 5.0,
        };

        assert_eq!(sphere_one.is_intersecting_sphere(&sphere_two), false)
    }

    #[test]
    fn non_intersection_test() {
        let sphere_one = BoundingSphere {
            centre: Vector3::new(),
            radius: 5.0,
        };
        let sphere_two = BoundingSphere {
            centre: Vector3 {x: 10.0, y: 0.0, z: 0.0},
            radius: 2.0,
        };

        assert_eq!(sphere_one.is_intersecting_sphere(&sphere_two), false)
    }

    #[test]
    fn intersection_volume_test() {
        let sphere_one = BoundingSphere {
            centre: Vector3::new(),
            radius: 2.0,
        };
        let sphere_two = BoundingSphere {
            centre: Vector3 {x: -2.0, y: 2.0, z: -1.0},
            radius: 2.0,
        };

        assert_eq!(sphere_one.get_intersection_volume(&sphere_two), PI * 11.0 / 12.0)
    }

}

///////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////// Bounding Box  /////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


#[cfg(test)]
mod bounding_box_tests {
    use super::{Vector3, BoundingBox, BoundingSphere};

    #[test]
    fn zero_points_test() {
        let points: Vec<Vector3> = vec![];

        assert_eq!(BoundingBox::from_points(&points), BoundingBox::new())
    }

    #[test]
    fn defined_bounds_point_test() {
        let points: Vec<Vector3> = vec![
            Vector3{x: 0.0, y: 0.0, z: 0.0},
            Vector3{x: 0.0, y: 2.5, z:0.0},
            Vector3{x: 5.0, y: 1.5, z: 1.5},
        ];
        let bounds = BoundingBox {
            pos: Vector3 {x: 0.0, y: 0.0, z: 0.0},
            width: 5.0,
            height: 2.5,
            depth: 1.5,
        };
        assert_eq!(bounds.contains_points(&points), true)
    }

    #[test]
    fn calculated_bounds_point_test() {
        let points: Vec<Vector3> = vec![
            Vector3{x: 0.0, y: 0.0, z: 0.0},
            Vector3{x: 0.0, y: 2.5, z:0.0},
            Vector3{x: 5.0, y: 1.5, z: 1.5},
        ];
        let bounds = BoundingBox::from_points(&points);
        assert_eq!(bounds.contains_points(&points), true)
    }

    #[test]
    fn defined_bounds_sphere_test() {
        let spheres: Vec<BoundingSphere> = vec![
            BoundingSphere {
                centre: Vector3{x: 0.0, y: 0.0, z: 0.0},
                radius: 5.0,
            },
            BoundingSphere {
                centre: Vector3{x: 0.0, y: 25.0, z:0.0},
                radius: 7.0,
            },
            BoundingSphere {
                centre: Vector3{x: 5.0, y: 15.0, z: 15.0},
                radius: 2.0,
            },
        ];
        let bounds = BoundingBox {
            pos: Vector3 {x: -7.0, y: -5.0, z: -7.0},
            width: 14.0,
            height: 37.0,
            depth: 24.0,
        };
        assert_eq!(bounds.contains_spheres(&spheres), true)
    }

    #[test]
    fn calculated_bounds_sphere_test() {
        let spheres: Vec<BoundingSphere> = vec![
            BoundingSphere {
                centre: Vector3{x: 0.0, y: 0.0, z: 0.0},
                radius: 5.0,
            },
            BoundingSphere {
                centre: Vector3{x: 0.0, y: 25.0, z:0.0},
                radius: 7.0,
            },
            BoundingSphere {
                centre: Vector3{x: 5.0, y: 15.0, z: 15.0},
                radius: 2.0,
            },
        ];
        let bounds = BoundingBox::from_spheres(&spheres);

        assert_eq!(bounds.contains_spheres(&spheres), true)
    }

    #[test]
    fn intersection_test() {
        let box_one = BoundingBox {
            pos: Vector3 {x: 0.0, y: 0.0, z: 0.0},
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let box_two = BoundingBox {
            pos: Vector3 {x: 2.5, y: 2.5, z: 2.5},
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert_eq!(box_one.is_intersecting_box(&box_two), true)
    }

    #[test]
    fn non_intersection_test() {
        let box_one = BoundingBox {
            pos: Vector3 {x: 0.0, y: 0.0, z: 0.0},
            width: 2.0,
            height: 2.0,
            depth: 2.0,
        };
        let box_two = BoundingBox {
            pos: Vector3 {x: 2.5, y: 2.5, z: 2.5},
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert_eq!(box_one.is_intersecting_box(&box_two), false)
    }
}


///////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////// Plants  ///////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod plant_tests {

    use super::{World, PlantBundle, PlantTag, Vector3, BoundingBox,
        StageLabel, SystemStage, Schedule, update_plant_intersections,
        Stage, With, Query, update_branch_intersections, BranchBundle, BoundingSphere,
        BranchTag, BranchNodes, BranchData, BranchList,
        update_plant_bounds, PlantData
    };


    #[test]
    fn intersections_test() {
        let mut test_world = World::new();

        test_world.spawn(PlantBundle {
            tag: PlantTag,
            branches: BranchList {branches: Vec::new(), connections: vec![]},
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
            branches: super::BranchList {branches: Vec::new(), connections: vec![]},
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
            branches: super::BranchList {branches: Vec::new(), connections: vec![]},
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
            nodes: BranchNodes {nodes: Vec::new(), connections: vec![]},
            data: BranchData::new(),
        })
        .id();

        let branch_two = test_world.spawn(BranchBundle {
            tag: BranchTag,
            bounds: BoundingSphere {
                centre: Vector3{x: 5.0, y: 2.0, z: 7.0},
                radius: 5.0,
            },
            nodes: BranchNodes {nodes: Vec::new(), connections: vec![]},
            data: BranchData::new(),
        })
        .id();

        let branch_three = test_world.spawn(BranchBundle {
            tag: BranchTag,
            bounds: BoundingSphere {
                centre: Vector3{x: 12.0, y: 3.0, z: 15.0},
                radius: 6.0,
            },
            nodes: BranchNodes {nodes: Vec::new(), connections: vec![]},
            data: BranchData::new(),
        })
        .id();


        test_world.spawn(PlantBundle {
            tag: PlantTag,
            branches: super::BranchList {branches: vec![branch_one, branch_two], connections: vec![]},
            bounds: BoundingBox::new(),
            data: PlantData::new(),
        });

        test_world.spawn(PlantBundle {
            tag: PlantTag,
            branches: super::BranchList {branches: vec![branch_three], connections: vec![]},
            bounds: BoundingBox::new(),
            data: PlantData::new(),
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
///////////////////////////// Branches ////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod branch_tests {

    
}

///////////////////////////////////////////////////////////////////////////////////////
///////////////////////////// Branch Prototypes ///////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod prototype_tests {
    use super::{BranchPrototypes, BranchBundle, GenericImageView};

    #[test]
    fn sampling_test() {
        let mut prototypes = BranchPrototypes::new();
        prototypes.setup(vec![
            (BranchBundle::new(), [0, 200, 0], 0.4, 0.4),
            (BranchBundle::new(), [200, 0, 0], 0.0, 0.0),
            (BranchBundle::new(), [0, 0, 200], 0.8, 0.8),
        ]);

        let sample = prototypes.voronoi.get_pixel(50, 50);
        assert_eq!(sample, image::Rgba([0, 200, 0, 255]))
    }
}