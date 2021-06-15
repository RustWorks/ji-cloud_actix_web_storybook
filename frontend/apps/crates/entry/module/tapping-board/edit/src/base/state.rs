use components::module::edit::prelude::*;
use components::audio_mixer::AudioMixer;
use std::rc::Rc;
use shared::domain::jig::{
    JigId, 
    Jig,
    module::{
        ModuleId, 
        body::{
            StepExt,
            ThemeChoice,
            Trace as RawTrace,
            Backgrounds as RawBackgrounds, 
            Audio,
            Instructions,
            tapping_board::{
                Step,
                PlaySettings as RawPlaySettings, 
                Hint, Next,
                Mode, 
                Content as RawContent, 
                ModuleData as RawData
            }
        }
    }
};
use futures_signals::{
    map_ref,
    signal::{self, Signal, SignalExt, ReadOnlyMutable, Mutable},
    signal_vec::MutableVec
};
use utils::prelude::*;
use components::{
    text_editor::{
        state::State as TextEditorState,
        callbacks::Callbacks as TextEditorCallbacks
    },
    stickers::{
        state::Stickers,
        callbacks::Callbacks as StickersCallbacks
    },
    backgrounds::{
        state::Backgrounds,
        callbacks::Callbacks as BackgroundsCallbacks,
    },
    traces::{
        bubble::state::TraceBubble,
        edit::{
            state::Edit as TracesEdit, 
            callbacks::Callbacks as TracesCallbacks
        }
    },
    tooltip::state::State as TooltipState
};
use dominator::clone;
use std::cell::RefCell;
pub struct Base {
    pub history: Rc<HistoryStateImpl<RawData>>,
    pub step: ReadOnlyMutable<Step>,
    pub theme: Mutable<ThemeChoice>,
    pub instructions: Mutable<Instructions>,
    pub jig_id: JigId,
    pub module_id: ModuleId,
    // TappingBoard-specific
    pub jig_theme_id: ReadOnlyMutable<ThemeId>,
    pub backgrounds: Rc<Backgrounds>, 
    pub stickers: Rc<Stickers>, 
    pub traces: Rc<TracesEdit>,
    pub traces_meta: MutableVec<TraceMeta>,
    pub text_editor: Rc<TextEditorState>,
    pub audio_mixer: AudioMixer,
    pub play_settings: Rc<PlaySettings>,
}

pub struct PlaySettings {
    pub hint: Mutable<Hint>,
    pub next: Mutable<Next>
}

impl PlaySettings {
    pub fn new(settings:RawPlaySettings) -> Self {

        Self {
            hint: Mutable::new(settings.hint),
            next: Mutable::new(settings.next),
        }
    }

    pub fn to_raw(&self) -> RawPlaySettings {
        RawPlaySettings {
            hint: self.hint.get_cloned(),
            next: self.next.get_cloned(),
        }
    }
}

#[derive(Clone)]
pub struct TraceMeta {
    pub audio: Mutable<Option<Audio>>,
    pub text: Mutable<Option<String>>,
    pub bubble: Mutable<Option<Rc<TraceBubble>>>,
}

impl TraceMeta {
    pub fn new(audio: Option<Audio>, text: Option<String>) -> Self {
        Self {
            audio: Mutable::new(audio),
            text: Mutable::new(text),
            bubble: Mutable::new(None)
        }
    }
}

