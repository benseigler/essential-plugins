#!/bin/bash

. _build-scripts/container.sh

buildContainer aarch64-unknown-linux-gnu
container aarch64-unknown-linux-gnu xpans-build/essential-plugins
