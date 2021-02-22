use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
pub struct Opt {
    /// Path to GeoTiff to dice up
    #[structopt(parse(from_os_str))]
    pub img_path: PathBuf,

    /// Path to the tile_index.shp
    #[structopt(parse(from_os_str))]
    pub tile_index: PathBuf,

    /// Output directory to save the diced images
    #[structopt(parse(from_os_str))]
    pub out_dir: PathBuf,
}

impl Opt {
    pub fn get_output_tile_path(&self, ll: &shapefile::PointZ) -> String {
        let img_stub = self.img_path.file_stem().expect("Could not parse img file name stem");
        let ext = self.img_path.extension().expect("Could not parse img extension");

        format!("{outdir}/{img_stub}_{easting}_{northing}.{ext}",
                outdir = self.out_dir.to_str().unwrap(),
                img_stub = img_stub.to_str().unwrap(),
                easting = ll.x.round(),
                northing = ll.y.round(),
                ext = ext.to_str().unwrap())
    }
}