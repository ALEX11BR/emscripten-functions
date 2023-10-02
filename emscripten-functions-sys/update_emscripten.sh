#!/usr/bin/env sh

# Set the `DOCKER` variable when running this script to specify a path to your docker.
# Otherwise `docker`, then `podman` will be tried.

if [ -z "$DOCKER" ]; then
    DOCKER=$(which docker)
fi
if [ -z "$DOCKER" ]; then
    DOCKER=$(which podman)
fi
if [ -z "$DOCKER" ]; then
    echo "ERROR: Couldn't find docker."
    exit 1
fi

ID=$("$DOCKER" create docker.io/emscripten/emsdk)
"$DOCKER" cp "$ID:/emsdk/upstream/emscripten/cache/sysroot/include/." emscripten/cache/sysroot/include/
"$DOCKER" rm "$ID"
