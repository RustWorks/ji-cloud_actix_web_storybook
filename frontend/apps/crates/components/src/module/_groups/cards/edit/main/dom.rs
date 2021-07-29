use dominator::{html, Dom, clone};
use std::rc::Rc;
use super::state::*;
use futures_signals::{
    signal_vec::SignalVecExt,
    signal::SignalExt
};
use super::pair::{
    state::MainPair,
    dom::render as render_pair,
};
use crate::{
    module::{
        _groups::cards::edit::state::*,
        _common::edit::prelude::*,
    },
    backgrounds::dom::render_single_background
};
use shared::domain::jig::module::body::_groups::cards::Step;

impl <RawData, E, GetSettingsStateFn, RenderSettingsStateFn, SettingsState> DomRenderable for Main<RawData, E, GetSettingsStateFn, RenderSettingsStateFn, SettingsState> 
where
    RawData: RawDataExt, 
    E: ExtraExt,
    GetSettingsStateFn: Fn(Rc<CardsBase<RawData, E>>) -> SettingsState + Clone + 'static,
    RenderSettingsStateFn: Fn(Rc<SettingsState>) -> Dom + Clone + 'static,
    SettingsState: 'static,
{
    fn render(state: Rc<Main<RawData, E, GetSettingsStateFn, RenderSettingsStateFn, SettingsState>>) -> Dom {
        html!("empty-fragment", {
            .child_signal(state.base.is_empty_signal().map(clone!(state => move |is_empty| {
                Some(
                    if is_empty {
                        html!("main-empty")
                    } else {
                        html!("empty-fragment", {
                            .child_signal(state.base.step.signal_cloned().map(clone!(state => move |step| {
                                Some(match step {
                                    Step::Three => {
                                        (state.render_settings) (Rc::new((state.get_settings) (state.base.clone())))
                                    },
                                    _ => {
                                        render_main_cards(state.base.clone(), step.clone())
                                    }
                                })
                            })))
                        })
                    }
                )
            })))
        })
    }
}
impl <RawData, E, GetSettingsStateFn, RenderSettingsStateFn, SettingsState> MainDomRenderable for Main<RawData, E, GetSettingsStateFn, RenderSettingsStateFn, SettingsState> 
where
    RawData: RawDataExt, 
    E: ExtraExt,
    GetSettingsStateFn: Fn(Rc<CardsBase<RawData, E>>) -> SettingsState + Clone + 'static,
    RenderSettingsStateFn: Fn(Rc<SettingsState>) -> Dom + Clone + 'static,
    SettingsState: 'static,
{
    fn render_bg(state: Rc<Main<RawData, E, GetSettingsStateFn, RenderSettingsStateFn, SettingsState>>) -> Option<Dom> {
        Some(render_single_background(state.base.background.signal_cloned(), state.base.theme_id.signal_cloned(), None))
    }
}

pub fn render_main_cards<RawData: RawDataExt, E: ExtraExt>(base: Rc<CardsBase<RawData, E>>, step: Step) -> Dom {
    html!("main-cards", {
        .children_signal_vec({
            base.pairs
                .signal_vec_cloned()
                .enumerate()
                .map(clone!(base => move |(index, pair)| {
                    let pair = MainPair::new(
                        base.clone(),
                        step.clone(),
                        index.clone(),
                        pair
                    );
                    render_pair(pair)
                }))
        })
    })
}
