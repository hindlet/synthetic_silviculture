#![allow(dead_code, unused_variables, unused_imports)]
use std::collections::HashMap;
use crate::maths::vector_three_int::Vector3Int;
use bevy_ecs::prelude::*;


#[derive(Resource, Debug)]
/// LightCells stores shadow data in a grid of cube units for growth vigor calulcations
/// 
/// Components:
///     - cells: a hash map using cell coordinates as a key and storing the cells own shadow volume and the shadow volume gained from above
///     - check_height: The maximum value up or down it will check for a cell existing without finding one before stopping, a value less than 5 should be fine
///     - cell_size: The side length of each cell in m
pub struct LightCells {
    // cells hash map, set out as Hashmap<id, (own_shadow_volume, upwards_shadow_volume)>
    cells: HashMap<Vector3Int, (f32, f32)>,
    check_height: i32,
    cell_size: f32,
}


impl LightCells {

    /// creates a new light_cells_resource with a given check height
    pub fn new(check_height: i32, cell_size: f32) -> Self{
        LightCells {
            cells: HashMap::new(),
            check_height,
            cell_size: cell_size.abs(),
        }
    }

    pub fn size(&self) -> f32 {
        self.cell_size
    }

    /// sets all the cell shadow values to 0
    pub fn set_all_zero(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.1.0 = 0.0;
            cell.1.1 = 0.0;
        }
    }

    /// adds a new cell to the grid, updating it's parent shadow and child shadow
    fn add_cell(&mut self, id: Vector3Int, shadow_volume: f32) {
        let parent_volume = match self.cells.get(&(id + Vector3Int::Y())) {
            Some(shadow_data) => shadow_data.0 + shadow_data.1,
            None => 0.0
        };
        self.cells.insert(id, (shadow_volume, parent_volume));
        self.propogate_down(id);
    }


    /// increases the shadow volume of a given cell, or creates one if it does not exist
    pub fn add_volume_to_cell(&mut self, id: impl Into<Vector3Int>, additional_volume: f32) {
        let id: Vector3Int = id.into();

        match self.cells.get_mut(&id) {
            Some(shade_data) => {shade_data.0 += additional_volume; self.propogate_down(id);},
            None => {self.add_cell(id, additional_volume);}
        }
    }

    /// returns the shadow value of a given cell, if it does not exist but a cell above within the check range exists, it's shadow value is taken
    /// 
    /// light_value = exp(-total_cell_shadow_volume)
    pub fn get_cell_light(&self, id: impl Into<Vector3Int>) -> f32 {
        let id: Vector3Int = id.into();

        match self.cells.get(&id) {
            Some(shade_data) => return (-shade_data.0 - shade_data.1).exp(),
            None => ()
        }

        let mut count = 1;
            
        loop {
            if count > self.check_height {break 1.0;}

            let next_id = id + Vector3Int::Y() * count;

            match self.cells.get(&next_id) {
                Some(shade_data) => {break (-shade_data.0 - shade_data.1).exp();},
                None => ()
            }

            count += 1;
        }
    }

    /// propogates shadow down the cells from a parent, does nothing if the start id does not exist
    fn propogate_down(&mut self, start_id: Vector3Int) {
        

        let mut current_shadow_volume = match self.cells.get(&start_id) {
            Some(shade_data) => shade_data.0 + shade_data.1,
            None => return,
        };

        let mut count = 1;
        let mut dist = 1;
        loop {
            if count > self.check_height {break;}

            let next_id = start_id - Vector3Int::Y() * dist;

            match self.cells.get_mut(&next_id) {
                Some(shade_data) => {shade_data.1 = current_shadow_volume; current_shadow_volume += shade_data.0; count = 0;},
                None => count += 1
            }

            dist += 1;
        }
    }
}



#[cfg(test)]
mod light_cells_tests{
    use super::{LightCells};
    use crate::maths::vector_three::Vector3;

    #[test]
    fn unknown_cell_test() {
        let cells = LightCells::new(0, 1.0);

        assert_eq!(cells.get_cell_light([0, 0, 0]), 1.0);
    }

    #[test]
    fn above_cell_test() {
        let mut cells = LightCells::new(2, 1.0);

        cells.add_volume_to_cell([0, 2, 0], 2.0);
        
        assert_eq!(cells.get_cell_light([0, 0, 0]), (-2.0_f32).exp());
    }

    #[test]
    fn known_cell_test() {
        let mut cells = LightCells::new(0, 1.0);

        cells.add_volume_to_cell([0, 0, 0], 2.0);

        assert_eq!(cells.get_cell_light([0, 0, 0]), (-2.0_f32).exp());
    }

    #[test]
    fn propogation_test() {
        let mut cells = LightCells::new(2, 1.0);

        cells.add_volume_to_cell([0, 0, 0], 0.0);

        cells.add_volume_to_cell([0, 2, 0], 2.0);

        assert_eq!(cells.get_cell_light([0, 0, 0]), (-2.0_f32).exp());
    }

    #[test]
    fn non_propogation_test() {
        let mut cells = LightCells::new(1, 1.0);

        cells.add_volume_to_cell([0, 0, 0], 0.0);

        cells.add_volume_to_cell([0, 2, 0], 2.0);

        assert_eq!(cells.get_cell_light([0, 0, 0]), 1.0);
    }

    #[test]
    fn multiple_propogation_test() {
        let mut cells = LightCells::new(3, 1.0);

        cells.add_volume_to_cell([0, 0, 0], 1.0);

        cells.add_volume_to_cell([0, 2, 0], 2.0);

        cells.add_volume_to_cell([0, 4, 0], 3.0);

        cells.add_volume_to_cell([0, 7, 0], 1.5);

        assert_eq!(cells.get_cell_light([0, 4, 0]), (-4.5_f32).exp());
        assert_eq!(cells.get_cell_light([0, 2, 0]), (-6.5_f32).exp());
        assert_eq!(cells.get_cell_light([0, 0, 0]), (-7.5_f32).exp());
    }

    #[test]
    fn shadow_by_volume_test() {
        let mut cells = LightCells::new(0, 1.0);
        let id = [0, 0, 0];

        assert_eq!(cells.get_cell_light(id), 1.0);
        cells.add_volume_to_cell(id, 1.0);
        assert_eq!(cells.get_cell_light(id), (-1.0_f32).exp());
        cells.add_volume_to_cell(id, 5.0);
        assert_eq!(cells.get_cell_light(id), (-6.0_f32).exp());
    }

    #[test]
    fn size_tests() {
        let mut cells = LightCells::new(0, 0.5);
        cells.add_volume_to_cell([0, 0, 0], 1.0);

        assert_eq!(cells.get_cell_light(Vector3::Y()), 1.0);
        assert_eq!(cells.get_cell_light(Vector3::new(0.0, 0.4, 0.0) / cells.size()), (-1.0_f32).exp());

        let mut cells = LightCells::new(0, 2.0);
        cells.add_volume_to_cell([0, 0, 0], 1.0);

        assert_eq!(cells.get_cell_light(Vector3::Y() / cells.size()), (-1.0_f32).exp());
        assert_eq!(cells.get_cell_light(Vector3::ZERO() / cells.size()), (-1.0_f32).exp());
        assert_eq!(cells.get_cell_light(Vector3::Y() * 2.0 / cells.size()), 1.0);
    }
}

// this is just here while there is a vscode bug that doesn't save terminal history
// cargo test light_cell -- --nocapture