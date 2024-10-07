use std::collections::HashMap;

use crate::branch::Branch;




pub struct Plant<'branch_lifetime, 'node_lifetime> {
    pub branches: HashMap<usize, Branch<'branch_lifetime, 'node_lifetime>>
}

impl<'branch_lifetime, 'node_lifetime> Plant<'branch_lifetime, 'node_lifetime> {

}