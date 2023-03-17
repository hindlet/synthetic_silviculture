#![allow(dead_code, unused_variables, unused_imports)]
use std::{collections::HashMap};

use bevy_ecs::prelude::*;
use plotters::prelude::*;
use voronator::{
    delaunator::Point,
    CentroidDiagram,
    VoronoiDiagram,
};
use image::{GenericImageView, DynamicImage, ImageBuffer};
use rand::Rng;

use crate::branch::BranchBundle;


///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Branch Prototypes //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Component)]
pub struct BranchPrototypesTag;

#[derive(Resource)]
pub struct BranchPrototypesSampler {
    pub prototypes: HashMap<image::Rgba<u8>, BranchPrototypeRef>,
    pub voronoi: DynamicImage,
    pub max_apical: f32, 
    pub max_determinancy: f32,
}

#[derive(Component)]
pub struct BranchPrototypeData {
    pub mature_age: f32,

}

#[derive(Component)]
pub struct BranchPrototypeRef (pub Entity);

#[derive(Component)]
pub struct BranchNodePrototypeData {

}




///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Voronoi Diagram ////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


impl BranchPrototypesSampler {

    pub fn get_prototype(&self, apical: f32, determinancy: f32) -> Entity {

        let x: u32 = (apical * (self.voronoi.dimensions().0 as f32 / self.max_apical)).round() as u32;
        let y: u32 = (determinancy * (self.voronoi.dimensions().1 as f32 / self.max_determinancy)).round() as u32;
        
        self.prototypes.get(&self.voronoi.get_pixel(x, y)).unwrap().0
    }

    // I use a temporary image file to move the data from plotter to image, it's an awful solution but there didnt seem to be another way
    #[allow(unused_must_use)]
    pub fn create(
        prototype_data: Vec<(BranchPrototypeRef, [u8; 3], f32, f32)>,
        size: (u32, u32), 
        max_apical: f32, 
        max_determinancy: f32
    ) -> BranchPrototypesSampler {

        let mut sampler = BranchPrototypesSampler {
            prototypes: HashMap::new(),
            voronoi: DynamicImage::new_rgba8(size.0, size.1),
            max_apical,
            max_determinancy,
        };

        let mut points: Vec<Point> = vec![];
        let mut colors: Vec<[u8; 3]> = vec![];
        for (prototype, color, apical, determinancy) in prototype_data {
            sampler.prototypes.insert(image::Rgba([color[0], color[1], color[2], 255]), prototype);

            points.push(Point{x: apical as f64, y: determinancy as f64});
            colors.push(color);
        }


        let diagram = VoronoiDiagram::new(
            &Point {x: 0.0, y: 0.0},
            &Point {x: sampler.max_apical as f64, y: sampler.max_determinancy as f64},
            &points,
        )
        .unwrap();

        let mut image_buf: Vec<u8> = Vec::new();
        for _i in 0..(size.0 * size.1) * 3 {
            image_buf.push(0);
        }

        println!("{}", image_buf.len());
        println!("{}", size.0 * size.1);
        
        {
            let root = BitMapBackend::with_buffer(&mut image_buf, sampler.voronoi.dimensions()).into_drawing_area();
            root.fill(&WHITE);


            let root = root.apply_coord_spec(RangedCoord::<RangedCoordf32, RangedCoordf32>::new(
                0f32..sampler.max_apical as f32,
                0f32..sampler.max_determinancy as f32,
                (0..sampler.voronoi.dimensions().0 as i32, 0..sampler.voronoi.dimensions().1 as i32),
            ));


            for i in 0..points.len() {
                let p: Vec<(f32, f32)> = diagram.cells()[i].points().into_iter().map(|x| (x.x as f32, x.y as f32)).collect();
                let color = RGBColor{
                    0: colors[i][0],
                    1: colors[i][1],
                    2: colors[i][2],
                };

                let poly = Polygon::new(p.clone(), ShapeStyle{color: color.to_rgba(), filled: true, stroke_width: 2});
                root.draw(&poly);
            }

            root.present();
        }

        // sampler.voronoi = image::open(TEMP_IMAGE_PATH).unwrap();
        sampler.voronoi = DynamicImage::ImageRgb8(ImageBuffer::from_raw(size.0, size.1, image_buf).unwrap());

        sampler
    }

}

impl BranchPrototypeRef {
    pub fn new(entity: Entity) -> Self {
        BranchPrototypeRef(entity)
    }
}






