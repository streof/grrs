#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// #![warn(clippy::pedantic)]
// #![warn(missing_debug_implementations)]
// #![warn(missing_docs)]
pub mod base;
pub mod cli;
pub mod ends_with;
pub mod ext;
pub mod gen_check;
pub mod matcher;
pub mod max_count;
pub mod results;
pub mod searcher;
pub mod starts_ends_with;
pub mod starts_with;
pub mod words;
pub mod writer;
