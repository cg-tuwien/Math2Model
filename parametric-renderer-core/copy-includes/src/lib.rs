use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

const AUTOGEN_PREFIX: &str = "//// AUTOGEN";
const AUTOGEN_SUFFIX: &str = "//// END OF AUTOGEN";

pub fn copy_includes(path: impl AsRef<Path>) -> std::io::Result<()> {
    println!(
        "Copying includes in file: {}",
        path.as_ref().to_string_lossy()
    );
    let contents = fs::read_to_string(path.as_ref())?;
    let includes = find_includes(path.as_ref().to_owned(), &contents);

    // Iterate over the file backwards, delete the old includes and insert the new ones
    let mut lines: Vec<String> = contents.lines().map(|v| v.to_string()).collect();
    for include in includes.iter().rev() {
        delete_old_include(&mut lines, include.line_number);
        insert_new_include(&mut lines, include);
    }

    let new_contents = lines.join("\n");
    fs::write(path, new_contents)?;

    Ok(())
}

fn insert_new_include(lines: &mut Vec<String>, include: &IncludeStatement) {
    let include_contents = match fs::read_to_string(include.full_path()) {
        Ok(v) => v,
        Err(e) => panic!(
            "Failed to read include file: {} {}",
            include.full_path().to_string_lossy(),
            e
        ),
    };
    // Here we could add more text to the include contents
    let hash = compute_hash(&include_contents.lines().collect::<Vec<_>>());

    let next_line = include.line_number + 1;
    lines.splice(
        next_line..next_line,
        vec![
            format!("{} {}", AUTOGEN_PREFIX, hash),
            include_contents,
            AUTOGEN_SUFFIX.to_string(),
        ],
    );
}

fn delete_old_include<T: AsRef<str>>(lines: &mut Vec<T>, line_number: usize) {
    let next_line = match lines.get(line_number + 1) {
        Some(v) => v.as_ref(),
        None => return,
    };

    if let Some(next_line_with_prefix) = next_line.strip_prefix(AUTOGEN_PREFIX) {
        let sha2_hash = next_line_with_prefix.trim_start();

        let prefix_index = line_number + 1;
        let suffix_index = lines
            .iter()
            .skip(prefix_index)
            .position(|v| v.as_ref().starts_with(AUTOGEN_SUFFIX))
            .expect("Expected to find end of autogen block")
            + prefix_index;

        let existing_lines = &lines[(prefix_index + 1)..suffix_index];
        let hash = compute_hash(existing_lines);
        if hash != sha2_hash {
            eprintln!("Existing lines:");
            for line in existing_lines {
                eprintln!("{}", line.as_ref());
            }
            panic!(
                "Hash mismatch, someone edited the autogen block! Delete this block and run again."
            );
        }

        lines.drain(prefix_index..=suffix_index);
    }
}

fn compute_hash<T: AsRef<str>>(lines: &[T]) -> String {
    let mut hasher = Sha256::new();
    for line in lines {
        let line_without_spaces = line
            .as_ref()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        hasher.update(line_without_spaces.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

struct IncludeStatement {
    import_path: PathBuf,
    relative_to: PathBuf,
    line_number: usize,
}

impl IncludeStatement {
    fn full_path(&self) -> PathBuf {
        self.relative_to.parent().unwrap().join(&self.import_path)
    }
}

fn find_includes(file_path: PathBuf, contents: &str) -> Vec<IncludeStatement> {
    let mut includes = vec![];
    let line_stream = contents.lines().enumerate();
    for (line_number, line) in line_stream {
        if let Some(include) = try_parse_include(file_path.clone(), line, line_number) {
            includes.push(include);
        }
    }
    includes
}

fn try_parse_include(
    file_path: PathBuf,
    line: &str,
    line_number: usize,
) -> Option<IncludeStatement> {
    const INCLUDE_PREFIX: &str = "////#include";
    if !line.starts_with(INCLUDE_PREFIX) {
        return None;
    }

    let mut path_stream = line[INCLUDE_PREFIX.len()..].trim_start().chars();
    assert_eq!(path_stream.next(), Some('"'));
    let mut import_path = String::new();
    loop {
        let c = match path_stream.next() {
            Some('\\') => {
                let c = path_stream.next();
                match c {
                    Some(c) => c,
                    None => panic!("Unexpected end of line"),
                }
            }
            Some(c) => c,
            None => panic!("Unexpected end of line"),
        };
        if c == '"' {
            break;
        }
        import_path.push(c);
    }

    Some(IncludeStatement {
        import_path: PathBuf::from(import_path),
        relative_to: file_path,
        line_number,
    })
}
