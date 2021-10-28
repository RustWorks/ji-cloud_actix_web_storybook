use super::state::Controller;
use std::{
    sync::atomic::{Ordering}
};
use components::audio::mixer::{AUDIO_MIXER, AudioSource};
use gloo_timers::callback::Timeout;
use utils::{
    prelude::*,
    math::bounds::BoundsF64
};
use shared::domain::jig::module::body::legacy::design::{Animation, HideToggle, Sticker as RawSticker};
use crate::base::actions::StageClick;

impl Controller {
    pub fn handle_click(&self, stage_click: StageClick) {

        let is_target = {
            match self.elem.borrow().as_ref() {
                None => false,
                Some(elem) => {
                    let bounds:BoundsF64 = elem.into();
                    bounds.contains_point(stage_click.mouse_x, stage_click.mouse_y)
                }
            }
        };

        if !is_target || !self.interactive {
            return;
        }
        
        let has_toggled_once = self.has_toggled_once.load(Ordering::SeqCst);

        if let Some(hide_toggle) = self.hide_toggle {
            if !has_toggled_once || hide_toggle == HideToggle::Always {
                let val = self.hidden.get();
                self.hidden.set(!val);
            }
        }

        self.has_toggled_once.store(true, Ordering::SeqCst);

        let (playing_anim, playing_audio) = if self.hidden.get() { 
            (false, false)
        } else { 
            let play_toggle = !self.playing.load(Ordering::SeqCst);

            if self.anim.tap {
                (play_toggle, play_toggle) 
            } else if self.anim.once && self.has_finished_once.load(Ordering::SeqCst) {
                (false, play_toggle)
            } else {
                (true, true)
            }
        };

        self.playing.store(playing_anim, Ordering::SeqCst);

        if playing_anim {
            // this is a small departure from TT, reset to the beginning in case 
            // the sound was a bit timed to the animation
            if self.anim.tap {
                self.curr_frame_index.store(0, Ordering::SeqCst);
            }
        }

        if playing_audio {
            if let Some(audio_filename) = self.audio_filename.as_ref() {
                let url = self.base.design_media_url(&audio_filename);
                //win the race condition with hotspots
                Timeout::new(0, move || { 
                    AUDIO_MIXER.with(|mixer| {
                        mixer.pause_all();
                        mixer.play_oneshot(AudioSource::Url(url))
                    });
                })
                .forget();
            }
        }

    }
}