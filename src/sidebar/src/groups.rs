use crate::types::{State, TabState, Group, Tab, Window};
use crate::url_bar::UrlBar;
use tab_organizer::{str_default, round_to_hour, time, TimeDifference, StackVec};
use tab_organizer::state::{SidebarMessage, TabChange, SortTabs, Tag};
use js_sys::Date;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::cmp::Ordering;
use futures::Future;
use futures_signals::signal::{Mutable, SignalExt};
use futures_signals::signal_vec::{MutableVec, MutableVecLockMut};
use dominator::animation::Percentage;


// TODO better name
trait Insertable<A>: Deref<Target = [A]> {
    fn insert(&mut self, index: usize, value: A);
    fn remove(&mut self, index: usize);
    fn retain<F>(&mut self, f: F) where F: FnMut(&A) -> bool;
}

impl<A> Insertable<A> for Vec<A> {
    fn insert(&mut self, index: usize, value: A) {
        self.insert(index, value);
    }

    fn remove(&mut self, index: usize) {
        self.remove(index);
    }

    fn retain<F>(&mut self, f: F) where F: FnMut(&A) -> bool {
        self.retain(f)
    }
}

impl<'a, A> Insertable<A> for MutableVecLockMut<'a, A> where A: Clone {
    fn insert(&mut self, index: usize, value: A) {
        self.insert_cloned(index, value);
    }

    fn remove(&mut self, index: usize) {
        self.remove(index);
    }

    fn retain<F>(&mut self, f: F) where F: FnMut(&A) -> bool {
        self.retain(f)
    }
}


fn get_group_index_name(groups: &[Arc<Group>], name: &str) -> Result<usize, usize> {
    get_group_index(groups, |x| x.cmp(name))
}

fn get_group_index_name_empty(groups: &[Arc<Group>], name: &str) -> Result<usize, usize> {
    get_group_index(groups, |x| {
        // TODO make this more efficient ?
        if x == "" {
            if name == "" {
                Ordering::Equal

            } else {
                Ordering::Greater
            }

        } else if name == "" {
            Ordering::Less

        } else {
            x.cmp(name)
        }
    })
}

fn get_group_index<F>(groups: &[Arc<Group>], mut f: F) -> Result<usize, usize> where F: FnMut(&str) -> Ordering {
    groups.binary_search_by(|group: &Arc<Group>| {
        let name = group.name.lock_ref();
        let name = str_default(&name, "");
        f(name)
    })
}

fn get_timestamp_index(groups: &[Arc<Group>], timestamp: f64) -> Result<usize, usize> {
    groups.binary_search_by(|group: &Arc<Group>| {
        group.timestamp.partial_cmp(&timestamp).unwrap().reverse()
    })
}

fn get_tab_index<F>(tabs: &[Arc<Tab>], f: F) -> usize where F: FnMut(&Arc<Tab>) -> Ordering {
    tabs.binary_search_by(f).unwrap_err()
}

fn get_pinned_index(groups: &[Arc<Group>]) -> Result<usize, usize> {
    get_group_index(groups, |name| {
        if name == "Pinned" {
            Ordering::Equal

        } else {
            Ordering::Greater
        }
    })
}

fn get_unpinned_index(groups: &[Arc<Group>]) -> Result<usize, usize> {
    get_group_index(groups, |name| {
        if name == "Pinned" {
            Ordering::Less

        } else {
            Ordering::Equal
        }
    })
}

fn generate_timestamp_title(timestamp: f64, current_time: f64) -> String {
    TimeDifference::new(timestamp, round_to_hour(current_time)).pretty()
}


fn make_new_group(name: Option<Arc<String>>, timestamp: f64, should_animate: bool) -> Arc<Group> {
    let show_header = name.is_some();

    let group = Arc::new(Group::new(timestamp, show_header, Mutable::new(name), vec![]));

    if should_animate {
        group.insert_animation.animate_to(Percentage::new(1.0));

    } else {
        group.insert_animation.jump_to(Percentage::new(1.0));
    }

    group
}

