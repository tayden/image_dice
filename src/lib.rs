use std::cmp::{max, min};
use std::path::PathBuf;
use std::sync::Arc;

use gdal;
use shapefile;
use structopt::StructOpt;
use threadpool::ThreadPool;

#[derive(StructOpt, Debug)]
#[structopt(name = "image_dice", version = "0.1.0", author = "Taylor Denouden", about = "Dice large .tif files using a lidar tile_index.shp file.")]
pub struct Opt {
    /// Path to GeoTiff to dice up
    #[structopt(parse(from_os_str))]
    img_path: PathBuf,

    /// Path to the tile_index.shp
    #[structopt(parse(from_os_str))]
    tile_index: PathBuf,

    /// Output directory to save the diced images
    #[structopt(parse(from_os_str))]
    out_dir: PathBuf,

    /// The number of threads to use for processing
    #[structopt(short = "n", long, default_value = "1")]
    threads: usize,
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

pub fn run(config: Opt) {
    // Open tile index with shapefile
    let tile_reader = shapefile::Reader::from_path(&config.tile_index)
        .unwrap();

    let pool = ThreadPool::new(config.threads.clone());
    let config = Arc::new(config);
    tile_reader.iter_shapes_as::<shapefile::PolygonZ>().for_each(|shape| {
        let config = Arc::clone(&config);
        match shape {
            Ok(shape) => {
                pool.execute(move || { crop_to_shape(shape, config); });
                // crop_to_shape(shape, config);
            }
            Err(err) => panic!("Unexpected shape type: {}", err)
        }
    });

    pool.join();
}

fn coord2idx(transform: &gdal::GeoTransform, x_coord: &f64, y_coord: &f64) -> (i32, i32) {
    let [x_origin, pixel_width, _, y_origin, _, pixel_height] = transform;

    let x_pos = ((x_coord - x_origin) / pixel_width).round() as i32;
    let y_pos = ((y_coord - y_origin) / pixel_height).round() as i32;

    (x_pos, y_pos)
}

fn crop_to_shape(shape: shapefile::PolygonZ, config: Arc<Opt>) {
    let img = gdal::Dataset::open(&config.img_path).unwrap();

    // Get img shape
    let (img_x_max, img_y_max) = img.raster_size();

    // Get the area to crop
    let ll_point = &shape.bbox().min;
    let ur_point = &shape.bbox().max;
    let ll = coord2idx(&img.geo_transform().unwrap(), &ll_point.x, &ll_point.y);
    let ur = coord2idx(&img.geo_transform().unwrap(), &ur_point.x, &ur_point.y);

    let window: (i32, i32) = (max(ll.0, 0), max(ur.1, 0));
    let win_x_size = min(img_x_max as i32, ur.0) - window.0;
    let win_y_size = min(img_y_max as i32, ll.1) - window.1;
    if win_x_size <= 0 || win_y_size <= 0 { return; }

    let num_bands = img.raster_count();
    let window_size = (win_x_size as usize, win_y_size as usize);
    let window = (window.0 as isize, window.1 as isize);

    // Create a copy of the raster with no data
    let output_path = config.get_output_tile_path(&shape.bbox().min);

    // Create output dataset
    let out_dataset = img.driver().create(&output_path, win_x_size as isize, win_y_size as isize, num_bands).unwrap();

    // Setup spatial reference and transforms of output raster
    let img_projection = &img.projection();
    out_dataset.set_projection(img_projection).unwrap();

    let mut win_geo_transform = img.geo_transform().unwrap().clone();
    win_geo_transform[0] = ll_point.x;
    win_geo_transform[3] = ur_point.y;
    out_dataset.set_geo_transform(&win_geo_transform).unwrap();

    let img_spatial_ref = &img.spatial_ref().unwrap();
    out_dataset.set_spatial_ref(img_spatial_ref).unwrap();

    // use read_into_slice on each band and save it to the new raster file
    for band_i in 1..num_bands + 1 {
        let band = img.rasterband(band_i)
            .expect("Could not read band");
        let out_band = out_dataset.rasterband(band_i)
            .expect("Could not read band");

        let buf = band.read_as::<f64>(window, window_size, window_size)
            .expect("Error buffering band data");

        out_band.write((0, 0), window_size, &buf)
            .expect("Error writing band data");
    }

    println!("Created {}", output_path);
}

// TODO: Stop band 4 from being alpha channel
// TODO: Multi-threading
