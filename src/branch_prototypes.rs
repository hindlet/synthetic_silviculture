#![allow(dead_code, unused_variables, unused_imports)]
use std::{collections::HashMap};

use bevy_ecs::prelude::*;
use plotters::prelude::*;
use voronator::{
    delaunator::Point,
    CentroidDiagram,
    VoronoiDiagram,
};
use image::GenericImageView;
use rand::Rng;

use crate::branch::BranchBundle;

const SAVE_IMAGE_PATH: &str = "assets/voronoi.png";
const SAVE_IMAGE_SIZE: (u32, u32) = (1000, 1000);

const TEMP_IMAGE_PATH: &str = "assets/temp_voronoi.png";


///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Branch Prototypes //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Component)]
pub struct BranchPrototypesTag;

#[derive(Default, Component)]
pub struct BranchPrototypes {
    pub prototypes: HashMap<image::Rgba<u8>, BranchBundle>,
    pub voronoi: image::DynamicImage,
    pub max_apical: f32,
    pub max_determinancy: f32,
}


#[derive(Default, Bundle)]
pub struct BranchPrototypesBundle {
    pub tag: BranchPrototypesTag,
    pub branch_prototypes: BranchPrototypes,

}




///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Voronoi Diagram ////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


impl BranchPrototypes {
    pub fn new() -> Self {
        BranchPrototypes {
            prototypes: HashMap::new(),
            voronoi: image::DynamicImage::new_rgba8(200, 200),
            max_apical: 1.0,
            max_determinancy: 1.0,
        }
    }

    pub fn get_prototype(&self, apical: f32, determinancy: f32) {

        let x: u32 = (apical * (self.voronoi.dimensions().0 as f32 / self.max_apical)).round() as u32;
        let y: u32 = (determinancy * (self.voronoi.dimensions().1 as f32 / self.max_determinancy)).round() as u32;
        
        println!("{:?}", self.voronoi.get_pixel(x, y));
    }

    // I use a temporary image file to move the data from plotter to image, it's an awful solution but there didnt seem to be another way
    #[allow(unused_must_use)]
    pub fn setup(&mut self, prototype_data: Vec<(BranchBundle, [u8; 3], f32, f32)>) {

        let mut points: Vec<Point> = vec![];
        let mut colors: Vec<[u8; 3]> = vec![];
        for (branch, color, apical, determinancy) in prototype_data {
            self.prototypes.insert(image::Rgba([color[0], color[1], color[2], 255]), branch);

            points.push(Point{x: apical as f64, y: determinancy as f64});
            colors.push(color);
        }


        let diagram = VoronoiDiagram::new(
            &Point {x: 0.0, y: 0.0},
            &Point {x: self.max_apical as f64, y: self.max_determinancy as f64},
            &points,
        )
        .unwrap();


        let root = BitMapBackend::new(TEMP_IMAGE_PATH, self.voronoi.dimensions()).into_drawing_area();
        root.fill(&WHITE);


        let root = root.apply_coord_spec(RangedCoord::<RangedCoordf32, RangedCoordf32>::new(
            0f32..self.max_apical as f32,
            0f32..self.max_determinancy as f32,
            (0..self.voronoi.dimensions().0 as i32, 0..self.voronoi.dimensions().1 as i32),
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

        self.voronoi = image::open(TEMP_IMAGE_PATH).unwrap();
    }
}






