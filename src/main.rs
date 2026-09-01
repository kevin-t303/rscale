mod ingredient;

use ingredient::{format_csv_line, format_recipe_line, parse_csv_line, parse_recipe_line};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Format {
    Recipe,
    Csv,
}

impl Format {
    fn parse(s: &str) -> Option<Format> {
        match s {
            "recipe" => Some(Format::Recipe),
            "csv" => Some(Format::Csv),
            _ => None,
        }
    }
}

struct Args {
    from: Format,
    to: Format,
    scale: f64,
    input: Option<String>,
    output: Option<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut from = None;
    let mut to = None;
    let mut scale = None;
    let mut input = None;
    let mut output = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--from" => from = Some(args.next().ok_or("--from needs a value")?),
            "--to" => to = Some(args.next().ok_or("--to needs a value")?),
            "--scale" => scale = Some(args.next().ok_or("--scale needs a value")?),
            "--input" => input = Some(args.next().ok_or("--input needs a value")?),
            "--output" => output = Some(args.next().ok_or("--output needs a value")?),
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let from = from.ok_or("--from is required (recipe or csv)")?;
    let to = to.ok_or("--to is required (recipe or csv)")?;
    let scale = scale.ok_or("--scale is required, e.g. --scale 2.5")?;

    let from = Format::parse(&from).ok_or_else(|| format!("unknown format: {from}"))?;
    let to = Format::parse(&to).ok_or_else(|| format!("unknown format: {to}"))?;
    let scale: f64 = scale
        .parse()
        .map_err(|_| format!("--scale must be a number, got {scale:?}"))?;
    if scale <= 0.0 {
        return Err("--scale must be greater than zero".to_string());
    }

    Ok(Args {
        from,
        to,
        scale,
        input,
        output,
    })
}

fn run(args: Args) -> io::Result<()> {
    // Both the reader and writer are line-buffered wrappers, not owned
    // strings, so a multi-gigabyte recipe file costs one line of memory
    // at a time rather than the whole file.
    let stdin;
    let file_in;
    let reader: Box<dyn BufRead> = match &args.input {
        Some(path) => {
            file_in = File::open(path)?;
            Box::new(BufReader::new(file_in))
        }
        None => {
            stdin = io::stdin();
            Box::new(BufReader::new(stdin))
        }
    };

    let stdout;
    let file_out;
    let mut writer: Box<dyn Write> = match &args.output {
        Some(path) => {
            file_out = File::create(path)?;
            Box::new(BufWriter::new(file_out))
        }
        None => {
            stdout = io::stdout();
            Box::new(BufWriter::new(stdout))
        }
    };

    for (number, line) in reader.lines().enumerate() {
        let line = line?;
        let line_number = number + 1;

        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            writeln!(writer, "{line}")?;
            continue;
        }

        let parsed = match args.from {
            Format::Recipe => parse_recipe_line(&line),
            Format::Csv => parse_csv_line(&line),
        };

        let ingredient = match parsed {
            Ok(ingredient) => ingredient,
            Err(err) => {
                eprintln!("line {line_number}: {err}");
                continue;
            }
        };

        let scaled = ingredient.scaled(args.scale);

        let out_line = match args.to {
            Format::Recipe => format_recipe_line(&scaled),
            Format::Csv => format_csv_line(&scaled),
        };
        writeln!(writer, "{out_line}")?;
    }

    writer.flush()
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("error: {err}");
            eprintln!(
                "usage: rscale --from <recipe|csv> --to <recipe|csv> --scale <factor> [--input FILE] [--output FILE]"
            );
            return ExitCode::FAILURE;
        }
    };

    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
