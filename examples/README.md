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

