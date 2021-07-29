use std::rc::Rc;
use dominator::{html, clone, Dom};
use utils::prelude::*;
use futures_signals::signal::{Signal, SignalExt};
use components::{
    image::search::dom::render as render_image_search,
    text_editor::dom::render_controls as render_text_editor,
    audio_input::{state::State as AudioInputState, dom::render as render_audio_input},
};

pub fn render_step_4() -> Dom {
    html!("module-sidebar-body", {
        .property("slot", "body")
        .child(
            html!("module-sidebar-drag-prompt")
        )
    })
}
