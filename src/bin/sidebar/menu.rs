use {visible, ROW_STYLE, STRETCH_STYLE, MENU_ITEM_HOVER_STYLE};
use futures_signals::signal::{Signal, IntoSignal, SignalExt, Mutable};
use dominator::{Dom, DomBuilder, text, HIGHEST_ZINDEX};
use dominator::events::{MouseEnterEvent, MouseLeaveEvent, ClickEvent};
use stdweb::web::IElement;


lazy_static! {
    static ref TOP_STYLE: String = class! {
        .style("position", "absolute")
        .style("top", "100%")
        .style("right", "0px")
        .style("z-index", HIGHEST_ZINDEX)
    };

    static ref MODAL_STYLE: String = class! {
        .style("position", "fixed")
        .style("left", "0px")
        .style("top", "0px")
        .style("width", "100%")
        .style("height", "100%")
        .style("background-color", "hsla(0, 0%, 0%, 0.15)")
    };

    static ref MENU_STYLE: String = class! {
        //.style("overflow", "hidden")
        .style("border", "1px solid black")
        .style("background-color", "white")
        .style("white-space", "pre")
        .style("box-shadow", "1px 1px 2px hsla(0, 0%, 0%, 0.25)")
    };

    static ref MENU_CHEVRON_STYLE: String = class! {
        .style("width", "7px")
        .style("height", "7px")
        .style("margin-left", "5px")
        .style("margin-right", "-3px")
    };

    static ref SUBMENU_CHILDREN_STYLE: String = class! {
        .style("position", "absolute")
        .style("top", "-1px")
        .style("right", "100%")
    };

    static ref MENU_ITEM_STYLE: String = class! {
        .style("margin-top", "-1px")
        .style("margin-bottom", "-1px")
        .style("padding-top", "1px")
        .style("padding-bottom", "1px")
        .style("padding-left", "5px")
        .style("padding-right", "5px")
        .style("color", "black")
        .style("text-shadow", "none")
        .style("border-left", "none")
        .style("border-right", "none")
    };

    static ref MENU_OPTION_SELECTED_STYLE: String = class! {
        .style("font-weight", "bold")
    };

    // TODO code duplication with main.rs
    static ref MENU_ITEM_SHADOW_STYLE: String = class! {
        .style("box-shadow", "      0px 1px  1px hsla(0, 0%,   0%, 0.25), \
                              inset 0px 0px  3px hsla(0, 0%, 100%, 1   ), \
                              inset 0px 0px 10px hsla(0, 0%, 100%, 0.25)")
    };

    static ref SEPARATOR_STYLE: String = class! {
        .style("background-color", "gainsboro")
        .style("margin", "2px 3px")
        .style("height", "1px")
    };
}


fn eq_index<A>(signal: A, index: usize) -> impl Signal<Item = bool> where A: IntoSignal<Item = Option<usize>> {
    signal.into_signal().map(move |hovered| {
        hovered.map(|hovered| hovered == index).unwrap_or(false)
    })
}


enum MenuState {
    Submenu {
        hovered: Mutable<Option<usize>>,
        children: Vec<MenuState>,
    },
    Item {
        hovered: Mutable<bool>,
    },
}

impl MenuState {
    fn reset(&self) {
        match self {
            MenuState::Submenu { hovered, children } => {
                hovered.set_neq(None);

                for state in children {
                    state.reset();
                }
            },
            MenuState::Item { hovered } => {
                hovered.set_neq(false);
            },
        }
    }
}


pub(crate) struct MenuBuilder {
    states: Vec<MenuState>,
    children: Vec<Dom>,
    hovered: Mutable<Option<usize>>,
}

impl MenuBuilder {
    fn new() -> Self {
        Self {
            states: vec![],
            children: vec![],
            hovered: Mutable::new(None),
        }
    }

    fn menu_item<A>(&mut self) -> impl FnOnce(DomBuilder<A>) -> DomBuilder<A> where A: IElement + Clone + 'static {
        let index = self.children.len();

        let mutable = self.hovered.clone();

        let hovered = Mutable::new(false);

        self.states.push(MenuState::Item {
            hovered: hovered.clone(),
        });

