
use crate::matching;
use crate::shared;
use crate::constants;

use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::{cell::Cell, rc::Rc};

use adw::glib::clone;
use libspa::utils::dict::DictRef;
use pipewire_sys;
use pipewire::{
    context::Context,
    main_loop::MainLoop,
    node::{Node, NodeInfoRef},
    port::{Port, PortInfoRef},
    link::{Link, LinkInfoRef},
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
    pw_state: &Rc<RefCell<shared::PwGraphState>>,
) -> Result<(), &'static str> {
    let proxy: Node = registry.bind(node).expect("didn't bind?");
    let listener = proxy.add_listener_local()
        .info(clone!(@strong pw_objects, @strong pw_state => move |info| {
            handle_node_info(info, &pw_objects, &pw_state);
        }))
        .register();
    pw_objects.borrow_mut().insert(node.id, shared::ProxyItem::Node {
        proxy: proxy, listener: listener,
    });

    pw_state.borrow_mut().add(node.id, shared::PwGraphItem::Node);

    return Ok(());
}

/*
 * this be called a lot whenever the node info/connections change
 */
fn handle_node_info(
    info: &NodeInfoRef,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
    pw_state: &Rc<RefCell<shared::PwGraphState>>,
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
    pw_state: &Rc<RefCell<shared::PwGraphState>>,
) {
    let id = port.id;
    let props = port.props.expect("port should have props");
    let proxy: Port = registry.bind(port).expect("proxy for port bind failed");
    let listener = proxy.add_listener_local()
        .info(clone!(@strong pw_objects, @strong pw_state => move |info| {
            handle_port_info(info, &pw_objects, &pw_state);
        }))
        .register();
    pw_objects.borrow_mut().insert(port.id, shared::ProxyItem::Port {
        proxy: proxy, listener: listener,
    });

    let mut pw_state = pw_state.borrow_mut();
    // initial props have useful info that we want to have immediately
    let node_id: u32 = props.get("node.id")
        .unwrap_or_else(|| panic!("ERROR! port props should have node.id, have: {:?}", props))
        .parse().expect("node.id should be u32");
    pw_state.add(id, shared::PwGraphItem::Port {
        node_id,
        direction: constants::direction_name_to_spa_direction(
            props.get("port.direction")
                .expect("port needs a direction")
        ),
    });
    // if audio.channel present, set it:
    if let Some(audio_channel) = props.get("audio.channel") {
        let ch = constants::channel_name_to_spa_audio_channel(audio_channel);
        pw_state.set_port_audio_channel(id, ch);
        // println!("initial audio channel {:?} ({:?})", audio_channel, ch);
    }
}

fn handle_port_info(
    info: &PortInfoRef,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
    pw_state: &Rc<RefCell<shared::PwGraphState>>,
) {
    let props = info.props().expect("port info should have props");
    let id = info.id();
    let pw_objects = pw_objects.borrow();
    let mut pw_state = pw_state.borrow_mut();

    // do we know this port?
    let Some(shared::ProxyItem::Port { proxy, .. }) = pw_objects.get(&id) else {
        println!("ERROR! port is unknown! {:?}", info);
        return;
    };

    // was this info an update? / do we already know about it?
    if let Some(shared::PwGraphItem::Port {..}) = pw_state.get(id) {
        // println!("port updated: {:?}", info);
        // TODO in case we need any of this data
    } else {
        let node_id: u32 = props.get("node.id")
            .unwrap_or_else(|| panic!("ERROR! port info should have node.id: {:?}", info))
            .parse().expect("node.id should be u32");

        pw_state.add(id, shared::PwGraphItem::Port {
            node_id,
            direction: info.direction(),
        });

    }

    // if audio.channel present, set it:
    if let Some(audio_channel) = props.get("audio.channel") {
        let ch = constants::channel_name_to_spa_audio_channel(audio_channel);
        pw_state.set_port_audio_channel(id, ch);
        // println!("Parsing audio channel {:?} ({:?})", audio_channel, ch);
    }
}

pub fn handle_link_added(
    link: &GlobalObject<&DictRef>,
    registry: &Rc<Registry>,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
    pw_state: &Rc<RefCell<shared::PwGraphState>>,
) {
    let proxy: Link = registry.bind(link).expect("proxy for link bind failed");
    let listener = proxy.add_listener_local()
        .info(clone!(@strong pw_objects, @strong pw_state => move |info| {
            handle_link_info(info, &pw_objects, &pw_state);
        }))
        .register();
    pw_objects.borrow_mut().insert(link.id, shared::ProxyItem::Link {
        proxy: proxy, listener: listener,
    });
}

fn handle_link_info(
    info: &LinkInfoRef,
    pw_objects: &Rc<RefCell<HashMap<u32, shared::ProxyItem>>>,
    pw_state: &Rc<RefCell<shared::PwGraphState>>,
) {
    let id = info.id();
    let props = info.props().expect("link should have props");

    let port_from = info.output_port_id();
    let port_to = info.input_port_id();

    pw_state.borrow_mut().add(id, shared::PwGraphItem::Link {port_from, port_to});
}
