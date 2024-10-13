use maths::Vector2;
use synthetic_silviculture::VoronoiNoise;



fn main() {
    let noise = VoronoiNoise::new(vec![
        (0, Vector2::new(50.0, 200.0)),
        (1, Vector2::new(175.0, 50.0)),
        (2, Vector2::new(100.0, 100.0)),
        (3, Vector2::new(200.0, 200.0))
    ]);

    noise.export(300, 300);
}