fn insert_group<A, F>(groups: &mut A, index: Result<usize, usize>, create: F) -> Arc<Group>
    where A: Insertable<Arc<Group>>,
          F: FnOnce() -> Arc<Group> {
    match index {
        Ok(index) => {
            groups[index].clone()
        },

        Err(index) => {
            let group = create();

            groups.insert(index, group.clone());

            group
        },
    }
}


fn sorted_groups<A>(sort: SortTabs, groups: &mut A, tab: &TabState, should_animate: bool) -> StackVec<Arc<Group>> where A: Insertable<Arc<Group>> {
    match sort {
        SortTabs::Window => StackVec::Single({
            if tab.pinned.get() {
                let index = get_pinned_index(groups);
                insert_group(groups, index, || {
                    make_new_group(Some(Arc::new("Pinned".to_string())), 0.0, should_animate)
                })

            } else {
                let index = get_unpinned_index(groups);
                insert_group(groups, index, || {
                    make_new_group(None, 0.0, should_animate)
                })
            }
        }),

        SortTabs::Tag => {
            let tags = tab.tags.lock_ref();

            let f = |groups: &mut A, tag: &Tag| {
                let index = get_group_index_name_empty(groups, &tag.name);
                insert_group(groups, index, || {
                    // TODO make this clone more efficient (e.g. by using Arc for the tags)
                    make_new_group(Some(Arc::new(tag.name.clone())), 0.0, should_animate)
                })
            };

            match tags.as_slice() {
                // TODO test this
                [] => StackVec::Single({
                    // TODO guarantee that this puts this group first ?
                    let index = get_group_index_name_empty(groups, "");
                    insert_group(groups, index, || {
                        make_new_group(None, 0.0, should_animate)
                    })
                }),
                [tag] => StackVec::Single(f(groups, tag)),
                tags => StackVec::Multiple(tags.into_iter().map(|tag| f(groups, tag)).collect()),
            }
        },

        SortTabs::TimeFocused => StackVec::Single({
            let timestamp = round_to_hour(tab.timestamp_focused.get());

            let index = get_timestamp_index(groups, timestamp);
            insert_group(groups, index, || {
                // TODO pass in the current time, rather than generating it each time ?
                make_new_group(Some(Arc::new(generate_timestamp_title(timestamp, Date::now()))), timestamp, should_animate)
            })
        }),

        SortTabs::TimeCreated => StackVec::Single({
            let timestamp = round_to_hour(tab.timestamp_created.get());

            let index = get_timestamp_index(groups, timestamp);
            insert_group(groups, index, || {
                // TODO pass in the current time, rather than generating it each time ?
                make_new_group(Some(Arc::new(generate_timestamp_title(timestamp, Date::now()))), timestamp, should_animate)
            })
        }),

        SortTabs::Url => StackVec::Single({
            let url = tab.url.lock_ref();
            let url = str_default(&url, "");

            // TODO make this faster/more efficient
            let url =  UrlBar::new(url)
                .map(|url| url.minify())
                .map(|url| format!("{}{}{}{}{}",
                    str_default(&url.protocol, ""),
                    str_default(&url.separator, ""),
                    str_default(&url.authority, ""),
                    str_default(&url.domain, ""),
                    str_default(&url.port, "")))
                .unwrap_or_else(|| "".to_string());

            let index = get_group_index_name(groups, &url);
            insert_group(groups, index, || {
                make_new_group(Some(Arc::new(url)), 0.0, should_animate)
            })
        }),

        SortTabs::Name => StackVec::Single({
            let title = tab.title.lock_ref();
            let title = str_default(&title, "");
            let title = title.trim(); // TODO is it too expensive to use Unicode trim ?

            let title = title.chars().nth(0);

            let title = if let Some(char) = title {
                // TODO is it too expensive to use Unicode uppercase ?
                char.to_ascii_uppercase().to_string()

            } else {
                "".to_string()
            };

            let index = get_group_index_name(groups, &title);
            insert_group(groups, index, || {
                make_new_group(Some(Arc::new(title)), 0.0, should_animate)
            })
        }),
    }
}

