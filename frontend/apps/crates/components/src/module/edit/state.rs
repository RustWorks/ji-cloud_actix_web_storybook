#![feature(type_alias_impl_trait)]
#![feature(min_type_alias_impl_trait)]
use futures_signals::{
    map_ref,
    signal::{self, Mutable, ReadOnlyMutable,  SignalExt, Signal},
    signal_vec::{MutableVec, SignalVecExt, SignalVec},
    CancelableFutureHandle, 
};
use dominator::{DomBuilder, Dom, html, events, clone, apply_methods, with_node};
use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::convert::{TryFrom, TryInto};
use std::future::Future;
use itertools::Itertools;
use std::fmt::Write;
use serde::{Serialize, de::DeserializeOwned};
use crate::module::page::ModulePageKind;
use crate::module::history::state::HistoryState;
use dominator_helpers::{
    signals::OptionSignal,
    futures::AsyncLoader,
};
use wasm_bindgen_futures::spawn_local;
//use super::actions::{HistoryChangeFn, HistoryUndoRedoFn};
use shared::domain::jig::{JigId, module::{ModuleId, body::BodyExt}};
use super::{
    actions::*,
    steps::state::*,
    choose::state::*,
};
use shared::{
    api::endpoints::{ApiEndpoint, self, jig::module::*},
    error::{EmptyError, MetadataNotFound},
    domain::jig::{*, module::{*, body::Body}},
};
use utils::{settings::SETTINGS, prelude::*};

pub struct GenericState <Mode, Step, RawData, Base, Main, Sidebar, Header, Footer, Overlay> 
where
    RawData: BodyExt + 'static,
    Mode: ModeExt + 'static,
    Step: StepExt + 'static,
    Base: BaseExt<Step> + 'static,
    Main: MainExt + 'static,
    Sidebar: SidebarExt + 'static,
    Header: HeaderExt + 'static,
    Footer: FooterExt + 'static,
    Overlay: OverlayExt + 'static,
{
    pub phase: Mutable<Rc<Phase<Mode, Step, Base, Main, Sidebar, Header, Footer, Overlay>>>,
    pub(super) jig: RefCell<Option<Jig>>,
    pub(super) opts: StateOpts<RawData>,
    pub(super) is_preview: Mutable<bool>,
    pub(super) raw_loader: AsyncLoader,
    pub(super) save_loader: Rc<AsyncLoader>,
    pub(super) history: RefCell<Option<Rc<HistoryStateImpl<RawData>>>>,
    pub(super) raw_loaded: Mutable<bool>,
    pub(super) page_body_switcher: AsyncLoader,
    pub(super) reset_from_history_loader: AsyncLoader,
}

pub enum Phase <Mode, Step, Base, Main, Sidebar, Header, Footer, Overlay> 
where
    Mode: ModeExt + 'static,
    Step: StepExt + 'static,
    Base: BaseExt<Step> + 'static,
    Main: MainExt + 'static,
    Sidebar: SidebarExt + 'static,
    Header: HeaderExt + 'static,
    Footer: FooterExt + 'static,
    Overlay: OverlayExt + 'static,
{
    Init,
    Choose(Rc<Choose<Mode>>),
    Steps(Rc<Steps<Step, Base, Main, Sidebar, Header, Footer, Overlay>>),
}

#[derive(Debug, Clone)]
pub struct StateOpts<RawData> {
    pub skip_save_for_debug: bool,
    pub skip_load_jig: bool,
    pub jig_id: JigId,
    pub module_id: ModuleId,
    //the step which is for previewing
    pub is_main_scrollable: bool,
    pub force_raw: Option<RawData>, 
}

impl <RawData> StateOpts<RawData> {
    pub fn new(jig_id: JigId, module_id: ModuleId) -> Self {
        Self {
            skip_save_for_debug: false,
            skip_load_jig: false,
            jig_id,
            module_id,
            is_main_scrollable: true,
            force_raw: None,
        }
    }
}


/*
 * Note: the idea is to create the top-level state
 * and then pass it down here
 */

pub type IsHistory = bool;

