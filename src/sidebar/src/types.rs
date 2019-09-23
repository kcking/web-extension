use crate::constants::{DRAG_ANIMATION_DURATION, INSERT_ANIMATION_DURATION};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use tab_organizer::{local_storage_get, state};
use tab_organizer::state::Options;
use crate::url_bar;
use crate::search;
use crate::menu::Menu;
use crate::groups::Groups;
use uuid::Uuid;
use web_sys::DomRect;
use futures_signals::signal::{Signal, Mutable};
use futures_signals::signal_vec::MutableVec;
use dominator::animation::{MutableAnimation, Percentage, OnTimestampDiff};
use lazy_static::lazy_static;


#[derive(Debug)]
pub(crate) enum DragState {
    DragStart {
        mouse_x: i32,
        mouse_y: i32,
        rect: DomRect,
        group: Arc<Group>,
        tab: Arc<Tab>,
        tab_index: usize,
    },

    // TODO maybe this should be usize rather than Option<usize>
    Dragging {
        mouse_x: i32,
        mouse_y: i32,
        rect: DomRect,
        group: Arc<Group>,
        tab_index: Option<usize>,
    },
}


#[derive(Debug)]
pub(crate) struct Dragging {
    pub(crate) state: Mutable<Option<DragState>>,
    pub(crate) selected_tabs: Mutable<Vec<Arc<Tab>>>,
}

impl Dragging {
    fn new() -> Self {
        Self {
            state: Mutable::new(None),
            selected_tabs: Mutable::new(vec![]),
        }
    }
}


#[derive(Debug)]
pub(crate) struct Scrolling {
    pub(crate) on_timestamp_diff: Mutable<Option<OnTimestampDiff>>,
    pub(crate) y: Mutable<f64>,
    pub(crate) height: Mutable<f64>,
}

impl Scrolling {
    fn new(scroll_y: f64) -> Self {
        Self {
            on_timestamp_diff: Mutable::new(None),
            y: Mutable::new(scroll_y),
            height: Mutable::new(0.0),
        }
    }
}


#[derive(Debug)]
pub(crate) struct State {
    pub(crate) search_box: Mutable<Arc<String>>,
    pub(crate) search_parser: Mutable<Arc<search::Parsed>>,

    pub(crate) url_bar: Mutable<Option<Arc<url_bar::UrlBar>>>,
    pub(crate) groups_padding: Mutable<f64>, // TODO use u32 instead ?

    pub(crate) groups: Groups,
    pub(crate) window: RwLock<Window>,
    pub(crate) options: Options,

    pub(crate) dragging: Dragging,
    pub(crate) scrolling: Scrolling,

    pub(crate) menu: Menu,
}

impl State {
    pub(crate) fn new(options: Options, window: Window) -> Self {
        let search_value = local_storage_get("tab-organizer.search").unwrap_or_else(|| "".to_string());
        let scroll_y = local_storage_get("tab-organizer.scroll.y").map(|value| value.parse().unwrap()).unwrap_or(0.0);

        let state = Self {
            search_parser: Mutable::new(Arc::new(search::Parsed::new(&search_value))),
            search_box: Mutable::new(Arc::new(search_value)),

            url_bar: Mutable::new(None),
            groups_padding: Mutable::new(0.0),

            groups: Groups::new(options.sort_tabs.get()),
            window: RwLock::new(window),
            options,

            dragging: Dragging::new(),
            scrolling: Scrolling::new(scroll_y),

            menu: Menu::new(),
        };

        state.groups.initialize(&state);

        state
    }

    /*fn is_dragging_group(&self, group_id: usize) -> impl Signal<Item = bool> {
        self.dragging.state.signal_ref(move |dragging| {
            if let Some(DragState::Dragging { group, .. }) = dragging {
                group.id == group_id

            } else {
                false
            }
        })
    }*/
}


#[derive(Debug)]
pub(crate) struct TabState {
    pub(crate) id: Uuid,
    pub(crate) favicon_url: Mutable<Option<Arc<String>>>,
    pub(crate) title: Mutable<Option<Arc<String>>>,
    pub(crate) url: Mutable<Option<Arc<String>>>,
    pub(crate) index: Mutable<usize>,
    pub(crate) focused: Mutable<bool>,
    pub(crate) unloaded: Mutable<bool>,
    pub(crate) pinned: Mutable<bool>,
    pub(crate) removed: Mutable<bool>,
    pub(crate) timestamp_created: Mutable<f64>,
    pub(crate) timestamp_focused: Mutable<f64>,
    pub(crate) tags: Mutable<Vec<state::Tag>>,
}

impl TabState {
    pub(crate) fn new(state: state::Tab, index: usize) -> Self {
        Self {
            id: state.serialized.id,
            favicon_url: Mutable::new(state.favicon_url.map(Arc::new)),
            title: Mutable::new(state.title.map(Arc::new)),
            url: Mutable::new(state.url.map(Arc::new)),
            index: Mutable::new(index),
            focused: Mutable::new(state.focused),
            unloaded: Mutable::new(state.unloaded),
            pinned: Mutable::new(state.pinned),
            removed: Mutable::new(false),
            timestamp_created: Mutable::new(state.serialized.timestamp_created),
            timestamp_focused: Mutable::new(state.serialized.timestamp_focused),
            tags: Mutable::new(state.serialized.tags),
        }
    }
}


