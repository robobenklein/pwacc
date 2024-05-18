
use pipewire::{
    node::{Node, NodeListener},
    port::{Port, PortListener},
    types::ObjectType,
};

pub enum ProxyItem {
    Node {
        proxy: Node,
        listener: NodeListener,
    },
    Port {
        proxy: Port,
        listener: PortListener,
    },
}
