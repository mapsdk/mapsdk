/*!
# MapSDK

A cross-platform library for rendering maps.

### Example

import in `Cargo.toml`:

```toml
mapsdk = "0.1.0"
```
*/

pub mod common;
pub(crate) mod env;
pub(crate) mod event;
pub mod geo;
pub(crate) mod http;
pub mod layer;
pub mod map;
pub mod render;
pub mod utils;
