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

Quantities can be a plain decimal (`1.5`), a simple fraction (`1/2`), or a
mixed number (`1 1/2`). In recipe format the mixed number's whole part and
fraction stay separated by a single space; in CSV format the whole quantity
field (including that space) goes in one comma-separated column, e.g.
`1 1/2,cup,milk`.

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

## Unit conversion

`--unit <target>` converts every ingredient's unit to `<target>` after
scaling. Volume units (`tsp`, `tbsp`, `floz`, `cup`, `pint`, `quart`,
`gallon`, `ml`, `l`) convert to each other exactly, and so do weight units
(`g`, `kg`, `oz`, `lb`).

Converting across the volume/weight line (`cup` to `g`, say) needs to know
how dense the ingredient is, which `rscale` only knows for a short list of
common baking ingredients (matched by substring against the ingredient
name — "packed brown sugar" matches "brown sugar"). Lines whose ingredient
isn't in that table are reported on stderr and skipped, same as a parse
error.

```
rscale --from recipe --to recipe --scale 1 --unit g --input pancakes.recipe
```

`count` and other non-volume, non-weight units aren't convertible; a line
using one is skipped with an error if `--unit` targets a different unit.

## Building

Standard library only, no external crates:

```
cargo build --release
```

## License

MIT, see LICENSE.
