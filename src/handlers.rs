
use crate::matching;
use crate::shared;

use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::{cell::Cell, rc::Rc};

use adw::glib::clone;
use libspa::utils::dict::DictRef;
use pipewire::{
    context::Context,
    main_loop::MainLoop,
    node::{Node, NodeInfoRef},
    port::{Port, PortInfoRef},
    registry::{GlobalObject, Registry},
    types::ObjectType,
};

// pub fn handle_new_object(
//     global: &GlobalObject<&DictRef>,
//     registry: &Rc<Registry>,
//     pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
// ) {
//     println!("Handlin a new PW thingie! {:?}", global);
//
//     // let proxy = registry.bind(global).expect("Failed to bind PW object");
//     // return proxy;
// }

pub fn handle_node_added(
    node: &GlobalObject<&DictRef>,
    registry: &Rc<Registry>,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
) -> Result<(), &'static str> {
    let proxy: Node = registry.bind(node).expect("didn't bind?");
    let listener = proxy.add_listener_local()
        .info(clone!(@strong pw_objects => move |info| {
            handle_node_info(info, &pw_objects);
        }))
        .register();
    pw_objects.borrow_mut().insert(node.id, shared::ProxyItem::Node {
        proxy: proxy, listener: listener,
    });
    Ok(())
}

/*
 * this be called a lot whenever the node info/connections change
 */
fn handle_node_info(
    info: &NodeInfoRef,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
) {
    let props = info.props().expect("NodeInfoRef should have props");
    // println!("== got me some nodeinfos {:?}", info);
    // TODO: the name / somethin in props might change after the initial node was added
    // should re-evaluate matches when that happens
}

pub fn handle_port_added(
    port: &GlobalObject<&DictRef>,
    registry: &Rc<Registry>,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
) {
    let proxy: Port = registry.bind(port).expect("proxy for port bind failed");
    let listener = proxy.add_listener_local()
        .info(clone!(@strong pw_objects => move |info| {
            handle_port_info(info, &pw_objects);
        }))
        .register();
    pw_objects.borrow_mut().insert(port.id, shared::ProxyItem::Port {
        proxy: proxy, listener: listener,
    });
}

fn handle_port_info(
    info: &PortInfoRef,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
) {
    // will I ever need these?
    let props = info.props().expect("port should have props");
}
