//! ardor library surface: the maker server's modules, exposed
//! so the REQ-9 structural projection (see `projection`) and
//! the gate/store machinery can be consumed and tested as a
//! library. The `ardor` binary is a thin shell over this crate.

pub mod accounts;
pub mod canon;
pub mod chains;
pub mod channels;
pub mod gate;
pub mod genesis;
pub mod guard;
pub mod model;
pub mod playtest;
pub mod portraits;
pub mod projection;
pub mod scene_record;
pub mod server;
pub mod store;
pub mod tools;
