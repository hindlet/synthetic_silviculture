#![allow(dead_code, unused_variables, unused_imports)]
use std::{collections::HashMap};

use bevy_ecs::prelude::*;
use plotters::{prelude::*, palette::white_point::C};
use voronator::{
    delaunator::Point,
    CentroidDiagram,
    VoronoiDiagram,
};
use image::{GenericImageView, DynamicImage, ImageBuffer};
use rand::Rng;

use super::{
    branch::BranchBundle,
    super::maths::{
        vector_three::Vector3,
        bounding_sphere::BoundingSphere, matrix_three::Matrix3
    }
};


///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Branch Prototypes //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

#[derive(Resource)]
pub struct BranchPrototypesSampler {
    pub prototypes: HashMap<image::Rgba<u8>, usize>,
    pub voronoi: DynamicImage,
    pub max_apical: f32, 
    pub max_determinancy: f32,
}




#[derive(Component)]
pub struct BranchPrototypeRef (pub usize);


/// Struct for storing data about a branch prototype
/// 
/// #### Parameters:
/// - mature_age: the age at which the branch is fully grown
/// - layers: the number of layers in the tree the branch will have when fully grown
/// - node_counts: how many children each node on a layer of the tree has
/// - directions: a list of normalised directions that correspond to the node pairs generated by the get_nodes_and_connections_base_to_tip fn in branch_nodes
pub struct BranchPrototypeData {
    pub mature_age: f32, // the mature age of the branch, used to interpolate growth
    pub layers: u32, // the number of layers in the tree diagram, used for interpolating growth
    pub node_counts: Vec<Vec<u32>>, // the number of nodes in each layer of the tree diagram: len() == layers - 1
    pub directions: Vec<Vector3>, // the directions of all the node pairs
    cuml_nodes: Vec<u32>,
}


impl BranchPrototypeData {
    pub fn new(mature_age: f32, node_counts: Vec<Vec<u32>>, directions: Vec<[f32; 3]>) -> Self {
        let (layers, cuml_nodes) = {
            let mut layer_count = 1;
            let mut nodes = vec![1];
            for count_set in node_counts.iter() {
                layer_count += 1;
                let mut layer_node_count = 0;
                for count in count_set {
                    layer_node_count += count;
                }
                nodes.push(layer_node_count + nodes.last().unwrap());
            }
            (layer_count, nodes)
        };
        let directions = {
            let mut new_dirs: Vec<Vector3> = Vec::new();
            for dir in directions {
                new_dirs.push(dir.into())
            }
            new_dirs
        };


        BranchPrototypeData {
            mature_age,
            layers,
            node_counts,
            directions,
            cuml_nodes
        }
    }


    /// calculates and returns the possible bounds of the branch at max age for a set of possible rotations
    /// 
    /// used to decide on an optimal rotation for a new branch module
    pub fn get_possible_bounds(&self, max_length: f32, initial_rot: Vector3, possible_rots: &Vec<Vector3>, root_pos: Vector3) -> Vec<BoundingSphere>{
        
        // calculate the branch node positions at max age

        // this creates a list of all the directions from each node to its children
        let dirs = {
            let mut dirs = Vec::new();
            let mut count = 0;
            for layer_child_count in self.node_counts.iter() {

                for node_child_count in layer_child_count.iter() {
                    let mut child_dirs = Vec::new();
                    for _i in 0..*node_child_count {
                        child_dirs.push(self.directions[count]);
                        count += 1;
                    }
                    dirs.push(child_dirs);
                }
            }
            dirs
        };

        let node_positions = {
            let mut nodes: Vec<Vector3> = vec![Vector3::ZERO()];
            let mut i = 0;
            for dir_set in dirs {
                for dir in dir_set {
                    nodes.push(nodes[i] + dir * max_length);
                }
                i += 1;
            }
            nodes
        };

        // calculate possible normals
        let possible_normals = {
            let mut norms = Vec::new();
            for rot in possible_rots {
                norms.push(Vector3::euler_angles_to_direction(initial_rot + rot));
            }
            norms
        };

        // calculate all the rotations of it
        let possible_bounds = {
            let mut bounds = Vec::new();
            for norm in possible_normals {

                let branch_rotation_matrix = {
                    let mut rotation_axis = norm.cross(Vector3::Y());
                    rotation_axis.normalise();
                    let rotation_angle = norm.angle_to(Vector3::Y());
                    Matrix3::from_angle_and_axis(-rotation_angle, rotation_axis)
                };

                let new_nodes: Vec<Vector3> = node_positions.iter().map(|n| n.transform(branch_rotation_matrix)).collect();

                if new_nodes.len() == 1 {
                    bounds.push(BoundingSphere::new(new_nodes[0], 0.01))
                } else {
                    bounds.push(BoundingSphere::from_points(new_nodes))
                }
            }
            for bound in bounds.iter_mut() {
                bound.centre += root_pos;
            }
            bounds
        };

        possible_bounds
    }

    
}