fn sorted_tab_index(sort: SortTabs, tabs: &[Arc<Tab>], tab: &TabState, tab_index: usize, is_initial: bool) -> usize {
    match sort {
        SortTabs::Window | SortTabs::Tag => {
            if is_initial {
                tabs.len()

            } else {
                get_tab_index(tabs, |tab| {
                    tab.index.get().cmp(&tab_index)
                })
            }
        },

        SortTabs::TimeFocused => {
            let timestamp = tab.timestamp_focused.get();

            get_tab_index(tabs, |tab| {
                tab.timestamp_focused.get().partial_cmp(&timestamp).unwrap().then_with(|| {
                    tab.index.get().cmp(&tab_index)
                }).reverse()
            })
        },

        SortTabs::TimeCreated => {
            let timestamp = tab.timestamp_created.get();

            get_tab_index(tabs, |tab| {
                tab.timestamp_created.get().partial_cmp(&timestamp).unwrap().then_with(|| {
                    tab.index.get().cmp(&tab_index)
                }).reverse()
            })
        },

        SortTabs::Url => {
            let url = tab.url.lock_ref();
            let url = str_default(&url, "");

            get_tab_index(tabs, |tab| {
                let x = tab.url.lock_ref();
                let x = str_default(&x, "");

                x.cmp(url).then_with(|| {
                    tab.index.get().cmp(&tab_index)
                })
            })
        },

        // TODO use upper-case sorting ?
        SortTabs::Name => {
            let title = tab.title.lock_ref();
            let title = str_default(&title, "");

            get_tab_index(tabs, |tab| {
                let x = tab.title.lock_ref();
                let x = str_default(&x, "");

                x.cmp(title).then_with(|| {
                    tab.index.get().cmp(&tab_index)
                })
            })
        },
    }
}


fn initialize(state: &State, sort: SortTabs, window: &Window, should_animate: bool) -> Vec<Arc<Group>> {
    let mut groups = vec![];

    for (tab_index, tab) in window.tabs.iter().cloned().enumerate() {
        tab_inserted(state, sort, &mut groups, tab, tab_index, should_animate, true);
    }

    groups
}

fn insert_tab_into_group(state: &State, sort: SortTabs, group: &Group, tab: Arc<TabState>, tab_index: usize, should_animate: bool, is_initial: bool) {
    let tab = Arc::new(Tab::new(tab));

    if should_animate {
        tab.insert_animation.animate_to(Percentage::new(1.0));

    } else {
        tab.insert_animation.jump_to(Percentage::new(1.0));
    }

    // TODO is this correct ?
    if !is_initial {
        state.search_tab(&group, &tab);
    }

    let mut tabs = group.tabs.lock_mut();

    let index = sorted_tab_index(sort, &tabs, &tab, tab_index, is_initial);

    tabs.insert_cloned(index, tab);
}

fn tab_inserted<A>(state: &State, sort: SortTabs, groups: &mut A, tab: Arc<TabState>, tab_index: usize, should_animate: bool, is_initial: bool) where A: Insertable<Arc<Group>> {
    sorted_groups(sort, groups, &tab, should_animate).each(|group| {
        // TODO if the tab doesn't match the search, and the group is already matching, then do nothing
        insert_tab_into_group(state, sort, &group, tab.clone(), tab_index, should_animate, is_initial);
    });
}

fn remove_group<A>(groups: &mut A, group: &Group) where A: Insertable<Arc<Group>> {
    let id = group.id;

    // TODO make this more efficient ?
    groups.retain(|group| {
        if group.id == id {
            group.insert_animation.animate_to(Percentage::new(0.0));
            false

        } else {
            true
        }
    });
}

