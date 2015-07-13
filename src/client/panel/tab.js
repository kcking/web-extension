import * as dom from "../dom";
import { animate, ease_in_out, range, round_range } from "../../util/animate";
import { concat, Stream } from "../../util/stream";


const ui_tab_style_hidden = dom.style({
  /*"transform": {
    "rotationX": "-90deg", // 120deg
    "rotationY": "5deg", // 20deg
    //"rotationZ": "-1deg", // -1deg
  },*/

  "border-top-width": "0px",
  "border-bottom-width": "0px",
  "padding-top": "0px",
  "padding-bottom": "0px",
  "height": "0px",
  "opacity": "0"
});

const ui_tab_style = dom.style({
  "border-width": "1px",
  "padding": "1px",
  "height": "20px",
  "border-radius": "5px",

  "cursor": "pointer",
  "transition-property": "background-color",
  "transition-timing-function": "ease-in-out",

  "transform-origin": "11px 50%",
  "transform": "translate3d(0, 0, 0)", /* TODO this is a hack to make animation smoother, should replace with something else */

  "text-shadow": "0px 1px 1px " + dom.hsl(211, 61, 50, 0.1),
  "transition-duration": "100ms"
});

const repeating = dom.repeating_gradient("-45deg",
                                         ["0px",  "transparent"],
                                         ["4px",  "transparent"],
                                         ["6px",  dom.hsl(0, 0, 100, 0.05)],
                                         ["10px", dom.hsl(0, 0, 100, 0.05)]);

const ui_tab_style_hover = dom.style({
  "font-weight": "bold",

  "transition-duration": "0ms",
  "background-image": dom.gradient("to bottom",
                                   ["0%",   dom.hsl(0, 0, 100, 0.2)],
                                   ["49%",  "transparent"          ],
                                   ["50%",  dom.hsl(0, 0,   0, 0.1)],
                                   ["80%",  dom.hsl(0, 0, 100, 0.1)],
                                   ["100%", dom.hsl(0, 0, 100, 0.2)]) + "," +
                      repeating,
  "box-shadow":       "1px 1px  1px " + dom.hsl(0, 0,   0, 0.25) + "," +
                "inset 0px 0px  3px " + dom.hsl(0, 0, 100, 1   ) + "," +
                "inset 0px 0px 10px " + dom.hsl(0, 0, 100, 0.25),
  "color": dom.hsl(211, 100, 99, 0.95),
  "background-color": dom.hsl(211, 100, 65),
  "border-color": dom.hsl(211, 38, 57),
  "text-shadow": "1px 0px 1px " + dom.hsl(211, 61, 50) + "," +
                 "0px 1px 1px " + dom.hsl(211, 61, 50)
});

const ui_tab_style_hold = dom.style({
  "padding-top": "2px",
  "padding-bottom": "0px",

  "background-position": "0px 1px",
  "background-image": dom.gradient("to bottom",
                                   ["0%",   dom.hsl(0, 0, 100, 0.2)  ],
                                   ["49%",  "transparent"            ],
                                   ["50%",  dom.hsl(0, 0,   0, 0.075)],
                                   ["80%",  dom.hsl(0, 0, 100, 0.1)  ],
                                   ["100%", dom.hsl(0, 0, 100, 0.2)  ]) + "," +
                      repeating,
  "box-shadow":       "1px 1px 1px "  + dom.hsl(0, 0,   0, 0.1) + "," +
                "inset 0px 0px 3px "  + dom.hsl(0, 0, 100, 0.9) + "," +
                "inset 0px 0px 10px " + dom.hsl(0, 0, 100, 0.225),
});

const ui_tab_icon_style = dom.style({
  "height": "16px",
  "border-radius": "4px",
  "box-shadow": "0px 0px 15px " + dom.hsl(0, 0, 100, 0.9),
  "background-color": dom.hsl(0, 0, 100, 0.35)
});

const ui_tab_favicon_style = dom.style({
  "width": "16px"
});

const ui_tab_text_style = dom.style({
  "padding-left": "2px",
  "padding-right": "2px"
});

const ui_tab_close_style = dom.style({
  "width": "18px",
  "border-width": "1px",
  "padding-left": "1px",
  "padding-right": "1px"
});

const ui_favicon = (tab) =>
  dom.image((e) => {
    e.add_style(ui_tab_icon_style);
    e.add_style(ui_tab_favicon_style);
    e.set_url(tab.get("favicon"));
  });

const ui_text = (tab) =>
  dom.stretch((e) => {
    e.add_style(ui_tab_text_style);
    e.push(dom.text(tab.get("title") || tab.get("url")));
  });

const ui_close = (tab) =>
  dom.image((e) => {
    e.add_style(ui_tab_icon_style);
    e.add_style(ui_tab_close_style);

    /*e.on_hover.each((hover) => {
      e.set_style(ui_tab_close_style_hover, hover);
    });

    e.on_hold.each((hold) => {
      e.set_style(ui_tab_close_style_hold, hold);
    });*/

    e.set_url("data/images/button-close.png");
  });

export const ui_tab = (tab) =>
  dom.row((e) => {
    e.add_style(ui_tab_style);

    e.on_hover.each((hover) => {
      e.set_style(ui_tab_style_hover, hover);
    });

    e.on_hold.each((hold) => {
      e.set_style(ui_tab_style_hold, hold);
    });

    const random = Stream((send, error, complete) => {
      setTimeout(() => {
        complete();
      }, Math["random"]() * 2000);
    });

    concat([
      //e.animate_from(ui_tab_style_hidden),

      //random,

      //e.animate_to(ui_tab_style_hidden),

      // /[0-9]+(px)?/

      animate(1000).map(ease_in_out).map((t) => {
        if (t === 1) {
          e._dom.style["border-top-width"] = "";
          e._dom.style["border-bottom-width"] = "";
          e._dom.style["padding-top"] = "";
          e._dom.style["padding-bottom"] = "";
          e._dom.style["height"] = "";
          e._dom.style["opacity"] = "";

        } else {
          e._dom.style["border-top-width"] = round_range(t, 0, 1) + "px";
          e._dom.style["border-bottom-width"] = round_range(t, 0, 1) + "px";
          e._dom.style["padding-top"] = round_range(t, 0, 1) + "px";
          e._dom.style["padding-bottom"] = round_range(t, 0, 1) + "px";
          e._dom.style["height"] = round_range(t, 0, 20) + "px";
          e._dom.style["opacity"] = range(t, 0, 1) + "";
        }
      }),

      animate(1000).map(ease_in_out).map((t) => {
        if (t === 1) {
          e._dom.style["border-top-width"] = "";
          e._dom.style["border-bottom-width"] = "";
          e._dom.style["padding-top"] = "";
          e._dom.style["padding-bottom"] = "";
          e._dom.style["height"] = "";
          e._dom.style["opacity"] = "";

        } else {
          e._dom.style["border-top-width"] = round_range(t, 1, 0) + "px";
          e._dom.style["border-bottom-width"] = round_range(t, 1, 0) + "px";
          e._dom.style["padding-top"] = round_range(t, 1, 0) + "px";
          e._dom.style["padding-bottom"] = round_range(t, 1, 0) + "px";
          e._dom.style["height"] = round_range(t, 20, 0) + "px";
          e._dom.style["opacity"] = range(t, 1, 0) + "";
        }
      })
    ]).forever().drain();

    e.push(ui_favicon(tab));
    e.push(ui_text(tab));
    e.push(ui_close(tab));
  });

/*animate(1000).map(ease_in_out).each((t) => {
  console.log(range(t, 0, 20));
});*/
