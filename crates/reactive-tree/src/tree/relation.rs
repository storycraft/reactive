use crate::ElementId;

#[derive(Debug)]
pub struct Relation {
    pub parent: Option<ElementId>,
    pub children: Vec<ElementId>,
}