impl <Mode, Step, RawData, Base, Main, Sidebar, Header, Footer, Overlay> GenericState <Mode, Step, RawData, Base, Main, Sidebar, Header, Footer, Overlay> 
where
    Mode: ModeExt + 'static,
    Step: StepExt + 'static,
    Base: BaseExt<Step> + 'static,
    Main: MainExt + 'static,
    Sidebar: SidebarExt + 'static,
    Header: HeaderExt + 'static,
    Footer: FooterExt + 'static,
    Overlay: OverlayExt + 'static,
    RawData: BodyExt + 'static, 
{
    pub fn new<InitFromModeFn, InitFromModeOutput, InitFromRawFn, InitFromRawOutput>(
        opts: StateOpts<RawData>, 
        init_from_mode: InitFromModeFn,
        init_from_raw: InitFromRawFn, 
    ) -> Rc<Self>
    where
        InitFromModeFn: Fn(JigId, ModuleId, Option<Jig>, Mode, Rc<HistoryStateImpl<RawData>>) -> InitFromModeOutput + Clone + 'static,
        InitFromModeOutput: Future<Output = StepsInit<Step, Base, Main, Sidebar, Header, Footer, Overlay>>,
        InitFromRawFn: Fn(JigId, ModuleId, Option<Jig>, RawData, IsHistory, Option<Rc<Steps<Step, Base, Main, Sidebar, Header, Footer, Overlay>>>, Rc<HistoryStateImpl<RawData>>) -> InitFromRawOutput + Clone + 'static,
        InitFromRawOutput: Future<Output = Option<StepsInit<Step, Base, Main, Sidebar, Header, Footer, Overlay>>>,
        <RawData as TryFrom<ModuleBody>>::Error: std::fmt::Debug
    {


        let _self = Rc::new(Self {
            opts,
            jig: RefCell::new(None),
            phase: Mutable::new(Rc::new(Phase::Init)), 
            is_preview: Mutable::new(false),
            history: RefCell::new(None),
            raw_loaded: Mutable::new(false),
            raw_loader: AsyncLoader::new(),
            save_loader: Rc::new(AsyncLoader::new()),
            page_body_switcher: AsyncLoader::new(),
            reset_from_history_loader: AsyncLoader::new(),
        });


        _self.raw_loader.load(clone!(_self => async move {
            if !_self.opts.skip_load_jig {
                *_self.jig.borrow_mut() = {

                        let path = endpoints::jig::Get::PATH.replace("{id}",&_self.opts.jig_id.0.to_string());

                        match api_with_auth::<JigResponse, EmptyError, ()>(&path, endpoints::jig::Get::METHOD, None).await {
                            Ok(resp) => {
                                Some(resp.jig)
                            },
                            Err(_) => {
                                panic!("error loading jig!")
                            },
                        }
                };
            }

            let raw = {
                if let Some(force_raw) = _self.opts.force_raw.clone() {
                    force_raw
                } else {
                    let path = Get::PATH
                        .replace("{id}",&_self.opts.jig_id.0.to_string())
                        .replace("{module_id}",&_self.opts.module_id.0.to_string());

                    match api_with_auth::<ModuleResponse, EmptyError, ()>(&path, Get::METHOD, None).await {
                        Ok(resp) => {
                            let body = resp.module.body.unwrap_ji();
                            body.try_into().unwrap_ji()
                        },
                        Err(_) => {
                            panic!("error loading module!")
                        }
                    }
                }
            };

            let history = Rc::new(HistoryState::new(
                raw.clone(),
                super::actions::save_history(
                    _self.opts.skip_save_for_debug,
                    _self.save_loader.clone(),
                    _self.opts.jig_id.clone(),
                    _self.opts.module_id.clone(),
                ),
                Self::reset_from_history(_self.clone(), init_from_raw.clone(), init_from_mode.clone())
            ));

            *_self.history.borrow_mut() = Some(history.clone());

            let (jig_id, module_id, jig) = (
                _self.opts.jig_id.clone(),
                _self.opts.module_id.clone(),
                _self.jig.borrow().clone()
            );

            if let Some(base) = init_from_raw(jig_id, module_id, jig, raw, false, None, history.clone()).await {
                Self::change_phase_steps(_self.clone(), base);
            } else {
                Self::change_phase_choose(_self.clone(), init_from_mode);
            }

            _self.raw_loaded.set_neq(true);
        }));


        _self
    }

}
