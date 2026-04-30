#!/bin/sh

buildContainer() {
    TARGET=$1
    podman build \
    -f _build-scripts/Containerfile.$TARGET \
    -t xpans-build/essential-plugins/$TARGET
}

container() {
    TARGET=$1
    IMAGE_PREFIX=$2
    podman run --rm \
    --network=host \
    -v $PWD:/usr/src/xpans_plugin_suite:rw,Z \
    -w /usr/src/xpans_plugin_suite \
    $IMAGE_PREFIX/$TARGET \
    ./build-all.sh $TARGET
}
