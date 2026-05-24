import { makeSlide } from "./deck-source.mjs";

export async function slide01(presentation, ctx) {
  return makeSlide(1, presentation, ctx);
}
