# Examples

This directory contains a **runnable** demonstration of recallweave and the
memory-pack it produces.

## Files

| file                | what it is                                                        |
|---------------------|-------------------------------------------------------------------|
| `demo.sh`           | POSIX shell demo driving the full lifecycle with the real binary  |
| `demo.ps1`          | the same demo for PowerShell                                      |
| `memory-pack.json`  | a committed pack exported by the demo (input for the viewer)      |
| `demo-store/`       | the append-only store the demo builds (gitignored; regenerated)   |

## Running the demo

From the **project root**:

```bash
# POSIX shell
sh examples/demo.sh

# PowerShell
pwsh examples/demo.ps1
```

The demo is deterministic — every command pins `--now` — so ids, fingerprints,
and output are stable across runs. It:

1. asserts a `semantic` region fact, then **supersedes** it when the region
   changes (`us-east-1` → `eu-west-1`);
2. records three `preference` memories, two of which are in genuine
   **polarity conflict** (`dark` vs `light` theme);
3. adds a `procedural` runbook step and an `episodic` note with a **1-day TTL**;
