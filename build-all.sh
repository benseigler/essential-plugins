#!/bin/bash

. _build-scripts/build.sh
. _build-scripts/packages.sh

TARGET=$1

rm -r target/bundled

for package in "${PACKAGES[@]}"
do
    allFormats $package $TARGET
done

copyToBundles $TARGET
