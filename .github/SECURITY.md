# Security Policy

## Supported versions

| Version | Supported          |
|---------|--------------------|
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a vulnerability

RecallWeave is a local-first tool: it reads and writes a store directory you
point it at, exports portable memory packs, and renders SVGs from those packs.
It makes no network calls and executes nothing from the memories it stores.

If you find a security-relevant issue - for example a store file that can
crash the engine, a path-traversal via `--dir` or the export pack, or an SVG
that escapes its escaping - please report it privately:

