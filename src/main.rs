use structopt::StructOpt;

use img_dice::{Opt, run};

fn main() {
    // Get cmdline args
    let opt = Opt::from_args();
    run(opt);
}
