use synthetic_silviculture::{
    apps::graphics_app::*,
    GrowthControlSettingParams, PlasticitySettingParams
};


fn main() {
    let branch_types = vec![
        (
            0.1,
            vec![vec![2], vec![1, 2], vec![2, 1, 2]],
            vec![
                [0.743, 0.371, 0.557],
                [0.192, 0.962, 0.192],

                [0.557, 0.743, 0.371],
                [0.236, 0.943, 0.236],
                [0.588, 0.784, 0.196],

                [0.802, 0.535, 0.267],
                [-0.535, 0.267, 0.802],
                [-0.302, 0.905, 0.302],
                [-0.333, 0.667, -0.667],
                [0.301, 0.904, 0.301],
            ],
        )
    ];
    let branch_conditions = (vec![(10.0, 10.0)], 20.0, 20.0);

    let plant_species = vec![
        (
            (GrowthControlSettingParams{
            max_age: 40.0,
            max_vigor: 10.0,
            min_vigor: 0.5,
            apical_control: 0.62,
            growth_rate: 0.19,
            tropism_time_control: 0.38,
            max_branch_segment_length: 1.0,
            branch_segment_length_scaling_coef: 1.0,
            tropism_angle_weight: 0.37,
            branching_angle: 0.52,
            thickening_factor: 0.05,
            },
            PlasticitySettingParams {
                seeding_frequency: 0.5,
                seeding_radius: 10.0,
                shadow_tolerance: 1.0,
                flowering_age: 15.0
            }),
            (18.0, 5.0, 90.0, 15.0)
        )
    ];


    let app = GraphicsTreeApp::new("plant_growth_example".into())
        .set_branch_presets(branch_types, branch_conditions)
        .set_shadow_cell_data(0.5, 3)
        .set_plant_death_rate(0.1)
        .set_time_step(5.0)
        .with_branch_graphics_gui()
        .set_light(([1.0, -1.0, 1.0], 1.0))
        .set_initial_plant_num(1)
        .set_plant_species(plant_species)
        .set_environmental_parameters((20.0, 0.1), 100.0)
        .with_flat_terrain(5.0, [0.0, 0.0, 0.0], [0.0, 154.0 / 255.0, 23.0 / 255.0])
        .set_branch_mesh_settings(7, false)
        .build();

    app.run();
}