impl Base {
    pub async fn new(init_args: BaseInitFromRawArgs<RawData, Mode, Step>) -> Rc<Self> {

        let BaseInitFromRawArgs { 
            raw,
            jig_id,
            module_id,
            jig_theme_id,
            history,
            step,
            theme,
            audio_mixer,
            ..
        } = init_args;

        let content = raw.content.unwrap_ji();

        let _self_ref:Rc<RefCell<Option<Rc<Self>>>> = Rc::new(RefCell::new(None));

        let instructions = Mutable::new(content.instructions);
      
        let stickers_ref:Rc<RefCell<Option<Rc<Stickers>>>> = Rc::new(RefCell::new(None));

        let text_editor = TextEditorState::new(

            match content.theme {
                ThemeChoice::Jig => {
                    // self.jig.as_ref().unwrap_ji().theme_id.clone()
                    log::warn!("waiting on jig settings");
                    ThemeId::Chalkboard
                },
                ThemeChoice::Override(theme_id) => theme_id
            },
            None, 
            TextEditorCallbacks::new(
                //New text
                Some(clone!(stickers_ref => move |value:&str| {
                    if let Some(stickers) = stickers_ref.borrow().as_ref() {
                        Stickers::add_text(stickers.clone(), value.to_string());
                    }
                })),
                //Text change
                Some(clone!(stickers_ref => move |value:&str| {
                    if let Some(stickers) = stickers_ref.borrow().as_ref() {
                        stickers.set_current_text_value(value.to_string());
                    }
                })),
                //Blur
                Some(clone!(stickers_ref => move || {
                    if let Some(stickers) = stickers_ref.borrow().as_ref() {
                        stickers.stop_current_text_editing();
                    }
                }))
        ));


        let backgrounds = Rc::new(Backgrounds::from_raw(
                &content.backgrounds,
                BackgroundsCallbacks::new(
                    Some(clone!(history => move |raw_bgs| {
                        history.push_modify(|raw| {
                            if let Some(content) = &mut raw.content {
                                content.backgrounds = raw_bgs;
                            }
                        });
                    }))
                )
        ));

        let stickers = Stickers::from_raw(
                &content.stickers,
                text_editor.clone(),
                StickersCallbacks::new(
                    Some(clone!(history => move |raw_stickers| {
                        history.push_modify(|raw| {
                            if let Some(content) = &mut raw.content {
                                content.stickers = raw_stickers;
                            }
                        });
                    }))
                )
        );

        *stickers_ref.borrow_mut() = Some(stickers.clone());


        let traces = TracesEdit::from_raw(

            &content.traces
                .iter()
                .map(|trace_meta| {
                    trace_meta.trace.clone()
                })
                .collect::<Vec<RawTrace>>(),
            crate::debug::settings().trace_opts.clone(),
            TracesCallbacks::new(
                Some(clone!(_self_ref => move |raw_trace| {
                    if let Some(_self) = _self_ref.borrow().as_ref() {
                        _self.on_trace_added(raw_trace);
                    }
                })),
                Some(clone!(_self_ref => move |index| {
                    if let Some(_self) = _self_ref.borrow().as_ref() {
                        _self.on_trace_deleted(index);
                    }
                })),
                Some(clone!(_self_ref => move |index, raw_trace| {
                    if let Some(_self) = _self_ref.borrow().as_ref() {
                        _self.on_trace_changed(index, raw_trace);
                    }
                })),
            )
        );

        let traces_meta = MutableVec::new_with_values(
            content.traces
                .iter()
                .map(|trace_meta| {
                    TraceMeta::new(
                        trace_meta.audio.clone(), 
                        trace_meta.text.clone()
                    )
                })
                .collect()
        );

        let _self = Rc::new(Self {
            jig_id,
            module_id,
            jig_theme_id: jig_theme_id.read_only(),
            history,
            step: step.read_only(),
            theme,
            instructions,
            text_editor,
            backgrounds,
            stickers,
            traces,
            traces_meta,
            audio_mixer,
            play_settings: Rc::new(PlaySettings::new(content.play_settings.clone())),
        });

        *_self_ref.borrow_mut() = Some(_self.clone());

        _self
    }
}


impl BaseExt<Step> for Base {
    type NextStepAllowedSignal = impl Signal<Item = bool>;
    type ThemeIdSignal = impl Signal<Item = ThemeId>;
    type ThemeIdStrSignal = impl Signal<Item = &'static str>;

    fn allowed_step_change(&self, from:Step, to:Step) -> bool {
        true
    }

    fn next_step_allowed_signal(&self) -> Self::NextStepAllowedSignal {
        signal::always(true)
    }

    fn get_theme_id(&self) -> ThemeId {
        match self.theme.get_cloned() {
            ThemeChoice::Jig => self.jig_theme_id.get(),
            ThemeChoice::Override(theme_id) => theme_id
        }
    }
    fn theme_id_signal(&self) -> Self::ThemeIdSignal { 
        map_ref! {
            let jig_theme_id = self.jig_theme_id.signal(),
            let theme = self.theme.signal()
                => {
                match *theme { 
                    ThemeChoice::Jig => *jig_theme_id,
                    ThemeChoice::Override(theme_id) => theme_id
                }
            }
        }
    }

    fn theme_id_str_signal(&self) -> Self::ThemeIdStrSignal { 
        self.theme_id_signal().map(|id| id.as_str_id())
    }
}
