import { chrome } from "../../common/globals";
import { Timer } from "../../util/time";
import { Dict } from "../../util/dict";
import { async } from "../../util/async";
import { async_chrome, throw_error } from "../common/util";
import { assert } from "../../util/assert";


// TODO move this into another module
const warn_if = (test, ...args) => {
  if (test) {
    console["warn"](...args);
  } else {
    console["debug"](...args);
  }
};


// TODO this is O(n)
const serialize = (value) => {
  if (value === null ||
      value === true ||
      value === false ||
      typeof value === "number" ||
      typeof value === "string") {
    return value;

  } else if (Array["isArray"](value)) {
    const o = new Array(value["length"]);

    for (let i = 0; i < value["length"]; ++i) {
      o[i] = serialize(value[i]);
    }

    return o;

  } else if (typeof value === "object") {
    if (typeof value["toJSON"] === "function") {
      return serialize(value["toJSON"]());

    } else {
      const o = {};

      // TODO function for this ?
      // TODO hasOwnProperty
      // TODO is this slow ?
      for (let s in value) {
        o[s] = serialize(value[s]);
      }

      return o;
    }

  } else {
    throw new Error("Cannot serialize: " + value);
  }
};


export const init = async(function* () {
  const timer = new Timer();

  let db = yield async_chrome((callback) => {
    chrome["storage"]["local"]["get"](null, callback);
  });

  timer.done();

  let setting = false;


  const delaying = new Dict();

  const with_delay = (key, f) => {
    if (delaying.has(key)) {
      delaying.get(key).thunk = f;
    } else {
      f();
    }
  };

  const delay = (key, ms) => {
    if (delaying.has(key)) {
      const info = delaying.get(key);

      if (ms <= info.ms) {
        return;
      }
    }

    const o = {
      thunk: null,
      ms: ms,
      timer: setTimeout(() => {
        if (delaying.get(key) === o) {
          delaying.remove(key);
        }

        o.thunk();
      }, ms)
    };

    delaying.set(key, o);
  };


  const get = (key, def) => {
    if (key in db) {
      return db[key];
    } else {
      return def;
    }
  };

  const set = (key, value) => {
    assert(!setting);

    delay(key, 1000);

    const timer_serialize = new Timer();
    const s_value = serialize(value);
    timer_serialize.done();

    db[key] = s_value;

    with_delay(key, () => {
      const timer = new Timer();

      // It's okay for this to be asynchronous, because
      // `serialize` makes a copy of `value`
      chrome["storage"]["local"]["set"]({ [key]: s_value }, () => {
        throw_error();
        timer.done();

        warn_if((timer.diff() + timer_serialize.diff()) >= 1000,
                "db.set: \"" +
                key +
                "\" (serialize " +
                timer_serialize.diff() +
                "ms) (commit " +
                timer.diff() +
                "ms)");
      });
    });
  };

  // TODO maybe make a copy ?
  const get_all = () => db;

  // TODO somehow guarantee that the db can't be changed until this is done ?
  const set_all = (o) => {
    assert(!setting);

    setting = true;

    const v = serialize(o);

    db = v;

    return async(function* () {
      try {
        yield async_chrome((callback) => {
          chrome["storage"]["local"]["clear"](callback);
        });

        yield async_chrome((callback) => {
          chrome["storage"]["local"]["set"](v, callback);
        });

      } finally {
        console.log("done");
        setting = false;
      }
    });
  };

  const remove = (key) => {
    assert(!setting);

    if (key in db) {
      delay(key, 1000);

      delete db[key];

      with_delay(key, () => {
        const timer = new Timer();

        // TODO test whether we need to batch this or not
        chrome["storage"]["local"]["remove"](key, () => {
          throw_error();
          timer.done();

          warn_if(timer.diff() >= 1000,
                  "db.remove: \"" +
                  key +
                  "\" (remove " +
                  timer.diff() +
                  "ms)");
        });
      });

    } else {
      throw new Error("Key not found: " + key);
    }
  };

  console["debug"]("db: initialized (" + timer.diff() + "ms)", db);

  return {
    get,
    set,
    remove,
    delay,
    get_all,
    set_all
  };
});
