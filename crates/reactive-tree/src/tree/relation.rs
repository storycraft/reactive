use crate::ElementId;

#[derive(Debug)]
pub struct Relation {
    pub parent: ElementId,
    pub children: Vec<ElementId>,
}
