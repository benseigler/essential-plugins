#!/bin/sh

. _build-scripts/build.sh
. _build-scripts/packages.sh

for package in "${PACKAGES[@]}"
do
    buildNative $package clap
    buildNative $package vst3
done
