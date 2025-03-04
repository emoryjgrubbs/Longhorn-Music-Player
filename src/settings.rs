pub struct Reader<'r> {
    max_history: i32,
    library_paths: Vec<&'r str>,
    top_songs_len: i32,
    top_albums_len: i32,
    top_artists_len: i32,
    top_decay: i32,
}

impl<'r> Reader<'_> {
    pub fn new() -> Reader<'r> {
        Reader { max_history: 50, library_paths: vec!["~/Music"], top_songs_len: 100, top_albums_len: 50, top_artists_len: 25, top_decay: 3 }
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
    pub fn top_decay(&self) -> i32 {
        self.top_decay.clone()
    }


    pub fn read(&mut self) {
        self.max_history = -1;
        // parse jsonc? settings file "src/config.jsonc"
    }
}


