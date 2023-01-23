mod get_nodes;
mod get_nodes_names;

pub use get_nodes::GetNodes;
pub use get_nodes_names::GetNodesNames;

pub trait RingerFunctions {
    const NAME: &'static str;
}
