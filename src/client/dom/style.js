import { each, entries } from "../../util/iterator";


const add_rules = (() => {
  const prefixes = {
    // TODO it's a bit hacky to use the prefix system for this purpose...
    //"width": ["width", "min-width", "max-width"],
    //"height": ["height", "min-height", "max-height"],
    "box-sizing": ["-moz-box-sizing", "box-sizing"] // TODO get rid of this later
  };

  return (style, rules) => {
    each(entries(rules), ([key, value]) => {
      const keys = (prefixes[key]
                     ? prefixes[key]
                     : [key]);

      const old_values = keys["map"]((key) => style["getPropertyValue"](key));

      const new_values = keys["map"]((key) => {
        // The third argument must be ""
        // http://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-setpropertyproperty-value-priority
        style["setProperty"](key, value, "");

        return style["getPropertyValue"](key);
      });

      const every = new_values["every"]((new_value, i) => {
        const old_value = old_values[i];
        // TODO  && old_value !== value
        return new_value === old_value;
      });

      if (every) {
        throw new Error("Invalid key or value (\"" + key + "\": \"" + value + "\")");
      }
    });
  };
})();


class Style {
  constructor(name, style, rules) {
    this._name = name;
    this._rules = rules;
    this._style = style;
    // TODO a little hacky
    this._keys = Object["keys"](rules);
  }
}

export const make_style = (() => {
  let style_id = 0;

  // TODO use batch_write ?
  var e = document["createElement"]("style");
  e["type"] = "text/css";
  document["head"]["appendChild"](e);

  const sheet = e["sheet"];
  const cssRules = sheet["cssRules"];

  return (rules) => {
    const class_name = "__style_" + (++style_id) + "__";

    //batch_write(() => {
      // TODO this may not work in all browsers
      const index = sheet["insertRule"]("." + class_name + "{}", cssRules["length"]); // TODO sheet.addRule(s)

      const style = cssRules[index]["style"];

      add_rules(style, rules);
    //});

    return new Style(class_name, style, rules);
  };
})();