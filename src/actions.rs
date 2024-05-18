
use crate::shared;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use std::collections::HashMap;

use pipewire;
use pipewire::{
    context::Context,
    core::Core,
    node::Node,
    properties::properties,
};

/*

example node props from a FIFO to file:
GlobalObject {
  id: 185, permissions: PermissionFlags(R | W | X | M),
  type_: Node, version: 3,
  props: Some(DictRef {
    flags: Flags(0x0), entries: {
      "object.serial": "26061", "factory.id": "6", "client.id": "451",
      "node.name": "cool_audio", "media.class": "Audio/Sink"} }) }

 */
pub fn create_main_passthrough_node(
    core: &Rc<Core>,
    name: &str,
) -> Node {
    let node: Node = core.create_object(
        "adapter", // TODO: fetch dynamically
        &properties! {
            "node.name" => name,
            "node.description" => "PWACC",
            "factory.name" => "support.null-audio-sink", // TODO: check availability
            "media.class" => "Audio/Sink",
            "object.linger" => "false",
        },
    ).expect("node creation failed");
    return node;
}

pub fn connect_nodes(
    core: &Rc<Core>,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
    node_from: Node,
    node_to: Node,
) {
    // PW needs ports, so gotta map all that out
}
