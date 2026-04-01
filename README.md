# bam2kg

A fast Rust CLI tool that converts the [BioASQ Task A](http://bioasq.org/) MeSH annotation dataset (large JSON) into a set of CSV files ready to be imported into a [Neo4j](https://neo4j.com/) knowledge graph.

## Overview

The BioASQ Task A dataset contains PubMed articles annotated with MeSH (Medical Subject Headings) terms. `bam2kg` parses this dataset, deduplicates entries by PMID, and emits CSV node and relationship files that Neo4j's bulk importer can consume directly.

### Graph schema

```
(Article)-[:IN_JOURNAL]->(Journal)
(Article)-[:HAS_MESH]->(MeSH)
(Article)-[:PUBLISHED_YEAR]->(Year)
```

### Output files

| File | Type | Description |
|---|---|---|
| `Articles.csv` | Node | PubMed articles (PMID, title, abstract) |
| `Journals.csv` | Node | Journals referenced by articles |
| `MeSHs.csv` | Node | MeSH terms used for annotation |
| `Years.csv` | Node | Publication years |
| `IN_JOURNAL.csv` | Relationship | Article → Journal |
| `HAS_MESH.csv` | Relationship | Article → MeSH term |
| `PUBLISHED_YEAR.csv` | Relationship | Article → Year |

## Installation

### With Nix (recommended)

```sh
nix run github:c2fc2f/bam2kg -- --help
```

To enter a development shell with all tooling (Rust, rust-analyzer, clippy, rustfmt):

```sh
nix develop
```

### With Cargo

```sh
cargo build --release
./target/release/bam2kg --help
```

## Usage

```
bam2kg --bam <path-to-dataset.json> [--output <output-folder>]
```

**Arguments:**

- `-b`, `--bam` — Path to the BioASQ Task A MeSH JSON file (must be UTF-8)
- `-o`, `--output` — Destination folder for the CSV files (defaults to the current directory)

**Example:**

```sh
bam2kg --bam BioASQ-training14b.json --output ./MeSH
```

## Importing into Neo4j

Once the CSVs are generated, use `neo4j-admin` to bulk-import them:

```sh
sudo neo4j-admin database import full \
  --verbose \
  --nodes=Article=/path/to/MeSH/Articles.csv \
  --nodes=Year=/path/to/MeSH/Years.csv \
  --nodes=Journal=/path/to/MeSH/Journals.csv \
  --nodes=MeSH=/path/to/MeSH/MeSHs.csv \
  --relationships=IN_JOURNAL=/path/to/MeSH/IN_JOURNAL.csv \
  --relationships=HAS_MESH=/path/to/MeSH/HAS_MESH.csv \
  --relationships=PUBLISHED_YEAR=/path/to/MeSH/PUBLISHED_YEAR.csv \
  --overwrite-destination=true \
  neo4j
```

> On some JVM versions you may need to add `JDK_JAVA_OPTIONS="--add-opens=java.base/java.nio=ALL-UNNAMED"` before the command, and optionally `--additional-config` pointing to your `neo4j.conf`.

## Implementation notes

- Duplicate PMIDs are merged: MeSH sets are unioned, and the entry with the longer title + abstract is kept.
- CSV fields containing quotes or newlines are escaped for Neo4j compatibility.
- The tool streams and deserializes the JSON incrementally to keep memory usage manageable on large datasets.

## License

MIT
