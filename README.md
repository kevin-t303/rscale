# rscale

Every recipe box scaler I've found either wants you to paste text into a web
form or assumes the whole recipe fits in RAM. `rscale` is a command-line
converter between two plain-text recipe formats — a human-writable one and a
CSV one that's easy to pipe into a spreadsheet or another script — and it
multiplies every quantity by a scale factor as it goes.

It reads and writes one line at a time, so a recipe file (or a batch of
thousands of them concatenated together) never has to fit in memory. That's
the part I care about getting right; the format conversion is secondary.

## Formats

**recipe** — one ingredient per line: `<quantity> <unit> <name>`

```
2 cups flour
1 tsp baking powder
1/2 cup milk
3 count eggs
```

**csv** — one ingredient per line: `<quantity>,<unit>,<name>`

```
2,cups,flour
1,tsp,baking powder
0.5,cup,milk
3,count,eggs
```

Blank lines and lines starting with `#` are passed through unchanged in
either format, so titles and notes survive a round trip.

Quantities can be a plain decimal (`1.5`) or a simple fraction (`1/2`).
Mixed numbers like `1 1/2` aren't supported yet — see the roadmap.

## Usage

Double a recipe, recipe format in and out:

```
rscale --from recipe --to recipe --scale 2 --input pancakes.recipe
```

Convert a recipe file to CSV at full size, writing to a file:

```
rscale --from recipe --to csv --scale 1 --input pancakes.recipe --output pancakes.csv
```

Scale a CSV export down to a third and read/write through pipes:

```
cat batch.csv | rscale --from csv --to csv --scale 0.33 > batch_small.csv
```

If `--input` or `--output` are omitted, `rscale` reads stdin and writes
stdout, so it composes with other line-oriented tools.

Lines that fail to parse are reported on stderr with their line number and
skipped, rather than aborting the whole conversion.

## Building

Standard library only, no external crates:

```
cargo build --release
```

## License

MIT, see LICENSE.
