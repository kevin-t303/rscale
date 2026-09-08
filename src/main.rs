mod ingredient;
mod units;

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
    unit: Option<String>,
    dry_run: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut from = None;
    let mut to = None;
    let mut scale = None;
    let mut input = None;
    let mut output = None;
    let mut unit = None;
    let mut dry_run = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--from" => from = Some(args.next().ok_or("--from needs a value")?),
            "--to" => to = Some(args.next().ok_or("--to needs a value")?),
            "--scale" => scale = Some(args.next().ok_or("--scale needs a value")?),
            "--input" => input = Some(args.next().ok_or("--input needs a value")?),
            "--output" => output = Some(args.next().ok_or("--output needs a value")?),
            "--unit" => unit = Some(args.next().ok_or("--unit needs a value")?),
            "--dry-run" => dry_run = true,
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
        unit,
        dry_run,
    })
}

/// Runs the conversion. Returns whether any line failed to parse or convert.
/// In dry-run mode no writer is opened at all, so an existing `--output`
/// file is left untouched rather than being truncated by a run that's only
/// meant to check the input.
fn run(args: &Args) -> io::Result<bool> {
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
    let mut writer: Option<Box<dyn Write>> = if args.dry_run {
        None
    } else {
        match &args.output {
            Some(path) => {
                file_out = File::create(path)?;
                Some(Box::new(BufWriter::new(file_out)))
            }
            None => {
                stdout = io::stdout();
                Some(Box::new(BufWriter::new(stdout)))
            }
        }
    };

    let mut line_count = 0;
    let mut error_count = 0;

    for (number, line) in reader.lines().enumerate() {
        let line = line?;
        let line_number = number + 1;

        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            if let Some(writer) = writer.as_mut() {
                writeln!(writer, "{line}")?;
            }
            continue;
        }

        line_count += 1;

        let parsed = match args.from {
            Format::Recipe => parse_recipe_line(&line),
            Format::Csv => parse_csv_line(&line),
        };

        let ingredient = match parsed {
            Ok(ingredient) => ingredient,
            Err(err) => {
                eprintln!("line {line_number}: {err}");
                error_count += 1;
                continue;
            }
        };

        let mut scaled = ingredient.scaled(args.scale);

        if let Some(target_unit) = &args.unit {
            match units::convert(scaled.quantity, &scaled.unit, target_unit, &scaled.name) {
                Ok(quantity) => {
                    scaled.quantity = quantity;
                    scaled.unit = target_unit.clone();
                }
                Err(err) => {
                    eprintln!("line {line_number}: {err}");
                    error_count += 1;
                    continue;
                }
            }
        }

        if let Some(writer) = writer.as_mut() {
            let out_line = match args.to {
                Format::Recipe => format_recipe_line(&scaled),
                Format::Csv => format_csv_line(&scaled),
            };
            writeln!(writer, "{out_line}")?;
        }
    }

    if let Some(writer) = writer.as_mut() {
        writer.flush()?;
    }

    if args.dry_run {
        println!("dry run: {line_count} line(s) checked, {error_count} error(s)");
    }

    Ok(error_count > 0)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("error: {err}");
            eprintln!(
                "usage: rscale --from <recipe|csv> --to <recipe|csv> --scale <factor> [--unit UNIT] [--input FILE] [--output FILE] [--dry-run]"
            );
            return ExitCode::FAILURE;
        }
    };

    let dry_run = args.dry_run;
    match run(&args) {
        Ok(had_errors) => {
            if dry_run && had_errors {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
