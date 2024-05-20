mod actions;
mod handlers;
mod matching;
mod shared;
mod constants;
mod helpers;

use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{cell::Cell, rc::Rc};

use adw::glib::{self, clone};
use clap::{Parser, Subcommand};
use libspa::utils::dict::DictRef;
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
  /// Optional name to operate on
  name: Option<String>,

  /// Sets a custom config file
  #[arg(short, long, value_name = "FILE")]
  inputs: Option<PathBuf>,

  /// Turn debugging information on
  #[arg(short, long, action = clap::ArgAction::Count)]
  debug: u8,

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
  //let re_outputs: Vec<Regex> = vec![];

  let central_node_connect_input_backlog: Rc<RefCell<Vec<u32>>> = Rc::new(RefCell::new(Vec::new()));
  let central_node_connect_output_backlog: Rc<RefCell<Vec<u32>>> = Rc::new(RefCell::new(Vec::new()));

  // helpers::do_pw_roundtrip(&mainloop, &core);

  // let node_factory_type = ObjectType::Node.to_str();
  // let node_factory: OnceCell<String> = OnceCell::new();
  // //let node_factory_clone = node_factory.clone();
  //
  // let link_factory_type = ObjectType::Link.to_str();
  // let link_factory: OnceCell<String> = OnceCell::new();
  // //let link_factory_clone = link_factory.clone();

  // println!("{:?} and {:?}", node_factory_type, link_factory_type);

  match &cli.command {
    Some(Commands::Inputs { input_patterns }) => {
      if input_patterns.len() > 0 {
        println!("Input patterns: {:?}", input_patterns);
      } else {
        println!("No input patterns given!");
      }

      re_inputs.set(matching::patterns_to_regexes(input_patterns));
      //re_inputs = patterns_to_regexes(input_patterns);
      println!("Matching regexes: {:?}", re_inputs);
    }
    Some(Commands::Outputs { output_patterns }) => {
      println!("NotImplemented!");
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
    .global_remove(clone!(@strong pw_objects, @strong pw_state => move |id| {
      println!("Removed PW object: {:?}", id);
      pw_objects.borrow_mut().remove(&id);
      pw_state.borrow_mut().del(id);
    }))
  .global(clone!(
      @weak registry, @strong pw_objects, @strong re_inputs,
      @strong central_node_connect_input_backlog,
      @strong central_node_connect_output_backlog => move |global| {

    match &global.type_ {
      ObjectType::Node => {
        println!("handlin da node {:?}", global);
        handlers::handle_node_added(
          global, &registry, &pw_objects, &pw_state,
        );
        // let node: shared::ProxyItem::Node = pw_objects.borrow().get(&global.id).expect("dunno lol");
        if let Some(shared::ProxyItem::Node {proxy, ..}) = pw_objects.borrow().get(&global.id) {
          println!("gotme a node: {:?}", proxy);
        }

        if matching::pw_node_is_readable(global) {
          println!("PW object is readable");
          if matching::pw_node_matches_regexes(global, re_inputs.get().expect("No inputs?")) {
            println!("PW Node matched! Link it!");
            central_node_connect_input_backlog.borrow_mut().push(global.id);
          }
        }
        // TODO writable
      }
      ObjectType::Port => {
        println!("new port: {:?}", global);
        handlers::handle_port_added(
          global, &registry, &pw_objects, &pw_state,
        );
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

  // .done(clone!(@strong setup_done @strong mainloop => move |id, seq| {
  //   if id == pw::core::PW_ID_CORE && seq == pending {
  //     setup_done.set(true);
  //     mainloop.quit();
  //     println!("PWACC initial sync complete");
  //   }
  // }))

  helpers::do_pw_roundtrip(&mainloop, &core);

  // while (!setup_done.get()) {
  //   mainloop.run();
  // }

  println!("PWACC initial sync complete");
  println!(
    "Nodes to connect next: inputs({:?}) outputs({:?})",
    central_node_connect_input_backlog.borrow(), central_node_connect_output_backlog.borrow()
  );

  // === start adding our own things now & listening for new changes:

  let central_node = actions::create_main_passthrough_node(
    &core, "pwacc_node_name",
  );
  let central_node_id: Rc<OnceCell<u32>> = Rc::new(OnceCell::new());

  // figures out what id PW gave our new node:
  // also called when we get our ports added to it
  let _central_node_listener = central_node.add_listener_local()
    .info(clone!(@strong central_node_id => move |info| {
        let id = info.id();
        let props = info.props();
        println!("got central_node {:?} info props {:?}", id, props);
        central_node_id.set(id);
    }))
    .register();

  println!("main node is {:?}", central_node);

  mainloop.run();

  Ok(())
}
