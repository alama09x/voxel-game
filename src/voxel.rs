use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash};

pub trait Voxel:
    'static
    + Serialize
    + for<'de> Deserialize<'de>
    + Debug
    + Default
    + Clone
    + Copy
    + PartialEq
    + Eq
    + hash::Hash
    + Send
    + Sync
{
    type Raw;
    fn default_empty() -> Self;
    fn default_opaque() -> Self;
    fn is_opaque(&self) -> bool;

    fn lerp(a: Self, b: Self, t: f64) -> Self;
    fn raw(&self) -> Self::Raw;

    fn all() -> &'static [Self];
}