#[derive(Resource)]
pub struct BranchPrototypes {
    pub prototypes: Vec<BranchPrototypeData>
}

impl BranchPrototypes {
    pub fn new(data: Vec<(f32, Vec<Vec<u32>>, Vec<[f32; 3]>)> ) -> Self{
        let mut prototypes = Vec::new();
        for (mature_age, node_counts, directions) in data {
            prototypes.push(BranchPrototypeData::new(mature_age, node_counts, directions));
        }
        BranchPrototypes {
            prototypes
        }
    }

    /// returns a set of directions
    pub fn get_directions(&self) -> Vec<&Vec<Vector3>> {
        let mut out: Vec<&Vec<Vector3>> = Vec::new();
        for prototype in self.prototypes.iter() {
            out.push(&prototype.directions);
        }
        out
    }

    pub fn get_age_layers_and_count(&self) -> Vec<(f32, u32, &Vec<Vec<u32>>)> {
        let mut out: Vec<(f32, u32, &Vec<Vec<u32>>)> = Vec::new();
        for prototype in self.prototypes.iter() {
            out.push((prototype.mature_age, prototype.layers, &prototype.node_counts));
        }
        out
    }

    pub fn get_ages(&self) -> Vec<f32> {
        let mut out: Vec<f32> = Vec::new();
        for prototype in self.prototypes.iter() {
            out.push(prototype.mature_age);
        }
        out
    }
}





///////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////// Voronoi Diagram ////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////


impl BranchPrototypesSampler {

    pub fn get_prototype_index(&self, apical: f32, determinancy: f32) -> usize {

        let x: u32 = (apical * (self.voronoi.height() as f32 / self.max_apical)).round() as u32 - 1;
        let y: u32 = (determinancy * (self.voronoi.width() as f32 / self.max_determinancy)).round() as u32 - 1;
        
        self.prototypes.get(&self.voronoi.get_pixel(x, y)).unwrap().clone()
    }

    #[allow(unused_must_use)]
    pub fn create(
        prototype_positions: Vec<(f32, f32)>,
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
        let mut current_colour: [u8; 3] = [0, 0, 0];
        for (apical, determinancy) in prototype_positions {
            points.push(Point{x: apical as f64, y: determinancy as f64});

            // generate and use a colour
            let colour = {
                let mut b = current_colour[2].overflowing_add(1);
                let mut g = if b.1 {b.0 = 0; current_colour[1].overflowing_add(1)} else {(current_colour[1], false)};
                let r = if g.1 {g.0 = 0; current_colour[0].overflowing_add(1)} else {(current_colour[0], false)};
                if r.1 {panic!("Too many branch prototypes in use - Maximum of 16777216")}
                [r.0, g.0, b.0]
            };
            current_colour = colour;
            sampler.prototypes.insert(image::Rgba([colour[0], colour[1], colour[2], 255]), sampler.prototypes.len());
            colors.push(colour);
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