        // TODO is this inline a good idea ?
        #[inline]
        move |dom| { dom
            .class(&ROW_STYLE)
            .class(&MENU_ITEM_STYLE)
            // TODO hacky
            .class(&super::MENU_ITEM_STYLE)

            .class_signal(&MENU_ITEM_HOVER_STYLE, hovered.signal())
            .class_signal(&MENU_ITEM_SHADOW_STYLE, hovered.signal())

            .event(clone!(hovered => move |_: MouseEnterEvent| {
                hovered.set_neq(true);
                mutable.set_neq(Some(index));
            }))

            .event(move |_: MouseLeaveEvent| {
                hovered.set_neq(false);
            })
        }
    }


    fn push_submenu<F>(&mut self, name: &str, f: F) where F: FnOnce(MenuBuilder) -> MenuBuilder {
        let index = self.children.len();

        let MenuBuilder { states, mut children, hovered } = f(MenuBuilder::new());

        self.states.push(MenuState::Submenu {
            hovered,
            children: states,
        });

        self.children.push(html!("div", {
            .class(&ROW_STYLE)
            .class(&MENU_ITEM_STYLE)
            // TODO hacky
            .class(&super::MENU_ITEM_STYLE)

            .class_signal(&MENU_ITEM_HOVER_STYLE, eq_index(self.hovered.signal(), index))
            .class_signal(&MENU_ITEM_SHADOW_STYLE, eq_index(self.hovered.signal(), index))

            // TODO make this cleaner
            .event({
                let hovered = self.hovered.clone();
                move |_: MouseEnterEvent| {
                    hovered.set_neq(Some(index));
                }
            })

            .children(&mut [
                // TODO figure out a way to avoid this wrapper div ?
                html!("div", {
                    .class(&STRETCH_STYLE)
                    .children(&mut [
                        text(name),
                    ])
                }),

                html!("img", {
                    .class(&MENU_CHEVRON_STYLE)
                    .attribute("src", "data/images/chevron-small-right.png")
                }),

                html!("div", {
                    .class(&MENU_STYLE)
                    .class(&SUBMENU_CHILDREN_STYLE)

                    .mixin(visible(eq_index(self.hovered.signal(), index)))

                    .children(&mut children)
                })
            ])
        }));
    }

    #[inline]
    pub(crate) fn submenu<F>(mut self, name: &str, f: F) -> Self where F: FnOnce(MenuBuilder) -> MenuBuilder {
        self.push_submenu(name, f);
        self
    }


    fn push_separator(&mut self) {
        self.children.push(html!("hr", {
            .class(&SEPARATOR_STYLE)
        }));
    }

    #[inline]
    pub(crate) fn separator(mut self) -> Self {
        self.push_separator();
        self
    }


    fn push_option<A, F>(&mut self, name: &str, signal: A, mut on_click: F)
        where A: IntoSignal<Item = bool>,
              A::Signal: 'static,
              F: FnMut() + 'static {

        let mixin = self.menu_item();

        self.children.push(html!("div", {
            .mixin(mixin)

            .class_signal(&MENU_OPTION_SELECTED_STYLE, signal.into_signal())

            .event(move |_: ClickEvent| {
                on_click();
            })

            .children(&mut [
                text(name)
            ])
        }));
    }

    #[inline]
    pub(crate) fn option<A, F>(mut self, name: &str, signal: A, on_click: F) -> Self
        where A: IntoSignal<Item = bool>,
              A::Signal: 'static,
              F: FnMut() + 'static {
        self.push_option(name, signal, on_click);
        self
    }
}


pub(crate) struct Menu {
    visible: Mutable<bool>,
}

impl Menu {
    pub(crate) fn new() -> Self {
        Self {
            visible: Mutable::new(false),
        }
    }

    pub(crate) fn show(&self) {
        self.visible.set_neq(true);
    }

    pub(crate) fn render<F>(&self, f: F) -> Dom where F: FnOnce(MenuBuilder) -> MenuBuilder {
        let MenuBuilder { states, mut children, hovered } = f(MenuBuilder::new());

        html!("div", {
            .class(&TOP_STYLE)

            .mixin(visible(self.visible.signal()))

            .children(&mut [
                html!("div", {
                    .class(&MODAL_STYLE)

                    // TODO make this cleaner
                    .event({
                        let visible = self.visible.clone();
                        move |_: ClickEvent| {
                            visible.set_neq(false);

                            hovered.set_neq(None);

                            for state in states.iter() {
                                state.reset();
                            }
                        }
                    })
                }),

                html!("div", {
                    .class(&MENU_STYLE)

                    .children(&mut children)
                }),
            ])
        })
    }
}
