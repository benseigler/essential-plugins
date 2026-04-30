#!/bin/bash

. _build-scripts/container.sh

buildContainer x86_64-unknown-linux-gnu
container x86_64-unknown-linux-gnu xpans-build/essential-plugins
