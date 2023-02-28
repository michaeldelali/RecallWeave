/**
 * Renders a memory-pack as a woven "tapestry" SVG.
 *
 * The metaphor is a loom: each memory kind is a horizontal *warp* band, and each
 * memory is a *weft* thread laid across it. Thread length encodes confidence,
 * colour encodes kind, and a small knot marks memories that carry links. Detected
 * conflicts are drawn as crossed threads between the two memories involved.
 *
 * The output is a standalone, dependency-free SVG string (no external fonts or
 * images) so it renders anywhere and can be committed to a repo.
 */

import type { MemoryPack, Memory, MemoryKind } from "./pack.js";
import { KINDS } from "./pack.js";

const KIND_COLOR: Record<MemoryKind, string> = {
  episodic: "#e07a5f", // terracotta
  semantic: "#3d8361", // moss
  procedural: "#5f7adb", // indigo
  preference: "#d4a017", // saffron
};

const WIDTH = 900;
const MARGIN = 60;
const BAND_HEIGHT = 90;
const HEADER = 120;
const FOOTER = 70;

function escapeXml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** Deterministic pseudo-random in [0,1) from a string seed (FNV-1a based). */
function seededUnit(seed: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < seed.length; i++) {
    h ^= seed.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return (h % 100000) / 100000;
}

