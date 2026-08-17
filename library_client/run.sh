#!/bin/sh
set -e

# Example manual invocation. Swap the tool/path for whatever you're testing.
# LD_PRELOAD is the mechanism the handout's whole design depends on: it
# forces the dynamic loader to resolve open/read/write/etc. against your
# mylib.so first, ahead of the real libc.
exec env LD_PRELOAD=/work/zig-client/zig-out/mylib.so \
    /work/tools/440cat "$@"
