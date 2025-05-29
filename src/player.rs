use crate::Song;
use std::{collections::{vec_deque, VecDeque}, vec};
use soloud::*;

#[derive(Debug)]
pub struct Player<'a> {
    max_history: i32,
    history_songs: VecDeque<Song<'a>>,
    current_song: Option<Song<'a>>,
    queue_songs: VecDeque<Song<'a>>,
    buff_tracks: Vec<soloud::audio::Wav>,
    buff_pos: usize,
    player: Soloud,
}

#[allow(dead_code)]
impl<'q> Player<'q> {
    pub fn new(max_history: i32) -> Player<'q> {
        let mut new_player = Player { max_history, history_songs: VecDeque::new(), current_song: None, queue_songs: VecDeque::new(),
            buff_tracks: vec![], buff_pos: 0, player: Soloud::default().expect("failed to open soloud player")
        };

        // arbitrary number of slots in track buffer
        //  should be odd, with half being history, half being queue
        //  and half being the current track
        //  intention is for buffer to keep enough tracks loaded
        //  for responsive play back, while keeping memory ussage down
        for _ in 0..3 {
            new_player.buff_tracks.push(audio::Wav::default());
        }

        new_player
    }

    // TODO remove this
    pub fn test_play(&mut self) {
        let test_tracks = vec!["./test-track-four.flac", "./test-track-five.flac", "./test-track-one.flac", "./test-track-two.wav", "./test-track-three.mp3"];
        for track in test_tracks {
            print!("playing {} ", track);
            // TODO have handling incase a track disapears
            // loads next track to buffer
            let up_ccoming_pos = (self.buff_pos + 1) % self.buff_tracks.len();
            if let Err(msg) = self.buff_tracks[up_ccoming_pos].load(&std::path::Path::new(track)) {
                // TODO handle sl errors properly
                println!("Encountered error: {}...", msg)
            }

            // lets current track play
            //  not fully seamless..
            while self.player.voice_count() > 0 {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            
            self.buff_pos += 1;
            self.buff_pos %= self.buff_tracks.len();

            println!("at {} in buff", self.buff_pos);
            // starts new track playing
            self.player.play(&self.buff_tracks[self.buff_pos]);
        }

        // lets last track finish
        while self.player.voice_count() > 0 {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    pub fn set_max(&mut self, new_max_history: i32) {
        self.max_history = new_max_history;
        self.cull_excess_history();
    }

    pub fn queue_after_current(&mut self, new_songs: Vec<Song<'q>>) {
        let mut vec_queue = Vec::from(self.queue_songs.clone());
        // splice in new_songs at start of queue
        vec_queue.splice(..0, new_songs);
        self.queue_songs = VecDeque::from(vec_queue);

        if let None = self.current_song {
            self.current_song = self.queue_songs.pop_front();
        }
    }

    pub fn queue_after_album(&mut self, new_songs: Vec<Song<'q>>) {
        // find location to splice new songs
        let splice_loc = {
            let mut i = 0;

            if let Some(curr_song) = self.current_song.clone() {
                let curr_album = curr_song.album;
                for song in self.queue_songs.clone() {
                    if song.album != curr_album { break }
                    i += 1;
                }
            }

            i
        };

        // splice new songs after album
        let mut vec_history = Vec::from(self.queue_songs.clone());
        vec_history.splice(splice_loc..splice_loc, new_songs);
        self.queue_songs = VecDeque::from(vec_history);

        if let None = self.current_song {
            self.current_song = self.queue_songs.pop_front();
        }
    }

    pub fn queue_after_queue(&mut self, new_songs: Vec<Song<'q>>) {
        let mut deque_new_songs = VecDeque::from(new_songs);
        self.queue_songs.append(&mut deque_new_songs);

        if let None = self.current_song {
            self.current_song = self.queue_songs.pop_front();
        }
    }

    pub fn advance_song(&mut self) {
        // prevent advancing to a none song
        if let Some(new_song) = self.queue_songs.pop_front() {
            if let Some(old_song) = self.current_song.clone() {
                self.history_songs.push_back(old_song);
            }
            self.current_song = Some(new_song);
            self.cull_excess_history();
        }
    }

    pub fn advance_album(&mut self) {
        loop {
            // prevent advancing to a none song
            if let Some(new_song) = self.queue_songs.pop_front() {
                if let Some(old_song) = self.current_song.clone() {
                    // looking for the first instance of the song album changing
                    self.history_songs.push_back(old_song.clone());
                    if new_song.album != old_song.album { 
                        self.current_song = Some(new_song);
                        self.cull_excess_history();
                        break
                    }
                }
                self.current_song = Some(new_song);
            }
            else {
                self.cull_excess_history();
                break
            }
        }
    }

    pub fn deadvance_song(&mut self) {
        // prevent deadvancing to a none song
        if let Some(new_song) = self.history_songs.pop_back() {
            if let Some(old_song) = self.current_song.clone() {
                self.queue_songs.push_front(old_song);
            }
            self.current_song = Some(new_song);
        }
        // TODO restart play if at breakpoint
    }

    pub fn deadvance_album(&mut self) {
        if let Some(prev_song) = self.history_songs.back() {
            // NOTE get the album of the previous song
            //  this means deadvancing on any song after the first
            //  in an album would restart it, but on the first song
            //  it would go to the previous album
            let deadvance_album = prev_song.album;

            loop {
                // prevent deadvancing to a none song
                if let Some(new_song) = self.history_songs.pop_back() {
                    if let Some(old_song) = self.current_song.clone() {
                        if new_song.album != deadvance_album {
                            self.history_songs.push_back(new_song);
                            break
                        }
                        self.queue_songs.push_front(old_song);
                    }
                    self.current_song = Some(new_song);
                }
                else { break }
            }
        }
    }

    pub fn jump_to_song(&mut self, relative_pos: i32) {
        if relative_pos < 0 {
            if let Some(song) = self.current_song.clone() {
                // move the current song
                self.queue_songs.push_front(song);

                // drain the portion of the history being moved
                let index;
                if relative_pos.abs() < self.queue_songs.len().try_into().unwrap() { index = self.history_songs.len() - usize::try_from(relative_pos.abs()).unwrap(); }
                else { index = 0; }
                let skip: Vec<Song> = self.history_songs.drain(index..).collect();

                // add the drained history to the queue
                let mut vec_queue = Vec::from(self.queue_songs.clone());
                // splice in new_songs at start of queue
                vec_queue.splice(..0, skip);
                self.queue_songs = VecDeque::from(vec_queue);

                // get the new current song
                self.current_song = self.queue_songs.pop_front();
            }
        }
        else if relative_pos > 0 {
            if let Some(song) = self.current_song.clone() {
                // move the current song
                self.queue_songs.push_front(song);

                // drain the portion of the queue being moved
                let index;
                if relative_pos < self.queue_songs.len().try_into().unwrap() { index = usize::try_from(relative_pos).unwrap(); }
                else { index = self.queue_songs.len() - 1; }
                let mut skip: VecDeque<Song> = self.queue_songs.drain(..index+1).collect();

                // add the drained queue to the history 
                self.history_songs.append(&mut skip);

                // get the new current song
                self.current_song = self.history_songs.pop_back();
                self.cull_excess_history();
            }
        }
        /*
        else {
            // TODO restart song?
        }
        */
    }

    fn cull_excess_history(&mut self) {
        // negative max_history (specifically -1) indicates no len constraint
        if self.max_history >= 0 && self.history_songs.len() > self.max_history.try_into().unwrap() {
            let overage = self.history_songs.len() - usize::try_from(self.max_history).unwrap();
            let mut vec_history = Vec::from(self.history_songs.clone());
            // splice out history overage
            vec_history.splice(..overage, []);
            self.history_songs = VecDeque::from(vec_history);
        }
    }
}