fn remove_tab_from_group<A>(groups: &mut A, group: &Group, tab: &TabState, should_remove_group: bool) where A: Insertable<Arc<Group>> {
    let mut tabs = group.tabs.lock_mut();

    let id = tab.id;

    // TODO make this more efficient ?
    tabs.retain(|tab| {
        if tab.id == id {
            tab.insert_animation.animate_to(Percentage::new(0.0));
            false

        } else {
            true
        }
    });

    if should_remove_group {
        if tabs.len() == 0 {
            remove_group(groups, group);

        } else {
            drop(tabs);
            State::update_group_search(group, false);
        }
    }
}

fn tab_removed(sort: SortTabs, groups: &mut MutableVecLockMut<Arc<Group>>, tab: &TabState, _tab_index: usize) {
    // TODO make this more efficient
    // TODO should this be animated ?
    sorted_groups(sort, groups, tab, true).each(|group| {
        remove_tab_from_group(groups, &group, tab, true);
    });
}

fn tab_updated<A>(state: &State, sort: SortTabs, groups: &mut A, old_groups: StackVec<Arc<Group>>, tab: Arc<TabState>, tab_index: usize) where A: Insertable<Arc<Group>> {
    let new_groups = sorted_groups(sort, groups, &tab, true);

    // TODO make this more efficient
    old_groups.each(|group| {
        let id = group.id;

        // TODO make this more efficient ?
        let is_in_new_group = new_groups.any(|group| group.id == id);

        // Remove the group if the group does not exist in new_groups AND group is empty
        remove_tab_from_group(groups, &group, &tab, !is_in_new_group);
    });

    new_groups.each(|group| {
        insert_tab_into_group(state, sort, &group, tab.clone(), tab_index, true, false);
    });
}


#[derive(Debug)]
pub(crate) struct Groups {
    sort: Mutex<SortTabs>,
    groups: MutableVec<Arc<Group>>,
}

impl Groups {
    pub(crate) fn new(sort_tabs: SortTabs) -> Self {
        Self {
            sort: Mutex::new(sort_tabs),
            groups: MutableVec::new(),
        }
    }

    pub(crate) fn initialize(&self, state: &State) {
        let window = state.window.read().unwrap();

        let sort = self.sort.lock().unwrap();
        let mut groups = self.groups.lock_mut();

        assert_eq!(groups.len(), 0);

        let new_groups = time!("Creating initial groups", { initialize(state, *sort, &window, false) });
        groups.replace_cloned(new_groups);
    }

    fn update_group_titles(&self) {
        let should_update = {
            let sort = self.sort.lock().unwrap();

            // TODO replace with `if let`
            match *sort {
                SortTabs::TimeCreated | SortTabs::TimeFocused => true,
                _ => false,
            }
        };

        if should_update {
            let groups = self.groups.lock_ref();

            let current_time = Date::now();

            for group in groups.iter() {
                // TODO only update if the new title is different from the old title
                group.name.set(Some(Arc::new(generate_timestamp_title(group.timestamp, current_time))));
            }
        }
    }

    fn change_sort(&self, state: &State, sort_tabs: SortTabs, window: &Window) {
        let mut sort = self.sort.lock().unwrap();

        let mut groups = self.groups.lock_mut();

        // This is necessary because other parts of the code use delay_remove
        for group in groups.iter() {
            group.insert_animation.jump_to(Percentage::new(0.0));

            let tabs = group.tabs.lock_ref();

            for tab in tabs.iter() {
                tab.insert_animation.jump_to(Percentage::new(0.0));
            }
        }

        *sort = sort_tabs;

        let new_groups = time!("Creating new groups", { initialize(state, *sort, window, false) });

        groups.replace_cloned(new_groups);
    }

