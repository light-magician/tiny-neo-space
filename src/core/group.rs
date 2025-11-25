use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Group {
    pub id: u32,
    pub name: String,
    pub cells: HashSet<(i32, i32)>,
}
