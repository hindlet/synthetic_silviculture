#![allow(dead_code, unused_variables, unused_imports)]
use std::collections::HashMap;
use crate::maths::vector_three_int::Vector3Int;
use bevy_ecs::prelude::*;


#[derive(Resource, Debug)]
pub struct LightCells {
    // cells hash map, set out as Hashmap<id, (own_shadow, upwards_shadow)>
    cells: HashMap<Vector3Int, (f32, f32)>,
    // the maximum height up or down that the code will check for cells if one isn't found
    check_height: i32,
}


impl LightCells {

    pub fn new(check_height: i32) -> Self{
        LightCells {
            cells: HashMap::new(),
            check_height
        }
    }

    /// adds a new cell to the grid, updating it's parent shadow and child shadow
    fn add_cell(&mut self, id: Vector3Int, shadow: f32) -> f32 {
        let parent_shadow = {
            let parent = self.cells.get(&(id + Vector3Int::Y()));
            if parent.is_some() {parent.unwrap().0 + parent.unwrap().1}
            else {0.0}
        };
        self.cells.insert(id, (shadow, parent_shadow));
        self.propogate_down(id);
        self.get_cell_shadow(id)
    }

    /// sets the shadow of a given cell and propogates it downwards, adds the cell if it does not exist
    pub fn update_cell_shadow(&mut self, id: impl Into<Vector3Int>, new_shadow: f32) {
        let id: Vector3Int = id.into();

        match self.cells.get_mut(&id) {
            Some(shade_data) => {shade_data.0 = new_shadow; self.propogate_down(id);},
            None => {self.add_cell(id, new_shadow);}
        }

    }

    /// returns the shadow value of a given cell, if it does not exist but a cell above within the check range exists, it's shadow value is taken
    pub fn get_cell_shadow(&self, id: impl Into<Vector3Int>) -> f32 {
        let id: Vector3Int = id.into();

        match self.cells.get(&id) {
            Some(shade_data) => return shade_data.0 + shade_data.1,
            None => ()
        }

        let mut count = 1;
            
        loop {
            if count > self.check_height {break 0.0;}

            let next_id = id + Vector3Int::Y() * count;

            match self.cells.get(&next_id) {
                Some(shade_data) => {break shade_data.0 + shade_data.1;},
                None => ()
            }

            count += 1;
        }
    }

    /// propogates shadow down the cells from a parent, does nothing if the start id does not exist
    fn propogate_down(&mut self, start_id: Vector3Int) {
        

        let mut current_shadow = {
            match self.cells.get(&start_id) {
                Some(shade_data) => shade_data.0 + shade_data.1,
                None => return,
            }
        };

        let mut count = 1;
        let mut dist = 1;
        loop {
            if count > self.check_height {break;}

            let next_id = start_id - Vector3Int::Y() * dist;

            match self.cells.get_mut(&next_id) {
                Some(shade_data) => {shade_data.1 = current_shadow; current_shadow += shade_data.0; count = 0;},
                None => count += 1
            }

            dist += 1;
        }
    }
}



#[cfg(test)]
mod light_cells_tests{
    use super::{LightCells};

    #[test]
    fn unknown_cell_test() {
        let cells = LightCells::new(0);

        assert_eq!(cells.get_cell_shadow([0, 0, 0]), 0.0);
    }

    #[test]
    fn above_cell_test() {
        let mut cells = LightCells::new(2);

        cells.update_cell_shadow([0, 2, 0], 2.0);

        assert_eq!(cells.get_cell_shadow([0, 0, 0]), 2.0);
    }

    #[test]
    fn known_cell_test() {
        let mut cells = LightCells::new(0);

        cells.update_cell_shadow([0, 0, 0], 2.0);

        assert_eq!(cells.get_cell_shadow([0, 0, 0]), 2.0);
    }

    #[test]
    fn propogation_test() {
        let mut cells = LightCells::new(2);

        cells.update_cell_shadow([0, 0, 0], 0.0);

        cells.update_cell_shadow([0, 2, 0], 2.0);

        assert_eq!(cells.get_cell_shadow([0, 0, 0]), 2.0);
    }

    #[test]
    fn non_propogation_test() {
        let mut cells = LightCells::new(1);

        cells.update_cell_shadow([0, 0, 0], 0.0);

        cells.update_cell_shadow([0, 2, 0], 2.0);

        assert_eq!(cells.get_cell_shadow([0, 0, 0]), 0.0);
    }

    #[test]
    fn multiple_propogation_test() {
        let mut cells = LightCells::new(3);

        cells.update_cell_shadow([0, 0, 0], 1.0);

        cells.update_cell_shadow([0, 2, 0], 2.0);

        cells.update_cell_shadow([0, 4, 0], 3.0);

        cells.update_cell_shadow([0, 7, 0], 1.5);

        assert_eq!(cells.get_cell_shadow([0, 4, 0]), 4.5);
        assert_eq!(cells.get_cell_shadow([0, 2, 0]), 6.5);
        assert_eq!(cells.get_cell_shadow([0, 0, 0]), 7.5);
    }
}

// this is just here while there is a vscode bug that doesn't save terminal history
// cargo test light_cell -- --nocapture