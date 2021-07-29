use components::module::_common::edit::prelude::*;
use crate::base::state::Base;
use std::rc::Rc;

pub struct Main {
    pub base: Rc<Base>,
}

impl Main {
    pub fn new(base: Rc<Base>) -> Self {
        Self {
            base,
        }
    }

}

impl MainExt for Main {
}


