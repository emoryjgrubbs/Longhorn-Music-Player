pub struct Reader<'r> {
    max_history: i32,
    library_paths: Vec<&'r str>,
}

impl<'r> Reader<'_> {
    pub fn new() -> Reader<'r> {
        Reader { max_history: 50, library_paths: vec!["~/Music"] }
    }



    pub fn history_len(&self) -> i32 {
        self.max_history.clone()
    }

    pub fn library_paths(&self) -> Vec<&str> {
        self.library_paths.clone()
    }


    pub fn read(&mut self) {
        self.max_history = -1;
        // parse jsonc? settings file "src/config.jsonc"
    }
}


