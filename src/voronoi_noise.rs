use maths::Vector2;



pub struct VoronoiNoise {
    points: Vec<(usize, Vector2)>
}

impl VoronoiNoise {
    pub fn new(
        points: Vec<(usize, Vector2)>
    ) -> Self{

        VoronoiNoise {
            points
        }
    }

    /// returns the id of the closest voronoi point to the given inout point
    pub fn get_closest(
        &self,
        point: Vector2
    ) -> usize {
        let mut min: (f32, usize) = (f32::MAX, 0);
        for (id, target) in self.points.iter() {
            let sqr_dist = (point - *target).sqr_magnitude();
            if sqr_dist < min.0 {
                min = (sqr_dist, *id);
            }
        }

        min.1
    }
}