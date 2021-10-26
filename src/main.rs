use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod single_or_vec;
mod sprite_id_with_weight;

use single_or_vec::SingleOrVec;
use sprite_id_with_weight::SpriteIdWithWeight;
use std::collections::HashSet;

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

#[derive(Clone, Debug, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
struct SingleTile {
    id: SingleOrVec<String>,
    #[serde(default)]
    fg: SingleOrVec<SpriteIdWithWeight>,
    #[serde(default)]
    bg: SingleOrVec<SpriteIdWithWeight>,
    rotates: Option<bool>,
    #[serde(default)]
    multitile: bool,
    #[serde(default)]
    animated: bool,
    #[serde(default)]
    height_3d: i32,
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
    #[serde(skip_deserializing)]
    base_path: PathBuf,
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

    let mut tileset: Tileset = serde_json::from_str(&tile_config_data).unwrap();
    tileset.base_path = base_path.to_owned();

    Some(tileset)
}

// TODO: get rid of this and use sprite hashes
fn cull_sprites(ids: &mut SingleOrVec<SpriteIdWithWeight>) {
    for spidw in &mut ids.0 {
        for id in &mut spidw.id.0 {
            *id = 1337;
        }
    }
}

impl Tileset {
    pub fn generate_variations(&self) -> Vec<SingleTile> {
        let mut ret = Vec::with_capacity(self.tiles_new.len());

        for tiles_new in &self.tiles_new {
            for tile in &tiles_new.tiles {
                for id in &tile.base.id.0 {
                    let mut cloned = tile.base.clone();
                    cloned.id = SingleOrVec::from_single(id.to_owned());
                    cull_sprites(&mut cloned.fg);
                    cull_sprites(&mut cloned.bg);
                    if cloned.rotates.is_none() {
                        cloned.rotates = Some(cloned.multitile);
                    }

                    for at in &tile.additional_tiles {
                        for at_id in &at.id.0 {
                            let mut cloned_at = at.clone();
                            cloned_at.id = SingleOrVec::from_single(id.to_owned() + "_" + at_id);
                            cull_sprites(&mut cloned_at.fg);
                            cull_sprites(&mut cloned_at.bg);
                            cloned_at.rotates = Some(true);
                            cloned_at.height_3d = cloned.height_3d;
                            ret.push(cloned_at);
                        }
                    }

                    ret.push(cloned);
                }
            }
        }

        ret.sort();
        ret
    }
}

fn dump_variations(vars: &Vec<SingleTile>, ts: &Tileset, fname: &str) {
    let dump = serde_json::to_string_pretty(&vars).unwrap();
    std::fs::write(ts.base_path.join(fname), dump).unwrap();
}

fn dump_exclusives(exc: &HashSet<&str>, ts: &Tileset) {
    let mut elems: Vec<&str> = exc.iter().cloned().collect();
    elems.sort();
    let dump = elems.join("\n");
    std::fs::write(ts.base_path.join("exclusives.txt"), dump).unwrap();
}

fn compare_tilesets(ts1: &Tileset, ts2: &Tileset) {
    let vars1 = ts1.generate_variations();
    let vars2 = ts2.generate_variations();

    {
        dump_variations(&vars1, ts1, "dump.json");
        dump_variations(&vars2, ts2, "dump.json");
    }
    {
        let idx1: HashSet<&str> = vars1.iter().map(|x| x.id.0[0].as_str()).collect();
        let idx2: HashSet<&str> = vars2.iter().map(|x| x.id.0[0].as_str()).collect();

        let in_1_only: HashSet<&str> = idx1.difference(&idx2).cloned().collect();
        let in_2_only: HashSet<&str> = idx2.difference(&idx1).cloned().collect();

        dump_exclusives(&in_1_only, ts1);
        dump_exclusives(&in_2_only, ts2);
    }
    {
        let idx1: HashSet<&SingleTile> = vars1.iter().collect();
        let idx2: HashSet<&SingleTile> = vars2.iter().collect();

        let in_1_only: Vec<SingleTile> = idx1.difference(&idx2).cloned().cloned().collect();
        let in_2_only: Vec<SingleTile> = idx2.difference(&idx1).cloned().cloned().collect();

        dump_variations(&in_1_only, ts1, "diff.json");
        dump_variations(&in_2_only, ts2, "diff.json");
    }
}

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
