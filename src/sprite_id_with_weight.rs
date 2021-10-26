use crate::single_or_vec::SingleOrVec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[serde(from = "SpriteIdSource")]
pub struct SpriteIdWithWeight {
    pub id: SingleOrVec<i32>,
    pub weight: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
enum SpriteIdSource {
    IdOnly(SingleOrVec<i32>),
    WithWeight {
        weight: i32,
        sprite: SingleOrVec<i32>,
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
