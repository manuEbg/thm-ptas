pub mod dcel;
pub mod dcel_file_writer;
pub mod dual_graph;
pub mod iterators;
pub mod quick_graph;
pub(crate) mod reducible;
pub(crate) mod reductions;
pub mod tree_decomposition;
pub mod builder;


pub use dcel::Dcel;
pub use builder::dcel_builder::DcelBuilder;
