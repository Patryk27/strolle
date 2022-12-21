//! This file is not an example - it just contains common code used by some of
//! the examples.

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use zip::ZipArchive;

pub fn unzip_assets() {
    unzip_asset("buddha");
    unzip_asset("bunny");
    unzip_asset("dragon");
    unzip_asset("nefertiti");
}

fn unzip_asset(name: &str) {
    let dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Please use `cargo` to run the examples");

    let dir = Path::new(&dir).join("assets");

    if dir.join(name).with_extension("obj").exists() {
        return;
    }

    let archive = dir.join(name).with_extension("zip");

    let archive = File::open(archive)
        .unwrap_or_else(|err| panic!("Couldn't open asset {}: {}", name, err));

    let mut archive =
        ZipArchive::new(BufReader::new(archive)).unwrap_or_else(|err| {
            panic!("Couldn't open archive for asset: {}: {}", name, err)
        });

    archive.extract(&dir).unwrap_or_else(|err| {
        panic!("Couldn't extract asset {}: {}", name, err)
    });
}

#[allow(dead_code)]
fn main() {}
