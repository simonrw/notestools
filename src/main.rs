use clap::{Parser, Subcommand};
use color_eyre::eyre;
use regex::RegexBuilder;
use std::{
    fmt::{self, Display},
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::{Path, PathBuf},
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Select { regex: String },
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let notes_file = args.path.unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Failed to get home directory")
            .join("notes.md")
    });

    match args.command {
        Command::Select { regex } => {
            let blocks = select_blocks(&regex, &notes_file)?;
            for block in blocks {
                println!("{block}");
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Block {
    title: String,
    content: Vec<String>,
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.title)?;
        for line in &self.content {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

struct BlockIterator<T>
where
    T: std::io::Read,
{
    lines: Lines<BufReader<T>>,
    buffer: Vec<String>,
}

impl<T> BlockIterator<T>
where
    T: std::io::Read,
{
    fn new(reader: BufReader<T>) -> Self {
        let lines = reader.lines();
        Self {
            lines,
            buffer: Vec::new(),
        }
    }
}

impl<T> Iterator for BlockIterator<T>
where
    T: std::io::Read,
{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        // Return buffered block if we have one
        let mut lines = if !self.buffer.is_empty() {
            std::mem::take(&mut self.buffer)
        } else {
            Vec::new()
        };

        // Find the next heading
        while let Some(Ok(line)) = self.lines.next() {
            if line.starts_with('#') {
                if lines.is_empty() {
                    // First heading - start collecting content
                    lines.push(line);
                } else {
                    // New heading while we have content - save for next iteration
                    self.buffer.push(line);
                    break;
                }
            } else if !lines.is_empty() {
                // Only collect content if we've seen a heading
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    lines.push(trimmed.to_string());
                }
            }
        }

        if lines.is_empty() {
            None
        } else {
            Some(Block {
                title: lines[0].clone(),
                content: lines[1..].to_vec(),
            })
        }
    }
}

fn select_blocks(regex_str: &str, notes_file: &Path) -> eyre::Result<impl Iterator<Item = Block>> {
    let blocks = all_blocks(notes_file)?;
    let regex = RegexBuilder::new(regex_str)
        .case_insensitive(true)
        .build()?;
    Ok(blocks.filter(move |block| regex.is_match(&block.title)))
}

fn all_blocks(notes_file: &Path) -> eyre::Result<impl Iterator<Item = Block>> {
    let file = File::open(notes_file)?;
    let reader = BufReader::new(file);
    let iterator = BlockIterator::new(reader);
    Ok(iterator)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn single_block() {
        let content_s = "\
# heading
content
";
        let f = Cursor::new(content_s);
        let mut iter = BlockIterator::new(BufReader::new(f));
        let block = iter.next().unwrap();
        assert_eq!(block.title, "# heading");
        assert_eq!(block.content, vec!["content"]);
        assert!(iter.next().is_none());
    }

    #[test]
    fn two_blocks() {
        let content_s = "\
# heading
content

## second heading
more content
### Something else
";
        let f = Cursor::new(content_s);
        let mut iter = BlockIterator::new(BufReader::new(f));
        let block = iter.next().unwrap();
        assert_eq!(block.title, "# heading");
        assert_eq!(block.content, vec!["content"]);
        let block = iter.next().unwrap();
        assert_eq!(block.title, "## second heading");
        assert_eq!(block.content, vec!["more content"]);
        let block = iter.next().unwrap();
        assert_eq!(block.title, "### Something else");
        assert_eq!(block.content, Vec::<String>::new());
        assert!(iter.next().is_none());
    }
}
