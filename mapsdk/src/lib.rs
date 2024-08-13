/*!
# MapSDK

A cross-platform library for rendering maps.

### Example

import in `Cargo.toml`:

```toml
mapsdk = "0.1.0"
```
*/

use std::fmt::Debug;

pub mod feature;
pub mod layer;
pub mod map;
pub mod render;
pub mod tiling;
pub mod utils;
pub mod vector_tile;

pub(crate) mod env;
pub(crate) mod event;

pub type JsonValue = serde_json::value::Value;

pub(crate) trait CoordType: Debug + num_traits::Float {
    fn num_max() -> Self;
    fn num_min() -> Self;
    fn to_f32(self) -> f32;
}

impl CoordType for f64 {
    fn num_max() -> Self {
        f64::MAX
    }

    fn num_min() -> Self {
        f64::MIN
    }

    fn to_f32(self) -> f32 {
        self as f32
    }
}

impl CoordType for f32 {
    fn num_max() -> Self {
        f32::MAX
    }

    fn num_min() -> Self {
        f32::MIN
    }

    fn to_f32(self) -> f32 {
        self
    }
}
