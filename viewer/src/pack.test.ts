/**
 * Tests for the tapestry viewer, using Node's built-in assert (no test runner
 * dependency). Run with `npm test`, which compiles then executes this file.
 * The script exits non-zero if any assertion fails.
 */

import assert from "node:assert/strict";
import { parsePackText, PackError, type MemoryPack } from "./pack.js";
import { renderReport } from "./report.js";
import { renderTapestry } from "./svg.js";

let passed = 0;
function test(name: string, fn: () => void): void {
  try {
    fn();
