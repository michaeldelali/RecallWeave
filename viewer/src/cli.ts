#!/usr/bin/env node
/**
 * `tapestry` — the recallweave memory-pack viewer.
 *
 * Usage:
 *   tapestry <pack.json>              print a text report
 *   tapestry <pack.json> --svg out.svg   also write a woven SVG tapestry
 *   tapestry <pack.json> --svg -     write the SVG to stdout
 *
 * The viewer never talks to the Rust engine or the log directly; it consumes the
 * portable pack only. That decoupling is deliberate (see docs/MEMORY.md).
 */

import { readFileSync, writeFileSync } from "node:fs";
import { parsePackText, PackError } from "./pack.js";
import { renderReport } from "./report.js";
import { renderTapestry } from "./svg.js";

interface Args {
  packPath: string | null;
  svgOut: string | null;
  help: boolean;
}

function parseArgs(argv: string[]): Args {
  const args: Args = { packPath: null, svgOut: null, help: false };
  for (let i = 0; i < argv.length; i++) {
