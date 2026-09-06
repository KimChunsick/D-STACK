// Mode settings and role execution share one provider/snapshot contract.
use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::verb::Verb;

pub mod execute;
mod provider;
pub mod settings;

struct ModeVerb(&'static str);

impl Verb for ModeVerb {
    fn name(&self) -> &'static str {
        self.0
    }

    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        match self.0 {
            "mode show" => settings::show(ctx, args),
            "mode set" => settings::set(ctx, args),
            _ => execute::run(ctx, args),
        }
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    ["mode show", "mode set", "mode exec"]
        .into_iter()
        .map(|name| Box::new(ModeVerb(name)) as Box<dyn Verb>)
        .collect()
}