    fn tab_inserted(&self, state: &State, tab_index: usize, tab: Arc<TabState>) {
        let sort = self.sort.lock().unwrap();
        let mut groups = self.groups.lock_mut();
        tab_inserted(state, *sort, &mut groups, tab, tab_index, true, false);
    }

    fn tab_removed(&self, tab_index: usize, tab: &TabState) {
        let sort = self.sort.lock().unwrap();
        let mut groups = self.groups.lock_mut();
        tab_removed(*sort, &mut groups, tab, tab_index);
    }

    fn tab_updated<F>(&self, state: &State, tab_index: usize, tab: Arc<TabState>, change: F) where F: FnOnce() {
        let sort = self.sort.lock().unwrap();
        let mut groups = self.groups.lock_mut();

        // TODO should this be animated ?
        let group_indexes = sorted_groups(*sort, &mut groups, &tab, true);

        change();

        tab_updated(state, *sort, &mut groups, group_indexes, tab, tab_index);
    }
}

impl Deref for Groups {
    type Target = MutableVec<Arc<Group>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}


impl Group {
    pub(crate) fn wait_until_removed(&self) -> impl Future<Output = ()> {
        let signal = self.insert_animation.signal();
        async { signal.wait_for(Percentage::new(0.0)).await; }
    }
}


impl Tab {
    pub(crate) fn wait_until_removed(&self) -> impl Future<Output = ()> {
        let signal = self.insert_animation.signal();
        async { signal.wait_for(Percentage::new(0.0)).await; }
    }
}


impl State {
    pub(crate) fn process_message(&self, message: SidebarMessage) {
        fn increment_indexes(tabs: &[Arc<TabState>]) {
            for tab in tabs {
                tab.index.replace_with(|index| *index + 1);
            }
        }

        fn decrement_indexes(tabs: &[Arc<TabState>]) {
            for tab in tabs {
                tab.index.replace_with(|index| *index - 1);
            }
        }

        match message {
            SidebarMessage::TabInserted { tab_index, tab } => {
                let mut window = self.window.write().unwrap();

                let tab = Arc::new(TabState::new(tab, tab_index));

                increment_indexes(&window.tabs[tab_index..]);

                window.tabs.insert(tab_index, tab.clone());

                self.groups.tab_inserted(self, tab_index, tab);
            },

            SidebarMessage::TabRemoved { tab_index } => {
                let mut window = self.window.write().unwrap();

                let tab = window.tabs.remove(tab_index);

                tab.removed.set_neq(true);

                self.groups.tab_removed(tab_index, &tab);

                decrement_indexes(&window.tabs[tab_index..]);
            },

            SidebarMessage::TabChanged { tab_index, changes } => {
                let window = self.window.read().unwrap();

                let tab = &window.tabs[tab_index];

                self.groups.tab_updated(self, tab_index, tab.clone(), || {
                    for change in changes {
                        match change {
                            TabChange::Title { new_title } => {
                                tab.title.set(new_title.map(Arc::new));
                            },
                            TabChange::Pinned { pinned } => {
                                tab.pinned.set_neq(pinned);
                            },
                            TabChange::AddedToTag { tag } => {
                                let mut tags = tab.tags.lock_mut();
                                assert!(tags.iter().all(|x| x.name != tag.name));
                                tags.push(tag);
                            },
                            TabChange::RemovedFromTag { tag_name } => {
                                let mut tags = tab.tags.lock_mut();
                                // TODO use remove_item
                                let index = tags.iter().position(|x| x.name == tag_name).unwrap();
                                tags.remove(index);
                            },
                        }
                    }
                });
            },
        }
    }

    pub(crate) fn change_sort(&self, sort_tabs: SortTabs) {
        let window = self.window.read().unwrap();
        self.groups.change_sort(self, sort_tabs, &window);
    }

    pub(crate) fn update_group_titles(&self) {
        self.groups.update_group_titles();
    }
}