#![feature(slice_partition_dedup)]

mod single_or_vec;
mod sprite_id_with_weight;

use single_or_vec::SingleOrVec;
use sprite_id_with_weight::SpriteIdWithWeight;

use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, ImageFormat, RgbaImage, SubImage};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TilesetTileInfo {
    #[serde(default = "default_pixelscale")]
    pixelscale: f32,
    #[serde(default)]
    iso: bool,
    width: u32,
    height: u32,
}

fn default_pixelscale() -> f32 {
    1.0
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
    sprite_width: Option<u32>,
    sprite_height: Option<u32>,
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

struct TileAtlas<'a> {
    img: &'a RgbaImage,
    sprite_w: u32,
    sprite_h: u32,
    tiles_x: u32,
    tiles_y: u32,
    tiles_start: u32,
    tiles_end: u32,
}

impl<'a> TileAtlas<'a> {
    pub fn tiles_total(&self) -> u32 {
        self.tiles_x * self.tiles_y
    }

    pub fn in_bounds(&self, tile_id: u32) -> bool {
        tile_id >= self.tiles_start && tile_id < self.tiles_end
    }

    fn get_sprite(&self, tile_id: u32) -> SubImage<&RgbaImage> {
        let id_within_atlas = tile_id - self.tiles_start;
        let within_x = id_within_atlas % self.tiles_x;
        let within_y = id_within_atlas / self.tiles_x;
        self.img.view(
            within_x * self.sprite_w,
            within_y * self.sprite_h,
            self.sprite_w,
            self.sprite_h,
        )
    }

    pub fn get_sprite_hash(&self, tile_id: u32) -> u32 {
        if !self.in_bounds(tile_id) {
            eprintln!(
                "WARNING: tile {} outside active atlas range {}..{}",
                tile_id, self.tiles_start, self.tiles_end
            );
            return 0;
        }

        let subimg = self.get_sprite(tile_id);

        let mut hasher = DefaultHasher::new();
        self.sprite_w.hash(&mut hasher);
        self.sprite_h.hash(&mut hasher);

        for px in subimg.pixels() {
            px.hash(&mut hasher);
        }

        // Intended narrowing conversion
        hasher.finish() as u32
    }

    pub fn dump_sprites_to_dir(&self, base_path: &Path) {
        for tile_id in self.tiles_start..self.tiles_end {
            let sprite_path = base_path.join(format!("{}.png", tile_id));
            let subimg = self.get_sprite(tile_id);
            subimg
                .to_image()
                .save_with_format(&sprite_path, ImageFormat::Png)
                .unwrap();
        }
    }
}

fn hash_sprites(ids: &mut SingleOrVec<SpriteIdWithWeight>, atlas: &TileAtlas) {
    for spidw in &mut ids.0 {
        for id in &mut spidw.id.0 {
            *id = atlas.get_sprite_hash(*id);
        }
    }
}

