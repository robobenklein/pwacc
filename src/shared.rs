
use libspa_sys::spa_audio_channel;
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
        direction: libspa::utils::Direction,
    },
    Link {
        port_from: u32,
        port_to: u32,
    },
}

#[derive(Default)]
pub struct PwGraphState {
    items: HashMap<u32, PwGraphItem>,
    links: HashMap<(u32, u32), u32>,
    port_audio_channels: HashMap<u32, spa_audio_channel>,
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

    pub fn set_port_audio_channel(&mut self, id: u32, ch: spa_audio_channel) {
        self.port_audio_channels.insert(id, ch);
    }

    pub fn get_port_audio_channel(&self, id: u32) -> Option<&spa_audio_channel> {
        return self.port_audio_channels.get(&id);
    }

    pub fn get(&self, id: u32) -> Option<&PwGraphItem> {
        return self.items.get(&id);
    }

    pub fn del(&mut self, id: u32) -> Option<PwGraphItem> {
        let old = self.items.remove(&id);

        if let Some(PwGraphItem::Link {port_from, port_to}) = old {
            self.links.remove(&(port_from, port_to));
        }
        if let Some(PwGraphItem::Port {..}) = old {
            self.port_audio_channels.remove(&id);
        }

        return old;
    }

    // conveniences
    pub fn ports_for_node(&self, node: &u32) -> Vec<&u32> {
        let mut matched_ports: Vec<&u32> = vec![];
        for (k, v) in self.items.iter() {
            if let PwGraphItem::Port {node_id, ..} = v {
                if node_id == node {
                    matched_ports.push(&k);
                }
            }
        }
        return matched_ports;
    }
}
