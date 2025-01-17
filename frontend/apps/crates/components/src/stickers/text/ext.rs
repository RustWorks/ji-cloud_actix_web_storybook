use shared::domain::jig::module::body::{Transform, _groups::design::Text};
use utils::prelude::*;
pub trait TextExt {
    fn new(value: String) -> Self;
}

impl TextExt for Text {
    /// Create a new Text
    fn new(value: String) -> Self {
        Self {
            value,
            transform: Transform::identity(),
        }
    }
}
