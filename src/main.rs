mod actions;
mod handlers;
mod matching;
mod shared;

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

//fn pw_create_unconnected_node(

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();

  pipewire::init();
  let mainloop = MainLoop::new(None)?;
  let context = Context::new(&mainloop)?;
  let core = Rc::new(context.connect(None).expect("failed to connect"));
  let registry = Rc::new(core.get_registry().expect("No registry?"));

  let orig_pw_objects: HashMap<u32, shared::ProxyItem> = HashMap::new();
  let pw_objects: Rc<RefCell<HashMap<u32, shared::ProxyItem>>> =
    Rc::new(RefCell::new(orig_pw_objects));
  // let orig_pw_factories: HashMap<String, u32> = HashMap::new();
  // let pw_factories: Rc<RefCell<HashMap<String, u32>>> = Rc::new(RefCell::new(orig_pw_factories));

  //let re_inputs: Rc<OnceCell<Vec<Regex>>> = Rc::new(OnceCell::new());
  //let re_inputs_clone = re_inputs.clone();
  let re_inputs: OnceCell<Vec<Regex>> = OnceCell::new();
  //let re_outputs: Vec<Regex> = vec![];

  let central_node = actions::create_main_passthrough_node(
    &core, "pwacc_node_name",
  );

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

  // Register a callback to the `global` event on the registry, which notifies of any new global objects
  // appearing on the remote.
  // The callback will only get called as long as we keep the returned listener alive.
  let _listener = registry
    .add_listener_local()
    .global_remove(clone!(@strong pw_objects => move |id| {
      println!("Removed PW object: {:?}", id);
      pw_objects.borrow_mut().remove(&id);
    }))
  .global(clone!(@weak registry, @strong pw_objects, @strong re_inputs => move |global| {

    match &global.type_ {
      ObjectType::Node => {
        println!("handlin da node {:?}", global);
        handlers::handle_node_added(
          global, &registry, &pw_objects,
          );
        // let node: shared::ProxyItem::Node = pw_objects.borrow().get(&global.id).expect("dunno lol");
        if let Some(shared::ProxyItem::Node {proxy, ..}) = pw_objects.borrow().get(&global.id) {
          println!("gotme a node: {:?}", proxy);
        }

        if matching::pw_node_is_readable(global) {
          println!("PW object is readable");
          if matching::pw_node_matches_regexes(global, re_inputs.get().expect("No inputs?")) {
            println!("PW Node matched! Link it!");
          }
        }
      }
      ObjectType::Port => {
        println!("new port: {:?}", global);
        handlers::handle_port_added(
          global, &registry, &pw_objects,
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

    // let res = handlers::handle_new_object(
    //     global, &registry, &pw_objects,
    // )
    // pw_objects.borrow_mut().insert(global.id, global);
    //if global.type_ == pipewire::types::ObjectType::Factory {
    //  println!("factory: {:?}", global);
    //  if let Some(props) = global.props {
    //    println!("has props: {:?}", props);
    //    let factory_type = props.get("factory.type.name").expect("Factory has no type").to_string();
    //    rc_pw_factories_clone.borrow_mut().insert(factory_type, global.id);

    //      //// Link factory:
    //      //link_factory_type => {
    //      //  println!("found {:?}", link_factory_type);
    //      //  let link_factory_name = props.get("factory.name").expect("Factory has no name");
    //      //  println!("before setting, link_factory is {:?}", link_factory.get());
    //      //  link_factory
    //      //    .set(link_factory_name.to_owned())
    //      //    .expect("Name was already set?");
    //      //}
    //      //node_factory_type => {
    //      //  println!("found {:?}", node_factory_type);
    //      //  let node_factory_name = props.get("factory.name").expect("Factory has no name");
    //      //  node_factory
    //      //    .set(node_factory_name.to_owned())
    //      //    .expect("Name was already set?");
    //      //}
    //      ////Some(s) => {
    //      ////  println!("we aint lookin for {:?}", s);
    //      ////}
    //      ////None => {}
    //  }
    //}
    // if !matching::pw_object_is_node(&global) {
    //   //println!("non-node object is {:?}", global.type_);
    //   return;
    // }
    //
    // // Run checks on the node against our rules
    // println!("PW object is node: {:?}", global);

  }))
  .register();

  println!("main node is {:?}", central_node);

  // Calling the `destroy_global` method on the registry will destroy the object with the specified id on the remote.
  // We don't have a specific object to destroy now, so this is commented out.
  // registry.destroy_global(313).into_result()?;

  mainloop.run();

  Ok(())
}
