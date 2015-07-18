import { performance } from "../common/globals";
import { Stream } from "./stream";


let batching = false;
let batches  = [];
let start    = null;

const batch = (f) => {
  if (!batching) {
    batching = true;
    // This is to ensure that animations that start on the same frame get
    // the same start time, so that they stay in sync
    start = performance["now"]();

    requestAnimationFrame(() => {
      // This is to ensure that animations that start on the same frame get
      // the same current time, so that they stay in sync
      const now = performance["now"]();

      const a = batches;

      // Have to reset these before calling the batched functions,
      // because one of the batched functions might call `batch`
      batching = false;
      batches  = [];

      // Call the batched functions
      for (let i = 0; i < a["length"]; ++i) {
        a[i](now);
      }
    });
  }

  batches["push"](f);

  return start;
};

// TODO test this
const unbatch = (f) => {
  const index = batches["indexOf"](f);
  if (index !== -1) {
    batches["splice"](index, 1);
  }
};


export const animate = (duration) =>
  Stream((send, error, complete) => {
    send(0);

    const loop = (now) => {
      // TODO the first frame seems to fire too quickly (e.g. ~4ms after start time)
      const diff = now - start;

      if (diff >= duration) {
        send(1);
        complete();

      } else {
        batch(loop);
        send(diff / duration);
      }
    };

    const start = batch(loop);

    // TODO test this
    return () => {
      unbatch(loop);
    };
  });

// TODO test this
export const ease_in_out = (t) => {
  let r = (t < 0.5
            ? t * 2
            : (1 - t) * 2);
  r *= r * r * r;
  return (t < 0.5
           ? r / 2
           : 1 - (r / 2));
};

export const range = (t, from, to) =>
  (t * (to - from)) + from;

export const round_range = (t, from, to) =>
  Math["round"](range(t, from, to));