mod frame;
mod node226;
use message_io::node::{self};
use node226::{start_node, Node226, Signals226};

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let (handler, listener) = node::split::<Signals226>();
    let handler_c = handler.clone();
    let mut lis_thread = message_io::util::thread::NamespacedThread::spawn("Net226", move || {
        start_node(Node226::default(), handler_c, listener);
    });

    ctrlc::set_handler(move || {
        handler.signals().send(Signals226::Quit);
    })
    .expect("Error setting Ctrl-C handler");

    lis_thread.join();
}
