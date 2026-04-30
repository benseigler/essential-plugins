#!/bin/sh

getFormatFolder() {
    FORMAT=$1
    if [ $FORMAT = "clap" ]; then
        echo "CLAP"
    fi
    if [ $FORMAT = "vst3" ]; then
        echo "VST3"
    fi
}
build() {
    PACKAGE=$1
    TARGET=$2
    FORMAT=$3
    cargo build --package $PACKAGE --release --target $TARGET --no-default-features --features $FORMAT
    cargo xtask bundle-only $PACKAGE --release --target $TARGET --no-default-features --features $FORMAT
}
clap() {
    PACKAGE=$1
    TARGET=$2
    build $PACKAGE $TARGET clap
}
vst3() {
    PACKAGE=$1
    TARGET=$2
    build $PACKAGE $TARGET vst3
}
allFormats() {
    PACKAGE=$1
    TARGET=$2
    clap $PACKAGE $TARGET
    vst3 $PACKAGE $TARGET
}

buildNative() {
    PACKAGE=$1
    FORMAT=$2
    cargo build --package $PACKAGE --release --no-default-features --features $FORMAT
    cargo xtask bundle-only $PACKAGE --release --no-default-features --features $FORMAT
}

copyToBundles() {
    TARGET=$1
    pkgpath="bundles/${TARGET}/"
    mkdir -p "${pkgpath}"
    cp -r target/bundled/* "${pkgpath}"
}
