use core::f64;

mod lexer;
use lexer::Lexer;

mod parser;
use parser::Parser;

#[derive(Debug)]
pub struct Settings {
    max_history: i32,
    library_paths: Vec<String>,
    top_songs_len: i32,
    top_albums_len: i32,
    top_artists_len: i32,
    top_decay: f64,
}
impl Settings {
    pub fn new() -> Settings {
        Settings { max_history: 50, library_paths: vec!["~/Music".to_string()], top_songs_len: 100, top_albums_len: 50, top_artists_len: 25, top_decay: 0.2 }
    }

    pub fn read_config(&mut self, optional_path: Option<&str>) {
        self.library_paths.clear();
        let mut file_path_stack = vec![];
        if let Some(path) = optional_path { file_path_stack.push(path.to_string()); }
        else { file_path_stack.push("./Longhorn.conf".to_string()); }
        // add way to set custom initial config file in future
        let mut config_errors = vec![];
        let empty = "";
        while let Some(path) = file_path_stack.pop() {
            let lexer_result = Lexer::process_file(&path);
            match lexer_result {
                Ok(lexical_units) => {
                    let len = path.len();
                    println!("\n{}\n{:-<len$}", path, empty);
                    let parser_result = Parser::process_tokens(self, &mut file_path_stack, lexical_units);
                    if let Ok(errors) = parser_result {
                        if errors.len() > 0 {
                            for error in errors {
                                let error = error.get_message() + " on Line " + &(error.get_line().to_string()) + " of " + &path;
                                config_errors.push(error);
                            }
                        }
                    }
                },
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            println!("No File Found At: {}\n", &path);
                            let error = "No File Found At ".to_string() + &path;
                            config_errors.push(error);
                        }
                        _ => {
                            println!("UNEXPECTED ERROR ENCOUNTERED!\n");
                            let error = "UNEXPECTED ERROR ENCOUNTERED IN ".to_string() + &path;
                            config_errors.push(error);
                        }
                    }
                },
            }
        }
        println!("\nConfig Errors\n{:-<13}", empty);
        for error in config_errors {
            println!("{}", error);
        }
    }

    pub fn history_len(&self) -> i32 {
        self.max_history.clone()
    }

    pub fn library_paths(&self) -> Vec<String> {
        self.library_paths.clone()
    }

    pub fn top_songs_len(&self) -> i32 {
        self.top_songs_len.clone()
    }
    pub fn top_albums_len(&self) -> i32 {
        self.top_albums_len.clone()
    }
    pub fn top_artists_len(&self) -> i32 {
        self.top_artists_len.clone()
    }
    pub fn top_decay(&self) -> f64 {
        self.top_decay.clone()
    }
}


