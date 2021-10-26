use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[serde(from = "SingleOrVecSource<T>")]
pub struct SingleOrVec<T>(pub Vec<T>);

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
enum SingleOrVecSource<T> {
    Single(T),
    Vector(Vec<T>),
}

impl<T> From<SingleOrVecSource<T>> for SingleOrVec<T> {
    fn from(other: SingleOrVecSource<T>) -> SingleOrVec<T> {
        match other {
            SingleOrVecSource::Single(v) => SingleOrVec(vec![v]),
            SingleOrVecSource::Vector(v) => SingleOrVec(v),
        }
    }
}

impl<T> SingleOrVec<T> {
    pub fn from_single(v: T) -> Self {
        Self(vec![v])
    }
}
