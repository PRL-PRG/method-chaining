use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::io::Write;

use clap::*;
use itertools::Itertools;

// fn is_keyword(string: &str) -> bool {
//     match string {
//         "abstract" => true,
//         "assert" => true,
//         "boolean" => true,
//         "break" => true,
//         "byte" => true,
//         "case" => true,
//         "catch" => true,
//         "char" => true,
//         "class" => true,
//         "const" => true,
//         "continue" => true,
//         "default" => true,
//         "do" => true,
//         "double" => true,
//         "else" => true,
//         "enum" => true,
//         "exports" => true,
//         "extends" => true,
//         "final" => true,
//         "finally" => true,
//         "float" => true,
//         "for" => true,
//         "if" => true,
//         "goto" => true,
//         "implements" => true,
//         "import" => true,
//         "instanceof" => true,
//         "int" => true,
//         "interface" => true,
//         "long" => true,
//         "module" => true,
//         "native" => true,
//         "new" => true,
//         "open" => true,
//         "opens" => true,
//         "package" => true,
//         "private" => true,
//         "protected" => true,
//         "provides" => true,
//         "public" => true,
//         "requires" => true,
//         "return" => true,
//         "short" => true,
//         "static" => true,
//         "strictfp" => true,
//         "super" => true,
//         "switch" => true,
//         "synchronized" => true,
//         "this" => true,
//         "throw" => true,
//         "throws" => true,
//         "to" => true,
//         "transient" => true,
//         "transitive" => true,
//         "try" => true,
//         "uses" => true,
//         _ => false,
//     }
// }

// fn is_identifier(token: &str) -> bool {
//     if token.is_empty() {
//         return false;
//     }

//     // simplifed, lose definition, allows numbers in first position
//     for character in token.chars() {
//         match character {
//             ch if ch.is_alphanumeric() => {} 
//             '$' | '_' => {}
//             _ => { return false }
//         }
//     }

//     return true;
// }

fn remove_comments(contents: &str) -> String {
    #[derive(Debug)]
    enum State { Basic, SlashFound, LineComment, BlockComment, StarFoundInComment }
    let mut state = State::Basic;
    let mut output = String::new();
    for character in contents.chars() {
        //println!("{:?} {:?}", state, character);
        match (&state, character) {
            (State::Basic, '/') => { state = State::SlashFound; }
            (State::Basic, any) => { output.push(any); }

            (State::SlashFound, '/') =>  { state = State::LineComment; }
            (State::SlashFound, '*') =>  { state = State::BlockComment; }
            (State::SlashFound, any) =>  { state = State::Basic; output.push('/'); output.push(any);  }

            (State::LineComment, '\n') => {state = State::Basic; }
            (State::LineComment, '\r') => {state = State::Basic; }
            (State::LineComment, _) => { /*ignore*/ }

            (State::BlockComment, '*') => {state = State::StarFoundInComment; }
            (State::BlockComment, _) => { /*ignore*/ }

            (State::StarFoundInComment, '/') => { state = State::Basic; }
            (State::StarFoundInComment, '*') => { /* ignore */ }
            (State::StarFoundInComment, _) => { state = State::BlockComment;  }
        }
    };
    output
}

#[derive(Clone, Debug,PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    Punctuation,//(char),
    String,//(String),
    Dot,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    // End,
}

fn tokenize(contents: &str) -> Vec<Token> {
    let mut token = String::new();
    let mut output = Vec::new();
    macro_rules! push { 
        (Token::String) => {
            if !token.is_empty() {
                output.push(Token::String/*(token.clone())*/);
            }
            token.clear();
        };        
        (Token::Punctuation($a:expr)) => {
            output.push(Token::Punctuation);
        };
        ($t:path) => {
            output.push($t);
        };
    }
    for character in contents.chars() {
        match character {
            // Whitespace
            ' ' | '\t' | '\n' | '\r' => {
                push!(Token::String);
            }
            // Punctuation
            '.' => {
                push!(Token::String);
                push!(Token::Dot);
            }
            '(' => {
                push!(Token::String);
                push!(Token::OpenParen);
            }
            ')' => {
                push!(Token::String);
                push!(Token::CloseParen);
            }
            '[' => {
                push!(Token::String);
                push!(Token::OpenBracket);
            }
            ']' => {
                push!(Token::String);
                push!(Token::CloseBracket);
            }
            '*' | '/' | '+' | '-' | '%' | 
            '\\' |
            ';' | ',' | '@' | ':' | '=' | 
            '{' | '}' | '<' | '>' | 
            '!' | '~' | '?' | '&' | '|' | '^' |
            '"' | '\'' => {
                push!(Token::String);
                push!(Token::Punctuation(character));
            }, 
            // Alphanumeric        
            _ => {
                token.push(character);
            },
        }
    }

    // push!(Token::End);
    output
}

#[allow(dead_code)]
fn sloppy_method_chain_detection(tokens: Vec<Token>) -> BTreeMap<usize, usize> {
    let mut tokens = VecDeque::from_iter(tokens.into_iter());
    let counters = sloppy_method_chain_detection_rec(&mut tokens);
    counters.into_iter().fold(BTreeMap::new(), |mut accumulator, chain_length| {
        *accumulator.entry(chain_length).or_insert(0) += 1;
        accumulator
    })
}

