# Changelog

All notable changes to RecallWeave are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- (planned) remote mirrors: push a store to object storage with the chain intact.
- (planned) query DSL: structured queries beyond the current filters.

## [1.0.0] - 2026-08-11

### Added

- Stable 1.0 store format, pinned and documented in `docs/MEMORY.md`.
- Release profile tuned: LTO + stripped binaries.

### Changed

- Viewer bumped to match the 1.0 pack schema.

## [0.8.0] - 2025-11-21

### Added

- Tapestry viewer: loom-woven SVG and text report rendered from an exported
  `memory-pack.json`; SVG output is deterministic and XML-escaped.

## [0.7.0] - 2024-11-19

### Added

- Portable export: `export` writes a checksummed memory pack readable on any
  machine; re-import preserves the chain.
- `verify` gains pack verification alongside store verification.

## [0.6.0] - 2023-09-07

### Added

- Forgetting: time-based and manual forgetting with an audit trail.
- Compaction that rewrites the tail while preserving the integrity chain.

## [0.5.0] - 2022-10-16

### Added

- Conflict detection: crossed threads spotted at weave time, with both
  conflicting records named in the output.

## [0.4.0] - 2021-12-03

### Added

