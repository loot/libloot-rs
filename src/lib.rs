mod archive;
mod database;
pub mod error;
mod game;
mod logging;
pub mod metadata;
mod plugin;
mod sorting;
#[cfg(test)]
mod tests;
mod version;

use pcre2::{
    Error as RegexImplError,
    bytes::{Regex, RegexBuilder},
};

pub use database::{Database, WriteMode};
pub use game::{Game, GameType};
pub use logging::{LogLevel, set_log_level, set_logging_callback};
pub use plugin::Plugin;
pub use sorting::vertex::{EdgeType, Vertex};
pub use version::{
    LIBLOOT_VERSION_MAJOR, LIBLOOT_VERSION_MINOR, LIBLOOT_VERSION_PATCH, is_compatible,
    libloot_revision, libloot_version,
};

fn case_insensitive_regex(value: &str) -> Result<Regex, Box<RegexImplError>> {
    RegexBuilder::new()
        .caseless(true)
        .utf(true)
        .ucp(true)
        .build(value)
        .map_err(Into::into)
}