#[derive(Debug)]
pub(crate) struct Tab {
    pub(crate) state: Arc<TabState>,

    pub(crate) selected: Mutable<bool>,
    pub(crate) dragging: Mutable<bool>,

    pub(crate) hovered: Mutable<bool>,
    pub(crate) holding: Mutable<bool>,

    pub(crate) close_hovered: Mutable<bool>,
    pub(crate) close_holding: Mutable<bool>,

    pub(crate) matches_search: Mutable<bool>,
    pub(crate) visible: Mutable<bool>,

    pub(crate) drag_over: MutableAnimation,
    pub(crate) insert_animation: MutableAnimation,
}

impl Tab {
    pub(crate) fn new(state: Arc<TabState>) -> Self {
        Self {
            state,

            selected: Mutable::new(false),
            dragging: Mutable::new(false),

            hovered: Mutable::new(false),
            holding: Mutable::new(false),

            close_hovered: Mutable::new(false),
            close_holding: Mutable::new(false),

            matches_search: Mutable::new(false),
            visible: Mutable::new(false),

            drag_over: MutableAnimation::new(DRAG_ANIMATION_DURATION),
            insert_animation: MutableAnimation::new_with_initial(INSERT_ANIMATION_DURATION, Percentage::new(0.0)),
        }
    }

    pub(crate) fn is_focused(&self) -> impl Signal<Item = bool> {
        self.focused.signal()
    }
}

impl Deref for Tab {
    type Target = TabState;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.state
    }
}


#[derive(Debug)]
pub(crate) struct Window {
    pub(crate) id: Uuid,
    pub(crate) name: Mutable<Option<Arc<String>>>,
    pub(crate) tabs: Vec<Arc<TabState>>,
}

impl Window {
    pub(crate) fn new(state: state::Window) -> Self {
        Self {
            id: state.serialized.id,
            name: Mutable::new(state.serialized.name.map(Arc::new)),
            tabs: state.tabs.into_iter().enumerate().map(|(index, tab)| Arc::new(TabState::new(tab, index))).collect(),
        }
    }
}


#[derive(Debug)]
pub(crate) struct Group {
    pub(crate) id: usize,
    pub(crate) show_header: bool,
    pub(crate) timestamp: f64,
    pub(crate) name: Mutable<Option<Arc<String>>>,
    pub(crate) tabs: MutableVec<Arc<Tab>>,

    pub(crate) insert_animation: MutableAnimation,
    pub(crate) visible: Mutable<bool>,

    pub(crate) matches_search: Mutable<bool>,
    pub(crate) last_selected_tab: Mutable<Option<Uuid>>,

    pub(crate) drag_over: MutableAnimation,
    pub(crate) drag_top: MutableAnimation,
    pub(crate) tabs_padding: Mutable<f64>, // TODO use u32 instead ?
}

impl Group {
    pub(crate) fn new(timestamp: f64, show_header: bool, name: Mutable<Option<Arc<String>>>, tabs: Vec<Arc<Tab>>) -> Self {
        lazy_static! {
            static ref ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        }

        Self {
            // TODO investigate whether it's possible to use a faster Ordering
            id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            show_header,
            timestamp,
            name,
            tabs: MutableVec::new_with_values(tabs),

            insert_animation: MutableAnimation::new_with_initial(INSERT_ANIMATION_DURATION, Percentage::new(0.0)),
            visible: Mutable::new(false),

            matches_search: Mutable::new(false),
            last_selected_tab: Mutable::new(None),

            drag_over: MutableAnimation::new(DRAG_ANIMATION_DURATION),
            drag_top: MutableAnimation::new(DRAG_ANIMATION_DURATION),
            tabs_padding: Mutable::new(0.0),
        }
    }

    pub(crate) fn click_tab(&self, tab: &Tab) {
        if !tab.selected.get() {
            {
                let tabs = self.tabs.lock_ref();

                for tab in tabs.iter() {
                    tab.selected.set_neq(false);
                }
            }

            self.last_selected_tab.set_neq(None);
        }
    }

    pub(crate) fn ctrl_select_tab(&self, tab: &Arc<Tab>) {
        let mut selected = tab.selected.lock_mut();

        *selected = !*selected;

        if *selected {
            self.last_selected_tab.set_neq(Some(tab.id));

        } else {
            self.last_selected_tab.set_neq(None);
        }
    }

    pub(crate) fn shift_select_tab(&self, tab: &Arc<Tab>) {
        let mut last_selected_tab = self.last_selected_tab.lock_mut();

        let selected = if let Some(last_selected_tab) = *last_selected_tab {
            let tabs = self.tabs.lock_ref();
            let mut seen = false;

            for x in tabs.iter() {
                if x.id == last_selected_tab ||
                   x.id == tab.id {
                    x.selected.set_neq(true);

                    if tab.id != last_selected_tab {
                        seen = !seen;
                    }

                } else if seen {
                    x.selected.set_neq(true);

                } else {
                    x.selected.set_neq(false);
                }
            }

            true

        } else {
            false
        };

        if !selected {
            tab.selected.set_neq(true);
            *last_selected_tab = Some(tab.id);
        }
    }
}