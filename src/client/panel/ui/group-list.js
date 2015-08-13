import * as dom from "../../dom";
import { async } from "../../../util/async";
import { always } from "../../../util/mutable/ref";
import { init as init_group } from "./group";
import { init as init_options } from "../../sync/options";


export const init = async(function* () {
  const { group: ui_group } = yield init_group;
  const { get: opt } = yield init_options;


  const style_group_list = dom.style({
    // TODO really hacky
    // This has to match with the height of the search bar
    "height": always("calc(100% - 26px)"),

    "padding": opt("groups.layout").map((x) => {
      switch (x) {
      case "horizontal":
        return "6px";
      case "grid":
        return "2px";
      default:
        return "4px 0px 0px 0px";
      }
    }),

    "overflow": always("auto"),
  });

  const style_group_children = dom.style({
    "overflow": opt("groups.layout").map((x) => {
      switch (x) {
      case "grid":
      case "horizontal":
        return "visible";
      default:
        return null;
      }
    }),

    "width": always("100%"),

    "height": opt("groups.layout").map((x) => {
      switch (x) {
      case "grid":
      case "horizontal":
        return "100%";
      default:
        return null;
      }
    }),

    "padding": opt("groups.layout").map((x) => {
      switch (x) {
      case "horizontal":
        return "0px 190px 0px 0px"
      default:
        return null;
      }
    }),

    "justify-content": opt("groups.layout").map((x) => {
      switch (x) {
      case "horizontal":
        // TODO the animation when inserting a new group is slightly janky
        //      (it's smooth when using "center", but janky when using
        //      "space-between")
        return "space-between";
      default:
        return null;
      }
    }),
  });


  const scroll_x = +(localStorage["popup.scroll.x"] || 0);
  const scroll_y = +(localStorage["popup.scroll.y"] || 0);


  // "grid" layout is neither horizontal nor vertical,
  // because it uses "float: left"
  const is_horizontal = opt("groups.layout").map((x) => x === "horizontal");


  const group_list = (groups) =>
    dom.parent((e) => [
      //e.set_style(dom.stretch, always(true)),
      e.set_style(style_group_list, always(true)),

      e.set_scroll({
        // TODO a little hacky
        x: always(scroll_x),
        // TODO a little hacky
        y: always(scroll_y)
      }),

      e.on_scroll(({ x, y }) => {
        localStorage["popup.scroll.x"] = "" + x;
        localStorage["popup.scroll.y"] = "" + y;
      }),

      // TODO this is pretty hacky, but I don't know a better way to make it work
      e.children([
        dom.parent((e) => [
          e.set_style(dom.row, is_horizontal),
          e.set_style(style_group_children, always(true)),

          e.children(groups.map(ui_group))
        ])
      ])
    ]);


  return { group_list };
});
