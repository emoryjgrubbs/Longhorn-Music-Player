use core::f64;

mod lexer;
use lexer::Lexer;

mod parser;
use parser::Parser;

pub struct Settings<'r> {
    max_history: i32,
    library_paths: Vec<&'r str>,
    top_songs_len: i32,
    top_albums_len: i32,
    top_artists_len: i32,
    top_decay: f64,
}
impl<'r> Settings<'_> {
    pub fn new() -> Settings<'r> {
        Settings { max_history: 50, library_paths: vec!["~/Music"], top_songs_len: 100, top_albums_len: 50, top_artists_len: 25, top_decay: 0.2 }
    }

    pub fn read_config(&mut self, optional_path: Option<&str>) {
        let mut file_path_stack = vec![];
        if let Some(path) = optional_path { file_path_stack.push(path); }
        else { file_path_stack.push("./Longhorn.conf"); }
        // add way to set custom initial config file in future
        while let Some(path) = file_path_stack.pop() {
            let lexer_result = Lexer::process_file(path);
            if let Ok(mut lexical_units) = lexer_result {
                while let Some(lexical_unit) = lexical_units.pop_front() {
                    println!("{:?}", lexical_unit);
                }
            }
            // syntax error in 'path'
            else { }
        }

    }

    pub fn history_len(&self) -> i32 {
        self.max_history.clone()
    }

    pub fn library_paths(&self) -> Vec<&str> {
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


