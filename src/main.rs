#![allow(clippy::collapsible_if)]

use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long)]
    long: bool,

    #[clap(short)]
    short: bool,

    #[clap(long)]
    with_val: Option<String>,

    ctx: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Complete {
        #[clap(long)]
        position: Option<usize>,

        #[clap(long)]
        raw: Option<String>,

        #[clap(trailing_var_arg = true)]
        cmd: Vec<String>,
    },
    Frobnicate {
        #[clap(short, long)]
        recursive: bool,

        #[clap(short, long)]
        update: Option<String>,

        path: String,

        path2: String,
    },
}

fn my_custom_suggest(subcommand: &str, arg: &str, to_complete: &str) -> Vec<String> {
    let mut completions = Vec::new();
    for i in 0..5 {
        completions.push(format!("{to_complete}/{i}-{subcommand}-{arg}"))
    }
    completions
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Complete { position, raw, cmd } => {
            let (prev_arg, to_complete) = match (position, &raw) {
                (Some(position), Some(raw)) => {
                    if position <= raw.len() {
                        let (before, _) = raw.split_at(position);
                        let (before, to_complete) = before.rsplit_once(' ').unwrap();
                        let (_, prev_arg) = before.rsplit_once(' ').unwrap_or(("", before));
                        (prev_arg, to_complete)
                    } else {
                        (cmd.last().unwrap().as_str(), "")
                    }
                }
                _ => match cmd.as_slice() {
                    [] => ("", ""),
                    [a] => ("", a.as_ref()),
                    [.., a, b] => (a.as_ref(), b.as_ref()),
                },
            };

            eprintln!("{}, {}", prev_arg, to_complete);

            let command = <Cli as CommandFactory>::command().ignore_errors(true);
            let matches = command.clone().try_get_matches_from(&cmd).unwrap();

            let mut completions =
                recurse_completions(prev_arg, to_complete, &command, &matches, my_custom_suggest);

            // only suggest words that match what the user has already entered
            completions.retain(|x| x.starts_with(to_complete));

            // we want "whole words" to appear before flags
            completions.sort_by(|a, b| match (a.starts_with('-'), b.starts_with('-')) {
                (true, true) => a.cmp(b),
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                (false, false) => a.cmp(b),
            });

            for completion in completions {
                println!("{}", completion)
            }
        }
        Command::Frobnicate {
            recursive,
            update,
            path,
            path2,
        } => todo!(),
    }
}

// TODO: probably needs more stuff (e.g: the root-level `clap::ArgMatches`)
type CustomCompleteFn = fn(subcommand: &str, arg_id: &str, to_complete: &str) -> Vec<String>;

// drills-down through subcommands to generate the right completion set
fn recurse_completions(
    prev_arg: &str,
    to_complete: &str,
    command: &clap::Command,
    matches: &clap::ArgMatches,
    custom_complete_fn: CustomCompleteFn,
) -> Vec<String> {
    if let Some((name, matches)) = matches.subcommand() {
        let command = command
            .get_subcommands()
            .find(|s| s.get_name() == name)
            .unwrap();
        recurse_completions(prev_arg, to_complete, command, matches, custom_complete_fn)
    } else {
        let mut completions = Vec::new();

        // first, check if the previous arg is a `-` arg or a `--` arg that
        // accepts a free-form completion value.
        //
        // if so, then we need to limit our completions to just the things that
        // arg expects
        for arg in command.get_arguments() {
            let long = arg.get_long().map(|x| format!("--{x}")).unwrap_or_default();
            let short = arg.get_short().map(|x| format!("-{x}")).unwrap_or_default();

            if prev_arg != long && prev_arg != short {
                continue;
            }

            // check if it needs an arg
            if !matches!(
                arg.get_action(),
                clap::ArgAction::Append | clap::ArgAction::Set
            ) {
                continue;
            }

            // ah, ok, the current completion corresponds to the value of the
            // prev_arg!

            return (custom_complete_fn)(command.get_name(), arg.get_id().as_str(), to_complete);
        }

        // at this point, we know we're subcommand-level completions

        // easy stuff out of the way: suggest `-` and `--` flags
        for arg in command.get_arguments() {
            // check if the arg was already set, and if so, don't suggest it again
            //
            // FIXME: handle args that can be set multiple times
            let is_set = match arg.get_action() {
                clap::ArgAction::SetTrue => matches.get_flag(arg.get_id().as_str()),
                clap::ArgAction::SetFalse => !matches.get_flag(arg.get_id().as_str()),
                _ => {
                    clap::parser::ValueSource::DefaultValue
                        != matches
                            .value_source(arg.get_id().as_str())
                            .unwrap_or(clap::parser::ValueSource::DefaultValue)
                }
            };

            if is_set {
                continue;
            }

            if let Some(long) = arg.get_long() {
                completions.push(format!("--{long}"))
            }

            if let Some(short) = arg.get_short() {
                completions.push(format!("-{short}"))
            }
        }

        // with those out of the way... check for all the positional args
        for positional in command.get_positionals() {
            // check if the arg has already been set, and if so: skip its
            // corresponding suggestions
            if matches
                .try_contains_id(positional.get_id().as_str())
                .unwrap_or(true)
            {
                // ...but if the user is actively overriding the already-set
                // arg, then _don't_ skip it!
                if !matches
                    .try_get_raw(positional.get_id().as_str())
                    .unwrap_or_default()
                    .unwrap_or_default()
                    .any(|x| x.to_string_lossy().starts_with(to_complete))
                {
                    continue;
                }

                // `starts_with` will return `true` when `to_complete` is empty,
                // so we need to add this extra check here
                if to_complete.is_empty() {
                    continue;
                }
            }

            if !to_complete.starts_with('-') {
                completions.extend((custom_complete_fn)(
                    command.get_name(),
                    positional.get_id().as_str(),
                    to_complete,
                ));
            }

            // don't suggest anything else, since there are some positionals
            // that have yet to be filled
            return completions;
        }

        for subcommand in command.get_subcommands() {
            completions.push(subcommand.get_name().into())
        }

        completions
    }
}
