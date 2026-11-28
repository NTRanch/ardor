//! ardor library surface: the maker server's modules, exposed
//! so the REQ-9 structural projection (see `projection`) and
//! the gate/store machinery can be consumed and tested as a
//! library. The `ardor` binary is a thin shell over this crate.

pub mod accounts;
pub mod canon;
pub mod chains;
pub mod channels;
pub mod clipverify;
pub mod gate;
pub mod genesis;
pub mod guard;
pub mod model;
pub mod playtest;
pub mod pngcodec;
pub mod projection;
pub mod raster;
pub mod scene_record;
pub mod server;
pub mod sheet;
pub mod store;
pub mod text;
pub mod tools;
