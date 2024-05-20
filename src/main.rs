mod actions;
mod handlers;
mod matching;
mod shared;
mod constants;
mod helpers;

use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::{cell::Cell, rc::Rc};

use adw::glib::{self, clone};
use clap::{Parser, Subcommand};
use libspa::utils::{dict::DictRef, Direction};
use libspa_sys::{spa_audio_channel, spa_direction};
use once_cell::unsync::OnceCell;
use pipewire::{
  context::Context,
  main_loop::MainLoop,
  node::{Node, NodeInfoRef},
  port::{Port, PortInfoRef},
  proxy::ProxyT,
  registry::GlobalObject,
  types::ObjectType,
};
use regex::Regex;

#[derive(Parser)]
#[command(name = "AutoConnectController")]
#[command(version, about, long_about = None)]
struct Cli {
  /// Change the user-visible node description.
  #[arg(short, long, value_name = "NODE_NAME")]
  name: Option<String>,

  /// Turn debugging information on
  #[arg(short, long, action = clap::ArgAction::Count)]
  verbose: u8,

  /// Include node.description when matching against patterns
  /// (instead of just the application name)
  #[arg(short, long)]
  match_description: bool,

  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
  Inputs {
    #[arg(num_args = 1..)]
    input_patterns: Vec<String>,
  },
  Outputs {
    #[arg(num_args = 1..)]
    output_patterns: Vec<String>,
  },
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();

  pipewire::init();
  let mainloop = MainLoop::new(None)?;
  let context = Context::new(&mainloop)?;
  let core = Rc::new(context.connect(None).expect("failed to connect"));
  let registry = Rc::new(core.get_registry().expect("No registry?"));

  let orig_pw_objects: HashMap<u32, shared::ProxyItem> = HashMap::new();
  let pw_objects = Rc::new(RefCell::new(orig_pw_objects));
  let pw_state = Rc::new(RefCell::new(shared::PwGraphState::new()));

  //let re_inputs: Rc<OnceCell<Vec<Regex>>> = Rc::new(OnceCell::new());
  //let re_inputs_clone = re_inputs.clone();
  let re_inputs: OnceCell<Vec<Regex>> = OnceCell::new();
  let re_outputs: OnceCell<Vec<Regex>> = OnceCell::new();

  let central_node_id: Rc<OnceCell<u32>> = Rc::new(OnceCell::new());
  let central_node_ports: Rc<RefCell<HashMap<(spa_direction, spa_audio_channel), u32>>> = Rc::new(RefCell::new(HashMap::new()));
  let central_node_connect_input_nodes: Rc<RefCell<HashSet<u32>>> = Rc::new(RefCell::new(HashSet::new()));
  let central_node_connect_output_nodes: Rc<RefCell<HashSet<u32>>> = Rc::new(RefCell::new(HashSet::new()));

  // helpers::do_pw_roundtrip(&mainloop, &core);

  // let node_factory_type = ObjectType::Node.to_str();
  // let node_factory: OnceCell<String> = OnceCell::new();
  // //let node_factory_clone = node_factory.clone();
  //
  // let link_factory_type = ObjectType::Link.to_str();
  // let link_factory: OnceCell<String> = OnceCell::new();
  // //let link_factory_clone = link_factory.clone();

  // println!("{:?} and {:?}", node_factory_type, link_factory_type);

  // TODO redo to allow both input and output patterns at the same time
  match &cli.command {
    Some(Commands::Inputs { input_patterns }) => {
      if input_patterns.len() > 0 {
        println!("Input patterns: {:?}", input_patterns);
      } else {
        println!("No input patterns given!");
      }

      re_inputs.set(matching::patterns_to_regexes(input_patterns));
      re_outputs.set(matching::patterns_to_regexes(&vec![]));
      println!("Matching regexes: {:?}", re_inputs);
    }
    Some(Commands::Outputs { output_patterns }) => {
        if output_patterns.len() > 0 {
          println!("Output patterns: {:?}", output_patterns);
        } else {
          println!("No output patterns given!");
        }

        re_inputs.set(matching::patterns_to_regexes(&vec![]));
        re_outputs.set(matching::patterns_to_regexes(output_patterns));
        println!("Matching regexes: {:?}", re_outputs);
    }
    None => {}
  }

  // === initial setup connection, getting the current state of the world:
  // let setup_done = Rc::new(Cell::new(false));