impl Tileset {
    pub fn generate_variations(&self) -> Vec<SingleTile> {
        let mut ret = Vec::with_capacity(self.tiles_new.len());

        let sprites_path = self.base_path.join("sprites");
        let _ = std::fs::remove_dir_all(&sprites_path);
        std::fs::create_dir(&sprites_path).unwrap();

        let mut tiles_start: u32 = 0;

        for tiles_new in &self.tiles_new {
            let img_path = self.base_path.join(&tiles_new.file);
            let img_raw: DynamicImage = ImageReader::open(&img_path).unwrap().decode().unwrap();
            let img: RgbaImage = img_raw.to_rgba8();
            let sprite_w = tiles_new.sprite_width.unwrap_or(self.tile_info[0].width);
            let sprite_h = tiles_new.sprite_height.unwrap_or(self.tile_info[0].height);

            if img.width() % sprite_w != 0 || img.height() % sprite_h != 0 {
                eprint!(
                    "WARNING: image '{}' cannot be properly divided into sprites of size {}x{}",
                    img_path.to_string_lossy(),
                    sprite_w,
                    sprite_h
                );
            }

            let mut atlas = TileAtlas {
                img: &img,
                sprite_w,
                sprite_h,
                tiles_x: img.width() / sprite_w,
                tiles_y: img.height() / sprite_h,
                tiles_start,
                tiles_end: tiles_start,
            };
            atlas.tiles_end = atlas.tiles_start + atlas.tiles_total();
            atlas.dump_sprites_to_dir(&sprites_path);

            tiles_start = atlas.tiles_end;

            for tile in &tiles_new.tiles {
                for id in &tile.base.id.0 {
                    let mut cloned = tile.base.clone();
                    cloned.id = SingleOrVec::from_single(id.to_owned());
                    hash_sprites(&mut cloned.fg, &atlas);
                    hash_sprites(&mut cloned.bg, &atlas);
                    if cloned.rotates.is_none() {
                        cloned.rotates = Some(cloned.multitile);
                    }

                    for at in &tile.additional_tiles {
                        for at_id in &at.id.0 {
                            let mut cloned_at = at.clone();
                            cloned_at.id = SingleOrVec::from_single(id.to_owned() + "_" + at_id);
                            hash_sprites(&mut cloned_at.fg, &atlas);
                            hash_sprites(&mut cloned_at.bg, &atlas);
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

fn dump_variations(vars: &Vec<SingleTile>, ts: &Tileset) {
    let dump = serde_json::to_string_pretty(&vars).unwrap();
    std::fs::write(ts.base_path.join("dump.json"), dump).unwrap();
}

fn find_duplicates(vars: &Vec<SingleTile>) -> Vec<&str> {
    let mut ids: Vec<&str> = vars.iter().map(|x| x.id.0[0].as_str()).collect();
    ids.sort_unstable();
    let (_, dups) = ids.partition_dedup();
    dups.to_vec()
}

fn dump_duplicates(dups: &Vec<&str>, ts: &Tileset) {
    let dump = dups.join("\n");
    std::fs::write(ts.base_path.join("duplicates.txt"), dump).unwrap();
}

fn dump_exclusives(exc: &HashSet<&str>, ts: &Tileset) {
    let mut elems: Vec<&str> = exc.iter().cloned().collect();
    elems.sort();
    let dump = elems.join("\n");
    std::fs::write(ts.base_path.join("exclusives.txt"), dump).unwrap();
}

fn dump_diffs(elems: &HashSet<&SingleTile>, ts: &Tileset) {
    let mut elems: Vec<&str> = elems.iter().map(|x| x.id.0[0].as_str()).collect();
    elems.sort();
    let dump = elems.join("\n");
    std::fs::write(ts.base_path.join("different.txt"), dump).unwrap();
}

fn compare_tilesets(ts1: &Tileset, ts2: &Tileset) {
    let vars1 = ts1.generate_variations();
    let vars2 = ts2.generate_variations();

    {
        dump_variations(&vars1, ts1);
        dump_variations(&vars2, ts2);
    }

    let do_diff: bool = {
        let dups1 = find_duplicates(&vars1);
        let dups2 = find_duplicates(&vars2);
        dump_duplicates(&dups1, ts1);
        dump_duplicates(&dups2, ts2);
        dups1.is_empty() && dups2.is_empty()
    };

    let ids_1: HashSet<&str> = vars1.iter().map(|x| x.id.0[0].as_str()).collect();
    let ids_2: HashSet<&str> = vars2.iter().map(|x| x.id.0[0].as_str()).collect();

    {
        let in_1_only: HashSet<&str> = ids_1.difference(&ids_2).cloned().collect();
        let in_2_only: HashSet<&str> = ids_2.difference(&ids_1).cloned().collect();

        dump_exclusives(&in_1_only, ts1);
        dump_exclusives(&in_2_only, ts2);
    }
    if do_diff {
        let idx1: HashSet<&SingleTile> = vars1.iter().collect();
        let idx2: HashSet<&SingleTile> = vars2.iter().collect();

        let in_1_only: HashSet<&SingleTile> = idx1
            .difference(&idx2)
            .cloned()
            .filter(|x| ids_2.contains(x.id.0[0].as_str()))
            .collect();
        let in_2_only: HashSet<&SingleTile> = idx2
            .difference(&idx1)
            .cloned()
            .filter(|x| ids_1.contains(x.id.0[0].as_str()))
            .collect();

        dump_diffs(&in_1_only, ts1);
        dump_diffs(&in_2_only, ts2);
    } else {
        eprintln!(
            "WARNING: duplicate tiles found in at least one tileset, diff will not be generated."
        );
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
