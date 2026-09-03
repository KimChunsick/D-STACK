// main.rs
// The dstack binary: build the Context, dispatch through the registry, exit with 0, 1 or 2.

use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::error::Error;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::verbs;

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let home = match Home::resolve() {
        Ok(home) => home,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(error.code());
        }
    };
    let self_exe = std::env::current_exe().unwrap_or_default();
    let registry = Rc::new(Registry::new(verbs::all_verbs()));
    let mut ctx = Context::new(home, self_exe, Rc::clone(&registry));
    let code = match registry.dispatch(&mut ctx, &argv) {
        Ok(()) => 0,
        // The shell's plain `exit n`: whatever had to be said is already on stdout.
        Err(Error::Exit(code)) => code,
        Err(error) => {
            ctx.out.err_line(&error.to_string());
            error.code()
        }
    };
    ctx.out.flush();
    std::process::exit(code);
}