#[allow(unused_assignments)]
fn sloppy_method_chain_detection_rec(tokens: &mut VecDeque<Token>) -> Vec<usize> {

    #[derive(Clone, Debug,PartialEq, Eq, PartialOrd, Ord)]
    enum State { Start, Potential, ParenEnd, Chain }

    let mut counter: usize = 0;
    let mut state = State::Start;
    let mut counters: Vec<usize> = Vec::new();

    macro_rules! method_found {
        () => { counter += 1; }
    }
    macro_rules! chain_complete {
        () => { 
            if counter != 0 {
                counters.push(counter); 
                counter = 0; 
            }
        }
    }

    macro_rules! stop {
        () => {
            chain_complete!();
            return counters;
        }
    }

    macro_rules! recurse {
        () => {{
            let recursion_result = sloppy_method_chain_detection_rec(tokens);
            counters.extend(recursion_result.into_iter());
        }}
    }

    while let Some(token) = tokens.pop_front() {
        //println!("{:?} {:?} counter={}, counters={:?}", state, token, counter, counters);
        match (&state, token) {
            

            (State::Start, Token::OpenParen)        => { recurse!(); }
            (State::Start, Token::OpenBracket)      => { recurse!(); }
            (State::Start, Token::CloseParen)       => { stop!(); }
            (State::Start, Token::CloseBracket)     => { stop!(); }            
            (State::Start, Token::String)           => { state = State::Potential; }
            (State::Start, _)                       => { /*nothing*/ }

            (State::Potential, Token::OpenParen)    => { recurse!(); state = State::ParenEnd; method_found!() }
            (State::Potential, Token::OpenBracket)  => { recurse!(); state = State::ParenEnd; /*not a method*/ }
            (State::Potential, Token::CloseParen)   => { stop!(); }
            (State::Potential, Token::CloseBracket) => { stop!(); }
            (State::Potential, Token::Dot)          => { state = State::Chain; }
            (State::Potential, _)                   => { state = State::Start; chain_complete!(); }    

            (State::ParenEnd, Token::OpenParen)     => { recurse!(); state = State::Start;  }
            (State::ParenEnd, Token::OpenBracket)   => { recurse!(); state = State::Start;  }
            (State::ParenEnd, Token::CloseParen)    => { stop!(); }
            (State::ParenEnd, Token::CloseBracket)  => { stop!(); }
            (State::ParenEnd, Token::Dot)           => { state = State::Chain; }
            (State::ParenEnd, _)                    => { state = State::Start; chain_complete!(); }

            (State::Chain, Token::OpenParen)        => { recurse!(); state = State::Start; }
            (State::Chain, Token::OpenBracket)      => { recurse!(); state = State::Start; }
            (State::Chain, Token::CloseParen)       => { stop!(); }
            (State::Chain, Token::CloseBracket)     => { stop!(); }
            (State::Chain, Token::String)           => { state = State::Potential; }
            (State::Chain, _)                       => { state = State::Start; chain_complete!(); }
        }
        //println!(" => {:?} counter={}, counters={:?}", state, counter, counters);
    }
    chain_complete!();
    //println!("counter={}, counters={:?}", counter, counters);
    counters
}

trait MethodChaining {
    fn method_chain_counts(&self) -> Vec<usize>;
    fn method_chain_histogram(&self) -> BTreeMap<usize, usize> {
        self.method_chain_counts()
            .into_iter()
            .fold(BTreeMap::new(), |mut accumulator, chain_length| {
                *accumulator.entry(chain_length).or_insert(0) += 1;
                accumulator
            })
    }
}

impl MethodChaining for &str {
    fn method_chain_counts(&self) -> Vec<usize> {
        let clean = remove_comments(self);
        let tokens = tokenize(clean.as_str());
        let mut tokens = VecDeque::from_iter(tokens.into_iter());
        let counters = sloppy_method_chain_detection_rec(&mut tokens);
        counters
    }
}

impl MethodChaining for String {
    fn method_chain_counts(&self) -> Vec<usize> {
        self.as_str().method_chain_counts()
    }
}

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

fn read_dir_all(path: &PathBuf) -> Vec<PathBuf> {
    std::fs::read_dir(&path)
        .expect(&format!("Cannot read directory {:?}", path))
        .into_iter()
        .map(|entry| entry.unwrap())
        .flat_map(|entry| {
            if entry.file_type().unwrap().is_dir() {
                read_dir_all(&entry.path())
            } else {                
                vec![entry.path()]
            }
        }).collect()
}

pub fn process_project_dir(i: usize, total_projects: usize, project_name: &str, project_dir: &PathBuf) -> BTreeMap<usize, usize> {
    let java_paths = read_dir_all(project_dir)
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
            std::fs::read_to_string(&path)
                .expect(&format!("Cannot read file {:?}", &path))
                .method_chain_counts()
        })
        .fold(BTreeMap::new(), |mut accumulator, chain_length| {
            *accumulator.entry(chain_length).or_insert(0) += 1;
            accumulator
        })
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