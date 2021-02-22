# Image Dice

Assumes that tile_index.shp and img.tif are in the same UTM coordinates reference system.

Will segment the img at `<img-path>` into smaller images using `<tile-index>` and outputs the resulting images at `<out-dir>`. 
The filenames of the output images will have the bottom left corner easting and northing in meters appended to the filename.

```
Dice large .tif files using a lidar tile_index.shp file.
USAGE:
    img_dice [OPTIONS] <img-path> <tile-index> <out-dir>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -n, --threads <threads>    The number of threads to use for processing [default: 1]

ARGS:
    <img-path>      Path to GeoTiff to dice up
    <tile-index>    Path to the tile_index.shp
    <out-dir>       Output directory to save the diced images
```
---
Created by: Taylor Denouden (2021)
