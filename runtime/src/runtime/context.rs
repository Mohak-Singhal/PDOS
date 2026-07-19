use crate::{
    discovery::Discovery,
    liveness::Liveness,
    registry::Registry,
};

pub struct RuntimeContext<'a> {
    pub discovery: &'a mut Discovery,
    pub liveness: &'a Liveness,
    pub registry: &'a mut Registry,
    pub local_node_id: &'a str,
}