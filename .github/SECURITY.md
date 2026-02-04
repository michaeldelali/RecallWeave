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

- Email: `michaeldelali@users.noreply.github.com` (via GitHub contact)
- Or open a GitHub security advisory on this repository.

Include the store/pack files and the command line used. We aim to acknowledge
reports within 7 days and ship a fix in the next release.

## What we do not consider vulnerabilities

- Content of memories you chose to weave in. Garbage in, loud error out is the
  intended behavior.
- The integrity chain flagging a tampered store - that is the feature working.
