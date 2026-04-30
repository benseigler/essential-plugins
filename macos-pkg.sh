#!/bin/bash

ARCH=$1
VERSION=$2
BUNDLES_PATH=$3
CERTIFICATE_ID=$4

id_prefix="audio.xpans.EssentialPlugins"

names=(
  "HeadphoneMonitor"
  "StereoMonitor"
  "MonoMonitor"
  "SceneEditor"
  "SceneExporter"
)
file=(
  "xpans Headphones Monitor"
  "xpans Stereo Monitor"
  "xpans Mono Monitor"
  "xpans Scene Editor"
  "xpans Scene Exporter"
)

format_installs=(
    "CLAP"
    "VST3"
)

format_extensions=(
    "clap"
    "vst3"
)

for f in "${!format_installs[@]}"; do
    for i in "${!names[@]}"; do
        pkgbuild --identifier "${id_prefix}.${names[$i]}.${format_extensions[f]}.pkg" \
            --version "${VERSION}" \
            --component "${BUNDLES_PATH}/${file[$i]}.${format_extensions[f]}" \
            --install-location "/Library/Audio/Plug-Ins/${format_installs[f]}" \
            "packaging/${id_prefix}.${names[$i]}.${format_extensions[f]}.pkg"
    done
done
cd packaging
productbuild --distribution distribution.xml --sign "$CERTIFICATE_ID" --timestamp "Install xpans Essential Plugins (${ARCH}).pkg"
