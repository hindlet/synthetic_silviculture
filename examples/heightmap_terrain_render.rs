use synthetic_silviculture::apps::graphics_app::*;


fn main() {
    const GRASS_COLOUR: [f32; 3] = [0.0, 0.604, 0.090];
    const ROCK_COLOUR: [f32; 3] = [0.502, 0.518, 0.529];
    // these two need to be in range 0->1
    const GRASS_SLOPE_THRESHOLD: f32 = 0.1;
    const GRASS_BLEND_AMOUNT: f32 = 1.0;

    let app = GraphicsTreeApp::new("heightmap_terrain_example".into())
        .with_heightmap_terrain(100.0, [0.0, 0.0, 0.0], 50, 10.0, "assets/Noise_Texture.png".into(), GRASS_COLOUR, ROCK_COLOUR, GRASS_SLOPE_THRESHOLD, GRASS_BLEND_AMOUNT)
        .set_lights(Vec::new(), vec![([2.0, -1.0, 0.0], 1.0)])
        .build();

    app.run();
}