# xpans Essential Plugins
Essential Plugins suite for the xpans Ecosystem

## Building
You will need to have rustup and Cargo installed.
On Linux, you may also need to have additional system libraries installed.

### Building one plugin natively:
```sh 
sh build-native.sh <package> <format>
```
`package` is the desired plugin you wish to build from the `plugins/` 
directory. (i.e. `editor`, `headphones`, `exporter`, etc.)

Possible values for `format` include `clap` and `vst3`.

The built plugin will be in `target/bundled/`

### Building all plugins natively:
```sh
sh build-all-native.sh
```
The built plugins will be in `target/bundled/`

### Building all plugins for specific target:
```sh
sh build-all.sh <target>
```
`target` is the target triple (i.e. `x86_64-unknown-linux-gnu`)

The built plugins will be in `bundles/<target>/`


## Requirements
You'll need a DAW that allows for tracks with a high channel count.
[REAPER](https://www.reaper.fm/) allows for 128 channels per track.

## Creating a spatial audio scene
It's recommended to have each audio source be its own track.
Each track should have the Scene Editor plugin in the FX chain, usually
at the very end (unless you are using spatially-aware effects!) These tracks
shouldn't route audio to the main/master track.

You'll also need a scene bus: a track that receives all of your 
audio sources' audio data via audio routing, and spatial data via MIDI routing.
This track shouldn't route audio to the main/master track. 

### Routing audio and spatial data
All audio source tracks should route their audio each to their own separate 
audio channel within the scene bus. MIDI messages should all exist on the same
MIDI channel, but you must route each audio source track's MIDI to the scene 
bus as well.

> Note: Audio and MIDI routing varies across DAWs.
> Refer to your DAW's documentation for more information.

In the Scene Editor plugin of each source, you'll also need to set the Source 
ID to the channel number your audio source occupies in the scene bus.

> Note: At the time of writing, Source IDs count from zero. The first audio
> source in your scene will have a Source ID of 0.

### Monitoring
You'll also need a way to listen to your scene. Create another track with the 
same channel count as your scene bus. Route all audio and MIDI from the scene
bus to this track. Add a monitoring plugin (ex. Headphone Monitor, Stereo 
Monitor, or Mono Monitor) to this track. Make sure that this track is the 
*only* track routing audio to your main/master track. Enjoy!

## Exporting your scene
Add a Scene Exporter plugin onto your scene bus.

Click 'Set Export Path' to choose a destination for your spatial data.

Move the playhead to the start of your scene, and click 'Set Scene Start'.
Do the same for the end of your scene.

When you are ready to export, move the playhead back to the beginning of
the scene, or before it. Then, click the 'Export' button.
The next time the project is played from the  start of your scene to the end, 
the Scene Exporter will write a 
[xpans Spatial Record](https://github.com/xpans-audio/xpans_xsr)
(.xsr) file to the export path you specified. 

No audio data is stored in a .xsr file. It's just spatial data. You will need
to render the scene bus (NOT the master/main track) to an audio file as well.

> Depending on the DAW you are using, spatial data and audio data can be exported
> at the same time. Scene Exporter should export your spatial data during either
> live playback or offline rendering roughly the same.

## Tips and tricks
Scene Editor supports sample-accurate automation. If your DAW is configured
correctly, you can have spatial properties update as frequently as your project
sample rate. Note that this can increase the size of your exported .xsr file
drastically.
