# recallweave — build, test, and demo automation.
#
# Targets:
#   make build        release build of the Rust engine
#   make test         Rust tests + viewer tests
#   make viewer       install deps and build the TypeScript viewer
#   make demo         run the demo script and regenerate the tapestry SVG
#   make verify       build + test + clippy + fmt-check + viewer build/test
#   make fmt          format Rust sources
#   make clean        remove build artifacts and the demo store

CARGO ?= cargo
NPM   ?= npm
NODE  ?= node

.PHONY: all build test viewer viewer-build viewer-test demo verify fmt fmt-check clippy clean

