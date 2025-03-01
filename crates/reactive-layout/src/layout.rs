pub mod flex;
pub mod grid;
pub mod display;
mod macros;

use display::Display;

#[derive(Debug)]
pub struct Layout {
    display: Display,
}

impl taffy::CoreStyle for Layout {}

impl taffy::FlexboxContainerStyle for Layout {

}

impl taffy::FlexboxItemStyle for Layout {
    
}
