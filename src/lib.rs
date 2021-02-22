use std::cmp::{max, min};

use gdal;
use shapefile;

pub mod config;


pub fn run(config: config::Opt) {
    // Open image with GDAL
    let img = gdal::Dataset::open(&config.img_path).unwrap();

    // Open tile index with shapefile
    let tile_reader = shapefile::Reader::from_path(&config.tile_index)
        .unwrap();

    tile_reader.iter_shapes_as::<shapefile::PolygonZ>().for_each(|shape| {
        let shape = shape.expect("Unexpected shape type");
        let output_path = config.get_output_tile_path(&shape.bbox().min);

        crop_to_shape(&img, shape, &*output_path);
    });
}

fn coord2idx(transform: &gdal::GeoTransform, x_coord: &f64, y_coord: &f64) -> (i32, i32) {
    let [x_origin, pixel_width, _, y_origin, _, pixel_height] = transform;

    let x_pos = ((x_coord - x_origin) / pixel_width).round() as i32;
    let y_pos = ((y_coord - y_origin) / pixel_height).round() as i32;

    (x_pos, y_pos)
}

// struct ImageWindow {
//     ll: (i32, i32),
//     ur: (i32, i32),
//     img_dims: (i32, i32)
// }
//
// impl ImageWindow {
//     fn window(&self) {}
//     fn window_size(&self) {}
// }

fn crop_to_shape(img: &gdal::Dataset, shape: shapefile::PolygonZ, output_path: &str) {
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

        let img_no_data = band.no_data_value().unwrap();
        out_band.set_no_data_value(img_no_data).unwrap();

        let buf = band.read_as::<f64>(window, window_size, window_size)
            .expect("Error buffering band data");

        out_band.write((0, 0), window_size, &buf)
            .expect("Error writing band data");
    }

    println!("Created {}", output_path);
}

// TODO: Stop band 4 from being alpha channel
// TODO: Check for matching SRS
// TODO: Separate cli logic to new module
