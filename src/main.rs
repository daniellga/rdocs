use clap::Parser;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Component, PathBuf},
    process::Command,
};

/// Create docs from R scripts
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Relative path to the files
    #[arg(short, long, num_args = 1..)]
    files: Vec<String>,

    /// Docs will be created in this path
    #[arg(short, long)]
    docs_path: String,

    /// Source button in docs will point to files in this github url
    #[arg(short, long, default_value_t = String::from(""))]
    gh_url: String,
}

fn main() {
    let args = Args::parse();
    let files = args.files;
    let gh_url = args.gh_url;
    let docs_path = PathBuf::from(args.docs_path);

    let mut hash: HashMap<String, Vec<String>> = HashMap::new();

    generate_r_docs(files, gh_url, &mut hash);
    output_file(hash, &docs_path);
    quarto_process(&docs_path);
}

// Currently it may give a bug if 2 methods impl for the same struct are on different files,
// depending on the order of the files on the list below. Try to reorder the vec in a way that
// if the code chunk contains "# Methods" it will be swapped to the vec's first position.
fn generate_r_docs(files: Vec<String>, gh_url: String, hash: &mut HashMap<String, Vec<String>>) {
    for file in &files {
        // Read the input file and filter to keep only lines starting with "###"
        let input_file = File::open(file).unwrap();
        let mut key = String::new();
        let mut last_line_was_comment = false;
        let mut skip_comment_chunk = false;

        // counts the line in a code chunk
        let mut counter: i32 = -1;
        for (line_counter, line) in BufReader::new(input_file).lines().flatten().enumerate() {
            let line_trimmed = line.trim_start();

            if let Some(stripped) = line_trimmed.strip_prefix("///") {
                counter += 1;
                if skip_comment_chunk {
                    continue;
                }

                // skip first space.
                let filtered_line = stripped.strip_prefix(' ').unwrap_or(stripped).to_string();

                // associate with key in first line of comment chunk. Keys are identifiable by a 1 word line.
                if !last_line_was_comment {
                    key = filtered_line.clone();
                    // key should have only one word
                    if key.contains(' ') {
                        skip_comment_chunk = true;
                        continue;
                    }
                    hash.entry(key.clone()).or_insert_with(Vec::new);
                    last_line_was_comment = true;
                } else {
                    hash.get_mut(&key).unwrap().push(filtered_line);
                }
            } else if let Some(stripped) = line_trimmed.strip_prefix("###") {
                counter += 1;
                if skip_comment_chunk {
                    continue;
                }

                // skip first space.
                let filtered_line = stripped.strip_prefix(' ').unwrap_or(stripped).to_string();

                // associate with key in first line of comment chunk. Keys are identifiable by a 1 word line.
                if !last_line_was_comment {
                    key = filtered_line.clone();
                    // key should have only one word
                    if key.contains(' ') {
                        skip_comment_chunk = true;
                        continue;
                    }
                    hash.entry(key.clone()).or_insert_with(Vec::new);
                    last_line_was_comment = true;
                } else {
                    hash.get_mut(&key).unwrap().push(filtered_line);
                }
            } else {
                if !gh_url.is_empty() {
                    // Regular expression to match function declarations
                    let fn_declaration_regex = Regex::new(r"\s*fn\s+[a-zA-Z_]\w*\s*\(").unwrap();
                    let function_declaration_regex =
                        Regex::new(r"\s*function\s+[a-zA-Z_]\w*\s*\(").unwrap();

                    // add the source text. Code on github must be updated.
                    if last_line_was_comment
                        && (fn_declaration_regex.is_match(line_trimmed)
                            || function_declaration_regex.is_match(line_trimmed))
                    {
                        let vec = hash.get_mut(&key).unwrap();
                        let len = vec.len();
                        let elem = &mut vec[len - counter as usize + 2];
                        elem.pop();

                        let filename = files
                            .iter()
                            .find(|&x| x.contains(&key.to_lowercase()))
                            .unwrap();

                        // Remove a possible starting dot in path.
                        let filename_str = if filename.strip_prefix(".").is_some() {
                            &filename[1..]
                        } else {
                            &filename[..]
                        };

                        let source = "<span style=\"float: right;\"> [source](".to_string()
                            + gh_url.as_str()
                            + filename_str
                            + "#L"
                            + &(line_counter + 1).to_string()
                            + ") </span> \\";
                        elem.push_str(&source);
                    }
                }

                counter = -1;
                skip_comment_chunk = false;
                last_line_was_comment = false;
            }
        }
    }
}

fn output_file(hash: HashMap<String, Vec<String>>, docs_path: &PathBuf) {
    for (key, value) in hash {
        let key_lowercase = key.to_lowercase();

        // Construct the output file path as the input file path with a .qmd extension.
        let docs_file_path = docs_path.join(&key_lowercase).with_extension("qmd");

        // Create the folder if it doesn't exist.
        fs::create_dir_all(docs_path).expect("directory could not be created");

        // Write the output text to the output file.
        write_output_file(&docs_file_path, &key, &value);
    }
}

fn write_output_file(file_path: &PathBuf, key: &str, value: &[String]) {
    // Construct the header
    let title = format!("title: {}", key);
    let text = ["---", &title, "---"].join("\n");

    // Construct the final output text.
    let output_text = [text, value.join("\n")].join("\n\n");

    // Write the output text to the output file.
    let mut output_file = File::create(file_path).expect("could not create output_file");
    output_file
        .write_all(output_text.as_bytes())
        .expect("could not write to output_file");
}

// Create a quarto project and render.
fn quarto_process(docs_path: &PathBuf) {
    let work_dir = docs_path.parent().unwrap();
    let folder_name = docs_path.file_name().unwrap();

    // If the directory is already used as a quarto project, it should error but the rest of the program is run anyway.
    let command = Command::new("quarto")
        .args([
            std::ffi::OsStr::new("create"),
            std::ffi::OsStr::new("project"),
            std::ffi::OsStr::new("website"),
            folder_name,
        ])
        .current_dir(work_dir)
        .output();

    let _ = Command::new("quarto")
        .args([std::ffi::OsStr::new("render"), folder_name])
        .current_dir(work_dir)
        .output();
}

// https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61
pub fn normalize_path(path: PathBuf) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}
