use std::collections::BTreeMap;
use std::path::PathBuf;
use std::io::Write;
use bstr::ByteSlice;

use clap::*;
use itertools::Itertools;

use method_chains::MethodChaining;

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), name = "method-chains")]
pub struct Options {
    #[clap(short = 'o', long = "output-path", parse(from_os_str))]
    pub output_path: PathBuf,

    #[clap(short = 'p', long = "project-dir", parse(from_os_str))]
    pub project_dir: PathBuf,
}

impl Options {
    pub fn output_path_as_str(&self) -> &str {
        self.output_path.as_os_str().to_str().unwrap()
    }
    pub fn project_dir_as_str(&self) -> &str {
        self.project_dir.as_os_str().to_str().unwrap()
    }
}

pub fn main() {
    let config = Options::parse();
    
    let project_dirs = std::fs::read_dir(&config.project_dir)
        .expect(&format!("Cannot read directory {}", config.project_dir_as_str()))
        .map(|entry | entry.unwrap())
        .filter(|entry| {
            entry.file_type().map_or(false, |file_type| file_type.is_dir())
        })
        .map(|entry| entry.file_name().to_str().unwrap().to_owned())
        .map(|file_name| {
            let mut path = config.project_dir.clone();
            path.push(file_name);
            path
        })
        .collect::<Vec<PathBuf>>();

    let total_projects = project_dirs.len();

    eprintln!("Found {} project directories in {}.", total_projects, config.project_dir_as_str());

    eprintln!("Creating CSV file at {} (if file exists, it will be overwritten)", config.output_path_as_str());

    let mut file = std::fs::File::create(config.output_path_as_str())
        .expect(&format!("Cannot create file {}", config.output_path_as_str()));
    writeln!(file, "project, chain length, frequency")
        .expect(&format!("Cannot write to file {}", config.output_path_as_str()));

    for (i, project_dir) in project_dirs.into_iter().enumerate() {

        let project_name = project_dir.file_name().unwrap().to_str().unwrap().to_owned();
        eprintln!("[{}/{}] processing project {}", i + 1, total_projects, project_name);

        let histogram = process_project_dir(i, total_projects, &project_name, &project_dir)
            .into_iter()
            .sorted()
            .rev()
            .map(|(chain, frequency)| {
                (project_name.clone(), chain, frequency)
            }).collect::<Vec<(String, usize, usize)>>();

        eprintln!("[{}/{}] appending {} items for project {} to CSV {}", i + 1, total_projects, 
                    histogram.len(), project_name, config.output_path_as_str());
        
        for (project, chain_length, frequency) in histogram {
            writeln!(file, "{}, {}, {}", project, chain_length, frequency)
                .expect(&format!("Cannot write to file {}", config.output_path_as_str()));
        }
    }

    eprintln!("Done.");
}

pub fn process_project_dir(i: usize, total_projects: usize, project_name: &str, project_dir: &PathBuf) -> BTreeMap<usize, usize> {
    let java_paths = method_chains::read_dir_all(project_dir)
        .into_iter()
        .filter(|path| {
            path.extension().map_or(false, |str| {
                str.to_str().unwrap() == "java"
            })
        })
        .collect::<Vec<PathBuf>>();

    eprintln!("[{}/{}] processing {} Java files for project {}", i + 1, total_projects, 
               java_paths.len(), project_name);

    
    java_paths.into_iter()
        .flat_map(|path| {
            std::fs::read(&path)                
                .expect(&format!("Cannot read file {:?}", &path))
                .to_str_lossy()
                .method_chain_counts()
        })
        .fold(BTreeMap::new(), |mut accumulator, chain_length| {
            *accumulator.entry(chain_length).or_insert(0) += 1;
            accumulator
        })
}

#[cfg(test)]
mod tests { 
    use std::iter::FromIterator;

    use crate::*;

    #[test]
    fn test_comment_removal() {
        let string = "// aaaaa\na/*   \n\n/**/*/b//c\nd";
        assert_eq!(remove_comments(string), "a*/bd");
    }

    #[test]
    fn test_tokenizer() {
        let string = "a(); bb(); c.dddd().e(); main {}";
        let tokens = vec![
            Token::String/*("a".to_owned())*/, Token::OpenParen, Token::CloseParen, Token::Punctuation/*(';')*/, 
            Token::String/*("bb".to_owned())*/, Token::OpenParen, Token::CloseParen, Token::Punctuation/*(';')*/, 
            Token::String/*("c".to_owned())*/, Token::Dot, 
            Token::String/*("dddd".to_owned())*/, Token::OpenParen, Token::CloseParen, Token::Dot, 
            Token::String/*("e".to_owned())*/, Token::OpenParen, Token::CloseParen, Token::Punctuation/*(';')*/, 
            Token::String/*("main".to_owned())*/, Token::Punctuation/*('{')*/, Token::Punctuation/*('}')*/,
        ];
        assert_eq!(tokenize(string), tokens);
    }


