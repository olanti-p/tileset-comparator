use crate::single_or_vec::SingleOrVec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[serde(from = "SpriteIdSource")]
pub struct SpriteIdWithWeight {
    pub id: SingleOrVec<u32>,
    pub weight: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
enum SpriteIdSource {
    IdOnly(SingleOrVec<u32>),
    WithWeight {
        weight: u32,
        sprite: SingleOrVec<u32>,
    },
}

impl From<SpriteIdSource> for SpriteIdWithWeight {
    fn from(other: SpriteIdSource) -> SpriteIdWithWeight {
        match other {
            SpriteIdSource::IdOnly(id) => SpriteIdWithWeight { id, weight: None },
            SpriteIdSource::WithWeight { weight, sprite } => SpriteIdWithWeight {
                weight: Some(weight),
                id: sprite,
            },
        }
    }
}
