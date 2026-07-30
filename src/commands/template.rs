use clap::Subcommand;
use owo_colors::OwoColorize;
use crate::style;

#[derive(Subcommand, Debug, Clone)]
pub enum TemplateAction {
    #[command(about = "Example action - replace with your own")]
    Run {
        #[arg(short, long, value_name = "NAME")]
        name: Option<String>,
    },
}

pub fn run(action: &TemplateAction) {
    match action {
        TemplateAction::Run { name } => {
            println!("{}", "Template Command".style(style::Theme::HEADER));
            if let Some(n) = name {
                println!("{}", style::label_value("Name", n));
            } else {
                println!("{}", "No name provided. Use --name <NAME>".style(style::Theme::MUTED));
            }
        }
    }
}
