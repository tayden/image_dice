use img_dice::config;
use structopt::StructOpt;

fn main() {
    // Get cmdline args
    let opt = config::Opt::from_args();
    img_dice::run(opt);
}
