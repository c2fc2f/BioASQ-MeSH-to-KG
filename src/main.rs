pub mod models;

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::PathBuf,
    process::ExitCode,
};

use clap::Parser;
use serde_json::Deserializer;

use crate::models::{BioASQDataset, BioASQEntry};

macro_rules! create_csv {
    ($path:expr, $name:expr) => {
        match File::create($path.join($name)) {
            Ok(f) => BufWriter::new(f),
            Err(e) => {
                eprintln!("Cannot create the CSV of {}:\n\n{e}", $name);
                return ExitCode::FAILURE;
            }
        }
    };
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// CLI tool that converts the BioASQ Task A MeSH annotation dataset (large
/// JSON) into a CSV-based Knowledge Graph representation
struct Args {
    /// Path to the JSON file of BioASQ MeSH (in UTF-8 !)
    #[arg(short, long)]
    bam: PathBuf,

    /// Path to the result folder
    #[arg(short, long, default_value = ".")]
    output: PathBuf,
}

fn merge_entries(current: &mut BioASQEntry, new: BioASQEntry) {
    current.mesh.extend(new.mesh);

    let current_score: usize = current.title.len() + current.r#abstract.len();
    let new_score: usize = new.title.len() + new.r#abstract.len();

    if new_score > current_score {
        current.title = new.title;
        current.r#abstract = new.r#abstract;
        current.journal = new.journal;
    }

    if current.year.is_none() && new.year.is_some() {
        current.year = new.year;
    }
}

fn deduplicate_bioasq(
    entries: Vec<BioASQEntry>,
) -> impl Iterator<Item = BioASQEntry> {
    let mut lookup: HashMap<u32, BioASQEntry> =
        HashMap::with_capacity(entries.len());

    for entry in entries {
        if let Some(existing_entry) = lookup.get_mut(&entry.pmid) {
            merge_entries(existing_entry, entry);
        } else {
            lookup.insert(entry.pmid, entry);
        }
    }
    lookup.into_values()
}

#[allow(clippy::too_many_arguments)]
fn write_csv(
    mut a_file: BufWriter<File>,
    mut y_node_file: BufWriter<File>,
    mut j_file: BufWriter<File>,
    mut m_file: BufWriter<File>,
    mut in_j_file: BufWriter<File>,
    mut has_m_file: BufWriter<File>,
    mut pub_y_file: BufWriter<File>,
    articles: impl Iterator<Item = BioASQEntry>,
) -> io::Result<()> {
    writeln!(a_file, ":ID(Article),pmid:int,title,abstract,:LABEL")?;
    writeln!(y_node_file, ":ID(Year),year:int,:LABEL")?;
    writeln!(j_file, ":ID(Journal),name,:LABEL")?;
    writeln!(m_file, ":ID(MeSH),name,:LABEL")?;

    writeln!(in_j_file, ":START_ID(Article),:END_ID(Journal),:TYPE")?;
    writeln!(has_m_file, ":START_ID(Article),:END_ID(MeSH),:TYPE")?;
    writeln!(pub_y_file, ":START_ID(Article),:END_ID(Year),:TYPE")?;

    let mut seen_years: HashSet<i32> = HashSet::new();
    let mut seen_journals: HashMap<String, u32> = HashMap::new();
    let mut seen_mesh: HashMap<String, u32> = HashMap::new();

    let mut journal_next_id: u32 = 0;
    let mut mesh_next_id: u32 = 0;

    for entry in articles {
        writeln!(
            a_file,
            "{0},{0},\"{1}\",\"{2}\",Article",
            entry.pmid, entry.title, entry.r#abstract,
        )?;

        let j_key: u32 = match seen_journals.get(entry.journal.as_str()) {
            None => {
                writeln!(
                    j_file,
                    "{},\"{}\",Journal",
                    journal_next_id, entry.journal,
                )?;
                seen_journals.insert(entry.journal, journal_next_id);
                journal_next_id += 1;
                journal_next_id - 1
            }
            Some(j_key) => *j_key,
        };
        writeln!(in_j_file, "{},{},IN_JOURNAL", entry.pmid, j_key)?;

        if let Some(year) = entry.year {
            if !seen_years.contains(&year) {
                writeln!(y_node_file, "{0},{0},Year", year).unwrap();
                seen_years.insert(year);
            }
            writeln!(pub_y_file, "{},{},PUBLISHED_YEAR", entry.pmid, year)?;
        }

        for mesh_term in entry.mesh {
            let m_key: u32 = match seen_mesh.get(mesh_term.as_str()) {
                None => {
                    writeln!(
                        m_file,
                        "{},\"{}\",MeSH",
                        mesh_next_id, mesh_term,
                    )?;
                    seen_mesh.insert(mesh_term, mesh_next_id);
                    mesh_next_id += 1;
                    mesh_next_id - 1
                }
                Some(m_key) => *m_key,
            };
            writeln!(has_m_file, "{},{},HAS_MESH", entry.pmid, m_key)?;
        }
    }

    a_file.flush()?;
    y_node_file.flush()?;
    j_file.flush()?;
    m_file.flush()?;
    in_j_file.flush()?;
    has_m_file.flush()?;
    pub_y_file.flush()?;

    Ok(())
}

fn main() -> ExitCode {
    let args: Args = Args::parse();

    let mesh: File = match File::open(args.bam) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "Cannot open the BioASQ Task A MeSH annotation dataset:\n\n\
                {e}"
            );
            return ExitCode::FAILURE;
        }
    };

    let bioasq: BioASQDataset = match serde_path_to_error::deserialize(
        &mut Deserializer::from_reader(BufReader::new(mesh)),
    ) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "The BioASQ Task A MeSH annotation dataset does not match \
                the required schema:\n\n{e}"
            );
            return ExitCode::FAILURE;
        }
    };

    let a_file: BufWriter<File> = create_csv!(args.output, "Articles.csv");
    let y_node_file: BufWriter<File> = create_csv!(args.output, "Years.csv");
    let j_file: BufWriter<File> = create_csv!(args.output, "Journals.csv");
    let m_file: BufWriter<File> = create_csv!(args.output, "MeSHs.csv");
    let in_j_file: BufWriter<File> = create_csv!(args.output, "IN_JOURNAL.csv");
    let has_m_file: BufWriter<File> = create_csv!(args.output, "HAS_MESH.csv");
    let pub_y_file: BufWriter<File> =
        create_csv!(args.output, "PUBLISHED_YEAR.csv");

    if let Err(e) = write_csv(
        a_file,
        y_node_file,
        j_file,
        m_file,
        in_j_file,
        has_m_file,
        pub_y_file,
        deduplicate_bioasq(bioasq.articles),
    ) {
        eprintln!("Error during convertion in CSV:\n\n{e}");
        return ExitCode::FAILURE;
    };

    ExitCode::SUCCESS
}
