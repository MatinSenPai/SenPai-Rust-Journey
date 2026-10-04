//! The file layer on its own: `Layer::from_toml` turns TOML text into a layer
//! in which every setting is optional, and refuses a mistyped key.
//!
//!     cargo run -p p3-04-01-config-and-secrets --example 02-file-layer

use p3_04_01_config_and_secrets::Layer;

fn main() {
    let good = "port = 7000\nallowed_origins = [\"https://anime.example.com\"]\n";
    println!("{:?}", Layer::from_toml(good));

    let typo = "prot = 7000\n";
    println!("{:?}", Layer::from_toml(typo));

    let wrong_type = "port = \"seven thousand\"\n";
    println!("{:?}", Layer::from_toml(wrong_type));
}
