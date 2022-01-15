use clap::Parser;

use crate::container::Container;

mod container;

#[derive(Parser)]
#[clap(version, author)]
struct RuntimeOpts {
    /// Container bundle
    #[clap(short, long)]
    bundle: String,
}

fn main() -> crate::container::Result<()> {
    let opts: RuntimeOpts = RuntimeOpts::parse();

    // Create a container by passing the bundle provided in arguments to it's constructor.
    let container = Container::new(&opts.bundle)?;

    // Run the container
    // At the moment, we don't have a detached mode for the container,
    // So the method call is blocking.
    container.run()?;

    Ok(())
}
