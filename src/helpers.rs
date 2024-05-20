
use std::{cell::Cell, rc::Rc};
use pipewire as pw;

// stolen from pipewire-rs examples
/// Do a single roundtrip to process all events.
/// See the example in roundtrip.rs for more details on this.
pub fn do_pw_roundtrip(mainloop: &pw::main_loop::MainLoop, core: &pw::core::Core) {
    let done = Rc::new(Cell::new(false));
    let done_clone = done.clone();
    let loop_clone = mainloop.clone();
    // Trigger the sync event. The server's answer won't be processed until we start the main loop,
    // so we can safely do this before setting up a callback. This lets us avoid using a Cell.
    let pending = core.sync(0).expect("sync failed");
    let _listener_core = core
        .add_listener_local()
        .done(move |id, seq| {
            if id == pw::core::PW_ID_CORE && seq == pending {
                done_clone.set(true);
                loop_clone.quit();
            }
        })
        .register();
    while !done.get() {
        mainloop.run();
    }
}
