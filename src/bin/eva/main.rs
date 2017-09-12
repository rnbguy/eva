extern crate app_dirs;
extern crate clap;
extern crate config;
extern crate eva;
#[macro_use]
extern crate error_chain;
extern crate shellexpand;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use eva::configuration::Configuration;

use errors::*;

mod configuration;

#[allow(unused_doc_comment)]
mod errors {
    use configuration;
    use eva;

    error_chain! {
        links {
            Configuration(configuration::Error, configuration::ErrorKind);
        }
        foreign_links {
            EvaCore(eva::Error);
        }
    }
}


fn main() {
    if let Err(ref error) = run() {
        handle_error(error);
    }
}

fn run() -> Result<()> {
    let configuration = configuration::read()?;
    let matches = cli(&configuration).get_matches();
    dispatch(&matches, &configuration)
}

fn cli<'a, 'b>(configuration: &Configuration) -> App<'a, 'b> {
    let add = SubCommand::with_name("add")
        .about("Adds a task")
        .arg(Arg::with_name("content").required(true)
             .help("What is it that you want to do?"))
        .arg(Arg::with_name("deadline").required(true)
             .help("When should it be finished? \
                   Give it in the format of '2 Aug 2017 14:03'."))
        .arg(Arg::with_name("duration").required(true)
             .help("How long do you estimate it will take? \
                   Give it in a (whole or decimal) number of hours."))
        .arg(Arg::with_name("importance").required(true)
             .help("How important is this task to you on a scale from 1 to 10?"));
    let rm = SubCommand::with_name("rm")
        .about("Removes a task")
        .arg(Arg::with_name("task-id").required(true));
    let set = SubCommand::with_name("set")
        .about("Changes the deadline, duration, importance or content of an existing task")
        .arg(Arg::with_name("property").required(true)
             .possible_values(&["content", "deadline", "duration", "importance"]))
        .arg(Arg::with_name("task-id").required(true))
        .arg(Arg::with_name("value").required(true));
    let schedule = SubCommand::with_name("schedule")
        .about("Lets Eva suggest a schedule for your tasks")
        .arg(Arg::with_name("strategy")
             .long("strategy")
             .takes_value(true)
             .possible_values(&["importance", "urgency"])
             .default_value(configuration.scheduling_strategy.as_str()));

    App::new("eva")
        .version(env!("CARGO_PKG_VERSION"))
        .global_setting(AppSettings::ColoredHelp)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(add)
        .subcommand(rm)
        .subcommand(set)
        .subcommand(schedule)
}

fn dispatch(inputs: &ArgMatches, configuration: &Configuration) -> Result<()> {
    match inputs.subcommand() {
        ("add", Some(submatches)) => {
            let content = submatches.value_of("content").unwrap();
            let deadline = submatches.value_of("deadline").unwrap();
            let duration = submatches.value_of("duration").unwrap();
            let importance = submatches.value_of("importance").unwrap();
            let importance: u32 = try!(importance.parse()
                .chain_err(|| "Please supply a valid integer as importance factor."));
            Ok(eva::add(configuration, content, deadline, duration, importance)?)
        },
        ("rm", Some(submatches)) => {
            let id = submatches.value_of("task-id").unwrap();
            let id: u32 = id.parse()
                .chain_err(|| "Please supply a valid integer as id.")?;
            Ok(eva::remove(configuration, id)?)
        },
        ("set", Some(submatches)) => {
            let field = submatches.value_of("property").unwrap();
            let id = submatches.value_of("task-id").unwrap();
            let value = submatches.value_of("value").unwrap();
            let id: u32 = id.parse()
                .chain_err(|| "Please supply a valid integer as id.")?;
            Ok(eva::set(configuration, field, id, value)?)
        }
        ("schedule", Some(submatches)) => {
            let strategy = submatches.value_of("strategy").unwrap();
            Ok(eva::print_schedule(configuration, strategy)?)
        },
        _ => unreachable!(),
    }
}

fn handle_error(error: &Error) {
    let chain = error.iter().skip(1)
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(". ");

    if chain.is_empty() {
        eprintln!("{}.", error);
    } else {
        eprintln!("{}. ({})", error, chain);
    }

    // Print backtrace when RUST_BACKTRACE=1
    if let Some(backtrace) = error.backtrace() {
        eprintln!("{:?}", backtrace);
    }

    ::std::process::exit(1);
}