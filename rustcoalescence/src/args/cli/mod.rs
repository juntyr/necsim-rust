use structopt::StructOpt;

pub mod replay;

#[derive(Debug, StructOpt)]
pub enum RustcoalescenceArgs {
    Simulate(CommandArgs),
    Replay(CommandArgs),
}

#[derive(Debug, StructOpt)]
#[structopt(template("{bin} {version}\n\nUSAGE:\n    {usage} args..\n\n{all-args}"))]
#[structopt(setting(structopt::clap::AppSettings::AllowLeadingHyphen))]
pub struct CommandArgs {
    #[structopt(hidden(true))]
    args: Vec<String>,
}

impl CommandArgs {
    pub fn into_config_string(self) -> String {
        let config = self.args.join(" ");
        std::mem::drop(self);
        config
    }
}
