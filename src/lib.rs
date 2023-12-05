use clap::{Arg, ArgAction, Command};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("uniqr")
        .version("0.1.0")
        .author("Kurt Fehlhauer")
        .about("Rust uniq")
        .arg(
            Arg::new("in_file")
                .help("Input file")
                .value_name("IN_FILE")
                .default_value("-"),
        )
        .arg(Arg::new("out_file").help("Out file").value_name("OUT_FILE"))
        .arg(
            Arg::new("count")
                .help("Show count")
                .value_name("COUNT")
                .short('c')
                .long("count")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    Ok(Config {
        in_file: matches.get_one::<String>("in_file").unwrap().to_string(),
        out_file: matches.get_one::<String>("out_file").map(String::from),
        count: matches.get_flag("count"),
    })
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("   {value} ")
    } else {
        "".to_string()
    }
}

fn output(field: String, count: usize, file: &mut Option<File>, config: &Config) -> MyResult<()> {
    let formatted = format!("{}{}", format_field(count, config.count), field);
    match file {
        Some(f) => write!(f, "{}", formatted)?,
        None => print!("{}", formatted),
    }
    Ok(())
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut old_line = String::new();
    let mut line_count: usize = 1;

    let mut bytes = file.read_line(&mut line)?;
    if bytes > 0 {
        let mut out_file = match &config.out_file {
            Some(file_name) => Some(File::create(file_name)?),
            None => None,
        };
        loop {
            if bytes == 0 {
                break;
            }

            if line.trim() == old_line.trim() {
                line_count += 1;
            } else {
                if !old_line.is_empty() {
                    output(old_line, line_count, &mut out_file, &config)?;
                }
                line_count = 1;
                old_line = line.clone();
            }
            line.clear();
            bytes = file.read_line(&mut line)?;
        }
        output(old_line, line_count, &mut out_file, &config)?;
        if let Some(mut file) = out_file {
            file.flush()?;
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