    #[test]
    fn test_chain1() {
        let tokens = vec![
            Token::String, Token::OpenParen, Token::CloseParen
        ];
        let histogram: BTreeMap<usize, usize> = BTreeMap::from_iter(vec![
            (1, 1)
        ].into_iter());
        assert_eq!(sloppy_method_chain_detection(tokens), histogram);
    }


    #[test]
    fn test_chain2() {
        let tokens = vec![
            Token::String, Token::OpenParen, Token::CloseParen, Token::Dot,
            Token::String, Token::OpenParen, Token::CloseParen
        ];
        let histogram: BTreeMap<usize, usize> = BTreeMap::from_iter(vec![
            (2, 1)
        ].into_iter());
        assert_eq!(sloppy_method_chain_detection(tokens), histogram);
    }

    #[test]
    fn test_chain3() {
        let tokens = vec![
            Token::String, Token::OpenParen, Token::CloseParen, Token::Dot,
            Token::String, Token::OpenParen, Token::CloseParen, Token::Dot,
            Token::String, Token::OpenParen, Token::CloseParen
        ];
        let histogram: BTreeMap<usize, usize> = BTreeMap::from_iter(vec![
            (3, 1)
        ].into_iter());
        assert_eq!(sloppy_method_chain_detection(tokens), histogram);
    }

    #[test]
    fn test_chain4() {
        let tokens = vec![
            Token::String, Token::OpenParen, Token::CloseParen, Token::Dot,
            Token::String, Token::Dot,
            Token::String, Token::OpenParen, Token::CloseParen
        ];
        let histogram: BTreeMap<usize, usize> = BTreeMap::from_iter(vec![
            (2, 1)
        ].into_iter());
        assert_eq!(sloppy_method_chain_detection(tokens), histogram);
    }

    #[test]
    fn test_chain5() {
        let tokens = vec![
            Token::String, Token::OpenParen, Token::CloseParen, Token::Dot,
            Token::String, Token::Dot,
            Token::String, Token::OpenParen, Token::CloseParen, Token::Punctuation/*(';')*/,
            Token::String, Token::OpenParen, Token::CloseParen
        ];
        let histogram: BTreeMap<usize, usize> = BTreeMap::from_iter(vec![
            (2, 1), (1, 1)
        ].into_iter());
        assert_eq!(sloppy_method_chain_detection(tokens), histogram);
    }


    #[test]
    fn test_chain6() {
        let tokens = vec![
            Token::String, Token::OpenParen, 
                           Token::String, Token::OpenParen, Token::CloseParen, Token::Punctuation/*(',')*/, // 1
                           Token::String, Token::OpenParen, Token::CloseParen,                              // 1
                           Token::CloseParen, Token::Dot,
            Token::String, Token::Dot,
            Token::String, Token::OpenParen, 
                           Token::String, Token::OpenParen, Token::CloseParen, Token::Punctuation/*(',')*/, // 1
                           Token::String, Token::OpenParen, Token::CloseParen,                              // 1
                           Token::CloseParen, Token::Punctuation/*(';')*/,                                  // 2
            Token::String, Token::OpenParen, Token::CloseParen                                              // 1
        ];
        let histogram: BTreeMap<usize, usize> = BTreeMap::from_iter(vec![
            (2, 1), (1, 5)
        ].into_iter());
        assert_eq!(sloppy_method_chain_detection(tokens), histogram);
    }

    #[test]
    fn test_chain7() {
        let tokens = vec![
            Token::String, Token::OpenParen, 
                           Token::String, Token::OpenParen, 
                                          Token::String, Token::OpenParen, Token::CloseParen, Token::Dot,
                                          Token::String, Token::Dot,
                                          Token::String, Token::OpenParen, Token::CloseParen, Token::Dot,
                                          Token::String, Token::Dot,
                                          Token::String, Token::OpenParen, Token::CloseParen,               // 3
                                          Token::CloseParen, Token::Punctuation/*(',')*/,                   // 1
                           Token::String, Token::OpenParen, Token::CloseParen,                              // 1
                           Token::CloseParen, Token::Dot,
            Token::String, Token::Dot,
            Token::String, Token::OpenParen, 
                           Token::String, Token::OpenParen, Token::CloseParen, Token::Punctuation/*(',')*/, // 1
                           Token::OpenBracket, 
                                Token::String, Token::OpenParen, Token::CloseParen, Token::Punctuation,     // 1
                                Token::String, Token::OpenParen, Token::CloseParen, Token::Punctuation,     // 1
                           Token::CloseBracket,
                           Token::String, Token::OpenParen, Token::CloseParen,                              // 1
                           Token::CloseParen, Token::Punctuation/*(';')*/,                                  // 2
            Token::String, Token::OpenParen, Token::CloseParen                                              // 1
        ];
        let histogram: BTreeMap<usize, usize> = BTreeMap::from_iter(vec![
            (3,1), (2, 1), (1, 7)
        ].into_iter());
        assert_eq!(sloppy_method_chain_detection(tokens), histogram);
    }
}