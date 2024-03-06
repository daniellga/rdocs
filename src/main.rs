use clap::Parser;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::{Component, Path, PathBuf},
    process::Command,
};

/// Create quarto docs from code comments. The command must be called in the package's main folder.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Relative path to the files.
    #[arg(long, num_args = 1..)]
    files: Vec<String>,

    /// Name of the documents folder.
    #[arg(long)]
    folder_name: String,

    /// Source button in docs will point to files in this github url.
    #[arg(long, default_value_t = String::from(""))]
    gh_url: String,

    /// Run R examples within code blocks.
    #[arg(long, default_value_t = false)]
    run_examples: bool,
}

fn main() {
    let args = Args::parse();
    let files: Vec<String> = args
        .files
        .iter()
        .map(|x| {
            normalize_path(Path::new(x))
                .to_str()
                .expect("file path not correct.")
                .to_string()
        })
        .collect();
    let folder_name = args.folder_name;
    let folder_name_hidden = ["_", folder_name.as_str()].join("");
    let gh_url = args.gh_url;
    let run_examples = args.run_examples;

    let mut hash: HashMap<String, Vec<String>> = HashMap::new();
    let mut examples: Vec<String> = Vec::new();

    generate_r_docs(
        files,
        gh_url.as_str(),
        run_examples,
        &mut hash,
        &mut examples,
    );
    output_file(hash, folder_name_hidden.as_str());
    quarto_process(folder_name.as_str(), folder_name_hidden.as_str());
    if run_examples {
        eval_examples(examples);
    }
}

// Currently it may give a bug if 2 methods impl for the same struct are on different files,
// depending on the order of the files on the list below. Try to reorder the vec in a way that
// if the code chunk contains "# Methods" it will be swapped to the vec's first position.
fn generate_r_docs(
    files: Vec<String>,
    gh_url: &str,
    run_examples: bool,
    hash: &mut HashMap<String, Vec<String>>,
    examples: &mut Vec<String>,
) {
    for file in &files {
        // Read the input file and filter to keep only lines starting with "###"
        let input_file = File::open(file).unwrap();
        let mut key = String::new();
        let mut last_line_was_comment = false;
        let mut skip_comment_chunk = false;
        let mut inside_code_chunk = false;

        // counts the line in a code chunk
        let mut counter: i32 = -1;
        for (line_counter, line) in BufReader::new(input_file).lines().flatten().enumerate() {
            let line_trimmed = line.trim_start();

            // skip non-commented lines.
            if let Some(stripped) = line_trimmed
                .strip_prefix("///")
                .or_else(|| line_trimmed.strip_prefix("###"))
            {
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
                    if run_examples {
                        let filtered_line_trimmed = filtered_line.trim_end();

                        if inside_code_chunk {
                            if filtered_line_trimmed == "```" {
                                examples.push("rm(list = ls())".to_string());
                                inside_code_chunk = false;
                            } else {
                                examples.push(filtered_line.clone());
                            }
                        } else if filtered_line_trimmed == "```r" {
                            inside_code_chunk = true;
                        }
                    }

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
                            + gh_url
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

fn output_file(hash: HashMap<String, Vec<String>>, folder_name_hidden: &str) {
    for (key, value) in hash {
        let key_lowercase = key.to_lowercase();

        // Construct the output file path as the input file path with a .qmd extension.
        let docs_file_path = Path::new(folder_name_hidden)
            .join(&key_lowercase)
            .with_extension("qmd");

        // Create the folder if it doesn't exist.
        std::fs::create_dir_all(folder_name_hidden).expect("directory could not be created");

        let title = format!("title: {}", key);
        let text = ["---", &title, "---"].join("\n");

        // Construct the final output text.
        let output_text = [text, value.join("\n")].join("\n\n");

        // Write the output text to the output file.
        let mut output_file = File::create(&docs_file_path).expect("could not create output_file");
        output_file
            .write_all(output_text.as_bytes())
            .expect("could not write to output_file");
    }
}

fn eval_examples(examples: Vec<String>) {
    // Construct the output text.
    let pkg_name_string = std::env::current_dir().unwrap();
    let pkg_name = pkg_name_string
        .as_path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let pkg_call = ["library", pkg_name].join(" ");
    let output_text = [pkg_call.as_str(), examples.join(";").as_str()].join(";");

    let _ = Command::new("Rscript")
        .args(["--vanilla", "-e", output_text.as_str()])
        .output();
}

// Create a quarto project and render.
fn quarto_process(folder_name: &str, folder_name_hidden: &str) {
    // If the directory is already used as a quarto project, it should error but the rest of the program is run anyway.
    let _ = Command::new("quarto")
        .args(["create", "project", "website", folder_name_hidden])
        .output();

    let output_path = Path::new("../").join(folder_name);
    let _ = Command::new("quarto")
        .args([
            "render",
            folder_name_hidden,
            "--output-dir",
            output_path.to_str().unwrap(),
        ])
        .output();
}

pub fn normalize_path(path: &Path) -> PathBuf {
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
