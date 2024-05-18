
use std::collections::HashMap;

use pipewire::{
    node::{Node, NodeListener},
    port::{Port, PortListener},
    link::{Link, LinkListener},
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
    Link {
        proxy: Link,
        listener: LinkListener,
    }
}

pub enum PwGraphItem {
    Node,
    Port {
        node_id: u32,
    },
    Link {
        port_from: u32,
        port_to: u32
    },
}

#[derive(Default)]
pub struct PwGraphState {
    items: HashMap<u32, PwGraphItem>,
    links: HashMap<(u32, u32), u32>,
}

impl PwGraphState {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn add(&mut self, id: u32, item: PwGraphItem) {
        if let PwGraphItem::Link {port_from, port_to} = item {
            self.links.insert((port_from, port_to), id);
        }
        self.items.insert(id, item); // moves ownership?
    }

    pub fn del(&mut self, id: u32) -> Option<PwGraphItem> {
        let old = self.items.remove(&id);

        if let Some(PwGraphItem::Link {port_from, port_to}) = old {
            self.links.remove(&(port_from, port_to));
        }

        return old;
    }
}
