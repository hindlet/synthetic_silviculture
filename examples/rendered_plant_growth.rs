use synthetic_silviculture::apps::graphics_app::*;


fn main() {
    let branch_types = vec![
        (
            25.0,
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


    let app = GraphicsTreeApp::new("plant_growth_example".into())
        .set_branch_presets(branch_types, branch_conditions)
        .set_shadow_cell_data(0.5, 3)
        .set_plant_death_rate(0.5)
        .set_time_step(0.75)
        .with_branch_graphics_gui()
        .set_lights(Vec::new(), vec![([1.0, -0.3, 0.0], 1.0)])
        .build();

    app.run();
}