  // Register a callback to the `global` event on the registry, which notifies of any new global objects
  // appearing on the remote.
  // The callback will only get called as long as we keep the returned listener alive.
  let _listener = registry
    .add_listener_local()
    .global_remove(clone!(@strong pw_objects, @strong pw_state,
        @strong central_node_connect_input_nodes,
        @strong central_node_connect_output_nodes => move |id| {
      println!("Removed PW object: {:?}", id);
      pw_objects.borrow_mut().remove(&id);
      pw_state.borrow_mut().del(id);

      if central_node_connect_input_nodes.borrow_mut().remove(&id) {
        println!("Removed PWACC input node {:?}", id);
      }
      if central_node_connect_output_nodes.borrow_mut().remove(&id) {
        println!("Removed PWACC output node {:?}", id);
      }
    }))
  .global(clone!(
      @weak core, @weak registry, @strong pw_objects, @strong re_inputs,
      @strong central_node_id,
      @strong central_node_ports,
      @strong central_node_connect_input_nodes,
      @strong central_node_connect_output_nodes => move |global| {

    match &global.type_ {
      ObjectType::Node => {
        // println!("handlin da node {:?}", global);
        handlers::handle_node_added(
          global, &registry, &pw_objects, &pw_state,
        );
        // let node: shared::ProxyItem::Node = pw_objects.borrow().get(&global.id).expect("dunno lol");
        if let Some(shared::ProxyItem::Node {proxy, ..}) = pw_objects.borrow().get(&global.id) {
          // println!("gotme a node: {:?}", proxy);
        }

        if matching::pw_node_matches_regexes(
            global, re_inputs.get().expect("inputs not init'd?"), cli.match_description,
        ) {
          println!("PW Node matched! Link this node: {:?}", global);
          central_node_connect_input_nodes.borrow_mut().insert(global.id);
        }
        if matching::pw_node_matches_regexes(
            global, re_outputs.get().expect("outputs not init'd?"), cli.match_description,
        ) {
          println!("PW Node matched! Link this node: {:?}", global);
          central_node_connect_output_nodes.borrow_mut().insert(global.id);
        }
      }
      ObjectType::Port => {
        println!("new port: {:?}", global);
        handlers::handle_port_added(
          global, &registry, &pw_objects, &pw_state,
        );
        let id = global.id;

        if central_node_id.get().is_none() {
            // waiting for the other end to exist...
            return;
        }

        let pw_state_clone = pw_state.borrow();
        let mut central_node_ports_clone = central_node_ports.borrow_mut();

        let shared::PwGraphItem::Port {node_id, direction} =
            pw_state_clone.get(id).expect("port not tracked after handler?")
        else {
            panic!("not a port after all?")
        };
        let Some(audio_channel) = pw_state_clone.get_port_audio_channel(id)
        else {
            println!("port has no audio channel in state!");
            return
        };

        println!("should we link port {:?} on node {:?}?", global.id, node_id);

        let central_node_id = central_node_id.get().unwrap();
        if central_node_id == node_id {
          // it is one of our own ports that got created, gotta check the backlog!
          println!(
            "       this is our port! {:?}", global);
          central_node_ports_clone.insert((direction.as_raw(), *audio_channel), id);
          // backlog loop TODO

          return;
        }
        println!("it has direction {:?}", direction);
        match *direction {
          Direction::Output => { // the target's Outputs to our Inputs
            println!("it be an output!");
            if central_node_connect_input_nodes.borrow().contains(node_id) {
              // it is an app we should link to our input
              println!("this is a target input! {:?}", global);
              let our_port: u32 = *central_node_ports_clone
                .get(&(direction.reverse().as_raw(), *audio_channel))
                .expect("we don't have a central port to match?");
              let _ = actions::connect_ports(
                &core, &pw_objects, &pw_state, id, our_port
              );
            } else {
              // nothing to do with it
              return;
            }
          }
          Direction::Input => {
            println!("it be an input!");
            if central_node_connect_output_nodes.borrow().contains(node_id) {
              // it is an app we should link to our input
              println!("this is a target output! {:?}", global);
              let our_port: u32 = *central_node_ports_clone
                .get(&(direction.reverse().as_raw(), *audio_channel))
                .expect("we don't have a central port to match?");
              let _ = actions::connect_ports(
                &core, &pw_objects, &pw_state, our_port, id
              );
            } else {
              return;
            }
          }
          _ => {
            panic!("port has an unknown or unset libspa::utils::Direction");
          }
        }

      }
      ObjectType::Link => {
        handlers::handle_link_added(
          global, &registry, &pw_objects, &pw_state,
        );
      }
      ObjectType::Factory => {
        println!("handlin da factory (jk): {:?}", global);
        // imma be lazy here and do what helvum does,
        // not gonna bother saving the factory name lmao
      }
      x => {
        // println!("something else: {:?}", x);
      }
    }

  }))
  .register();

  helpers::do_pw_roundtrip(&mainloop, &core);

  let central_node = actions::create_main_passthrough_node(
    &core, "pwacc_node", &cli.name.unwrap_or("PWACC".to_string()),
  );
  // figures out what id PW gave our new node:
  // also called when we get our ports added to it
  let central_node_listener = central_node.add_listener_local()
    .info(clone!(@strong central_node_id => move |info| {
        let id = info.id();
        let state = info.state();
        let props = info.props().expect("central node info update should give me props");
        println!("got central_node {:?} state {:?} info props {:?}", id, state, props);
        central_node_id.set(id);
    }))
    .register();

  println!("main node is {:?}", central_node);
  println!("PWACC initial sync complete");
  println!(
    "Nodes to be connected next: inputs({:?}) outputs({:?})",
    central_node_connect_input_nodes.borrow(), central_node_connect_output_nodes.borrow()
  );

  // done with this one now
  // how do I do this???
  // central_node_listener.unregister();

  println!("PWACC established and listening for new changes...");

  // === start listening for new changes only:


  mainloop.run();

  Ok(())
}
