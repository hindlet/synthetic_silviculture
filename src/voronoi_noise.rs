use core::f32;
use std::{fs::{File, OpenOptions}, io::Write};

use image::{ImageFormat, Rgb, RgbImage};
use maths::{ChaChaRng, Rng, SeedableRng, Vector2};



pub struct VoronoiNoise {
    points: Vec<(usize, Vector2)>
}

impl VoronoiNoise {
    /// points will be clamped to positive values
    pub fn new(
        mut points: Vec<(usize, Vector2)>
    ) -> Self{

        for (_, point) in points.iter_mut() {
            point.x = point.x.max(0.0);
            point.y = point.y.max(0.0);
        }


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

    /// write the voronoi diagram to an image file and generate a file with the colour key
    /// 
    /// width and height will be extended to contain at least all the points
    pub fn export(
        &self,
        width: u32,
        height: u32,
    ) {
        let mut random = ChaChaRng::seed_from_u64(567890);


        let mut width = width;
        let mut height = height;
        let mut colours: Vec<(Vector2, Rgb<u8>)> = Vec::new();

        for (_, point) in self.points.iter() {
            if point.x > width as f32 {width = point.x.ceil() as u32};
            if point.y > height as f32 {height = point.y.ceil() as u32};
            let colour = Rgb([random.gen(), random.gen(), random.gen()]);
            colours.push((*point, colour));
        }

        let mut image = RgbImage::new(width, height);


        for x in 0..width {
            for y in 0..height {
                let colour = Self::get_colour(Vector2::new(x as f32, y as f32), &colours);
                image.put_pixel(x, y, colour);
            }
        }

        let _ = image.save_with_format("assets/voronoi_output.png", ImageFormat::Png);

        let mut file = File::create("assets/voronoi_key.txt").unwrap();

        for i in 0..colours.len() {
            let _ = writeln!(file, "Point: {}  -  Position: {}  -  Colour: {:?}", self.points[i].0, colours[i].0, colours[i].1.0);
        }        
    }

    fn get_colour(
        point: Vector2,
        colours: &Vec<(Vector2, Rgb<u8>)>
    ) -> Rgb<u8>{
        let mut dist = f32::MAX;
        let mut current_colour: Rgb<u8> = Rgb([0, 0, 0]);

        for (target, colour) in colours.iter() {
            let new_dist = (point-*target).sqr_magnitude();
            if new_dist < 1.0 {
                return Rgb([0, 0, 0]);
            }
            if new_dist < dist {
                dist = new_dist;
                current_colour = *colour;
            }
        }
        current_colour
    }

}