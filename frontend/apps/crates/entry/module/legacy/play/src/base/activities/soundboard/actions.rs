use std::{borrow::BorrowMut, rc::Rc};
use super::state::*;
use shared::domain::jig::module::body::legacy::activity::AdvanceTrigger;
use utils::prelude::*;
use dominator::{Dom, html, clone};

impl Soundboard {
    pub fn on_start(self: Rc<Self>) {

        let state = self;

        if let Some(audio_filename) = state.raw.audio_filename.as_ref() {
            state.base.audio_manager.play_clip_on_ended(
                state.base.activity_media_url(&audio_filename),
                clone!(state => move || {
                    state.on_intro_finished();
                })
            );
        } else {
            state.on_intro_finished();
        }

    }

    pub fn on_intro_finished(&self) {
        if let Some(bg_audio_filename) = self.raw.bg_audio_filename.as_ref() {
            self.base.audio_manager.play_bg(self.base.activity_media_url(&bg_audio_filename));
        }

        self.phase.set_neq(if self.raw.show_hints { Phase::Hints } else { Phase::Playing });
    }

    pub fn on_hints_finished(self: Rc<Self>) {
        self.phase.set_neq(Phase::Playing);
    }
}


impl SoundboardItem {
    pub fn on_click(self: Rc<Self>, parent: Rc<Soundboard>) {
        let state = self;

        let was_revealed = state.revealed.replace(true);
        if !was_revealed {
            log::info!("first time!");
        }

        state.hotspot.tooltip_text.set(state.text.clone());


        if let Some(audio_filename) = state.audio_filename.as_ref() {
            state.base.audio_manager.play_clip_on_ended(
                state.base.activity_media_url(&audio_filename),
                clone!(state => move || {
                    if let Some(index) = state.jump_index {
                        log::info!("jumpin to {}", index);
                        let _ = IframeAction::new(ModuleToJigPlayerMessage::JumpToIndex(index)).try_post_message_to_top();
                    } else {
                        let all_revealed = parent.items.iter().all(|item| item.revealed.get());

                        if all_revealed {
                            log::info!("finished all, going next");
                            let _ = IframeAction::new(ModuleToJigPlayerMessage::Next).try_post_message_to_top();
                        }
                    }
                })
            );
        }

        
    }
}