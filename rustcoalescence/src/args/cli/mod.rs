use clap::Parser;

pub mod replay;

#[derive(Debug, Parser)]
pub enum RustcoalescenceArgs {
    Simulate(CommandArgs),
    Replay(CommandArgs),
}

#[derive(Debug, Parser)]
#[clap(setting(clap::AppSettings::AllowLeadingHyphen))]
pub struct CommandArgs {
    args: Vec<String>,
}

impl CommandArgs {
    pub fn into_config_string(self) -> String {
        let config = self.args.join(" ");
        std::mem::drop(self);
        config
    }
}
