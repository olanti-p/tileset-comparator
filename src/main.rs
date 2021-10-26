use serde::{Deserialize, Serialize};
use std::path::Path;

mod single_or_vec;
mod sprite_id_with_weight;

use single_or_vec::SingleOrVec;
use sprite_id_with_weight::SpriteIdWithWeight;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TilesetTileInfo {
    pixelscale: Option<i32>,
    width: i32,
    height: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OverlayOrderElem {
    id: SingleOrVec<String>,
    order: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SingleTile {
    id: SingleOrVec<String>,
    #[serde(default)]
    fg: SingleOrVec<SpriteIdWithWeight>,
    #[serde(default)]
    bg: SingleOrVec<SpriteIdWithWeight>,
    rotates: Option<bool>,
    multitile: Option<bool>,
    animated: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CompositeTile {
    #[serde(flatten)]
    base: SingleTile,
    #[serde(default)]
    additional_tiles: Vec<SingleTile>,
    // Comments
    #[serde(default, rename = "//")]
    _comment: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SingleAscii {
    offset: i32,
    bold: bool,
    color: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TilesNew {
    file: String,
    sprite_width: Option<i32>,
    sprite_height: Option<i32>,
    sprite_offset_x: Option<i32>,
    sprite_offset_y: Option<i32>,
    tiles: Vec<CompositeTile>,
    #[serde(default)]
    ascii: Vec<SingleAscii>,
    // Comments
    #[serde(default, rename = "//")]
    _comment: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Tileset {
    tile_info: Vec<TilesetTileInfo>,
    #[serde(rename = "tiles-new")]
    tiles_new: Vec<TilesNew>,
    #[serde(default)]
    overlay_ordering: Vec<OverlayOrderElem>,
}

fn load_tileset(base_path: &Path) -> Option<Tileset> {
    assert!(base_path.exists());
    assert!(base_path.is_dir());

    let base_tile_config = base_path.join("tile_config.json");

    assert!(base_tile_config.exists());

    let tile_config_data = std::fs::read_to_string(base_tile_config).unwrap();

    let tileset = serde_json::from_str(&tile_config_data).unwrap();

    Some(tileset)
}

fn compare_tilesets(_ts1: &Tileset, _ts2: &Tileset) {}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Expected 2 paths: BN tileset and DDA tileset");
        return;
    }

    let path_bn = &args[1];
    let path_dda = &args[2];

    println!("Loading BN tileset:  {}", path_bn);
    let tiles_bn = load_tileset(Path::new(path_bn));

    println!("Loading DDA tileset: {}", path_dda);
    let tiles_dda = load_tileset(Path::new(path_dda));

    if tiles_bn.is_none() || tiles_dda.is_none() {
        println!("Aborted.");
        return;
    }

    println!("Running comparison...");

    compare_tilesets(tiles_bn.as_ref().unwrap(), tiles_dda.as_ref().unwrap());

    println!("Done!");
}
