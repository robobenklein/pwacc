
use crate::shared;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use std::collections::HashMap;

use libspa::utils::Direction;
use pipewire;
use pipewire::{
    context::Context,
    core::Core,
    node::Node,
    link::Link,
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
            // TODO specifiy channels???
        },
    ).expect("node creation failed");
    return node;
}

pub fn connect_nodes(
    core: &Rc<Core>,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
    pw_state: &Rc<RefCell<shared::PwGraphState>>,
    node_from: &u32,
    node_to: &u32,
) -> Result<(), ()> {
    // PW needs ports, so gotta map all that out
    let pw_state = pw_state.borrow();

    let node_from_ports = pw_state.ports_for_node(node_from);
    let node_to_ports = pw_state.ports_for_node(node_to);

    println!("connect node {:?} to {:?}", node_from, node_to);

    return Ok(());
}

pub fn connect_ports(
    core: &Rc<Core>,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
    pw_state: &Rc<RefCell<shared::PwGraphState>>,
    port_from_id: u32,
    port_to_id: u32,
) -> Result<Link, ()> {
    let pw_state = pw_state.borrow();
    // let core = core.borrow_mut();

    let shared::PwGraphItem::Port {node_id: node_from_id, direction: port_from_direction} =
        pw_state.get(port_from_id).expect("from port did not exist!")
    else {
        panic!("that was not a port's id!")
    };
    assert_eq!(port_from_direction.clone(), Direction::Output);

    let shared::PwGraphItem::Port {node_id: node_to_id, direction: port_to_direction} =
        pw_state.get(port_to_id).expect("to port did not exist!")
    else {
        panic!("that was not a port's id!")
    };
    assert_eq!(port_to_direction.clone(), Direction::Input);

    assert_eq!(
        port_from_direction.reverse(), port_to_direction.clone(),
        "we cant link these! {:?} and {:?}", port_from_id, port_to_id
    );

    let link_props = &properties! {
        "link.output.node" => node_to_id.to_string().as_str(),
        "link.output.port" => port_to_id.to_string().as_str(),
        "link.input.node" => node_from_id.to_string().as_str(),
        "link.input.port" => port_from_id.to_string().as_str(),
        "object.linger" => "1",
    };
    println!("   making a link! with {:?}", link_props);
    let link_proxy = core.create_object::<Link>(
        "link-factory", // TODO: fetch dynamically
        link_props,
    ).expect("link creation failed!");

    return Ok(link_proxy);
}
