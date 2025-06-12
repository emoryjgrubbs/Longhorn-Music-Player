use crate::Song;
use crate::Link;
use std::{collections::{VecDeque}, vec};
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
    playing: bool,
}

#[allow(dead_code)]
impl<'q> Player<'q> {
    pub fn new(max_history: i32, initial_volume: f32, buffer_size: u32) -> Player<'q> {
        let mut new_player = Player { max_history, history_songs: VecDeque::new(), current_song: None, queue_songs: VecDeque::new(),
            buff_tracks: vec![], buff_pos: 0, player: Soloud::default().expect("failed to open soloud player"), playing: false,
        };

        // arbitrary number of slots in track buffer
        //  should be odd, with half being history, half being queue
        //  and half being the current track
        //  intention is for buffer to keep enough tracks loaded
        //  for responsive play back, while keeping memory ussage down
        for _ in 0..buffer_size {
            new_player.buff_tracks.push(audio::Wav::default());
        }

        new_player.player.set_global_volume(initial_volume);

        new_player
    }

    pub fn set_max(&mut self, new_max_history: i32) {
        self.max_history = new_max_history;
        self.cull_excess_history();
    }

    pub fn current_song(&self) -> Option<Song> {
        self.current_song.clone()
    }

    pub fn queue_after_current(&mut self, new_songs: Vec<Song<'q>>) {
        let mut vec_queue = Vec::from(self.queue_songs.clone());
        // splice in new_songs at start of queue
        vec_queue.splice(..0, new_songs);
        self.queue_songs = VecDeque::from(vec_queue);

        if let None = self.current_song {
            self.current_song = self.queue_songs.pop_front();
            if let Some(song) = &self.current_song {
                if let Link::Local(path) = song.link {
                    if let Err(_) = self.buff_tracks[self.buff_pos].load(&std::path::Path::new(path)) { todo!() }
                    self.player.play(&self.buff_tracks[self.buff_pos]);
                    if !self.playing {
                        self.player.set_pause_all(true);
                    }
                }
            }
        }

        // load possible new songs into buffer
        for load_offset in 0..(self.buff_tracks.len()/2) {
            if let Some(song) = self.queue_songs.get(load_offset) {
                // TODO these should probably become match statements
                //  when i implement other type of play handling
                if let Link::Local(path) = song.link {
                    let load_pos = (self.buff_pos + 1 + load_offset).rem_euclid(self.buff_tracks.len());
                    if let Err(_) = self.buff_tracks[load_pos].load(&std::path::Path::new(path)) { todo!() }
                }
            }
            // if there aren't more songs in the queue, no point in coninuing
            else { break }
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
            if let Some(song) = &self.current_song {
                if let Link::Local(path) = song.link {
                    if let Err(_) = self.buff_tracks[self.buff_pos].load(&std::path::Path::new(path)) { todo!() }
                    self.player.play(&self.buff_tracks[self.buff_pos]);
                    if !self.playing {
                        self.player.set_pause_all(true);
                    }
                }
            }
        }

        // load possible new songs into buffer
        for load_offset in splice_loc..(self.buff_tracks.len()/2) {
            if let Some(song) = self.queue_songs.get(load_offset) {
                // TODO these should probably become match statements
                //  when i implement other type of play handling
                if let Link::Local(path) = song.link {
                    let load_pos = (self.buff_pos + 1 + load_offset).rem_euclid(self.buff_tracks.len());
                    if let Err(_) = self.buff_tracks[load_pos].load(&std::path::Path::new(path)) { todo!() }
                }
            }
            // if there aren't more songs in the queue, no point in coninuing
            else { break }
        }
    }

    pub fn queue_after_queue(&mut self, new_songs: Vec<Song<'q>>) {
        let mut deque_new_songs = VecDeque::from(new_songs);
        let old_queue_len = self.queue_songs.len();
        self.queue_songs.append(&mut deque_new_songs);

        if let None = self.current_song {
            self.current_song = self.queue_songs.pop_front();
            if let Some(song) = &self.current_song {
                if let Link::Local(path) = song.link {
                    if let Err(_) = self.buff_tracks[self.buff_pos].load(&std::path::Path::new(path)) { todo!() }
                    self.player.play(&self.buff_tracks[self.buff_pos]);
                    if !self.playing {
                        self.player.set_pause_all(true);
                    }
                }
            }
        }

        // load possible new songs into buffer
        for load_offset in old_queue_len..(self.buff_tracks.len()/2) {
            if let Some(song) = self.queue_songs.get(load_offset) {
                // TODO these should probably become match statements
                //  when i implement other type of play handling
                if let Link::Local(path) = song.link {
                    let load_pos = (self.buff_pos + 1 + load_offset).rem_euclid(self.buff_tracks.len());
                    if let Err(_) = self.buff_tracks[load_pos].load(&std::path::Path::new(path)) { todo!() }
                }
            }
            // if there aren't more songs in the queue, no point in coninuing
            else { break }
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

            // advance buffer
            self.player.stop_all();
            self.buff_pos += 1;
            self.buff_pos = self.buff_pos.rem_euclid(self.buff_tracks.len());
            // resume playback
            self.player.play(&self.buff_tracks[self.buff_pos]);
            if !self.playing {
                self.player.set_pause_all(true);
            }
            // load new song to buffer
            let buff_max_future = self.buff_tracks.len() / 2;
            if let Some(song) = self.queue_songs.get(buff_max_future) {
                if let Link::Local(path) = song.link {
                    let load_pos = (self.buff_pos + buff_max_future).rem_euclid(self.buff_tracks.len());
                    if let Err(_) = self.buff_tracks[load_pos].load(&std::path::Path::new(path)) { todo!() }
                }
            }
        }
        else {
            self.player.stop_all();
        }
    }

    pub fn advance_album(&mut self) {
        self.player.stop_all();
        loop {
            // prevent advancing to a none song
            if let Some(new_song) = self.queue_songs.pop_front() {
                if let Some(old_song) = self.current_song.clone() {
                    // advance buffer
                    self.history_songs.push_back(old_song.clone());
                    self.buff_pos += 1;
                    self.buff_pos = self.buff_pos.rem_euclid(self.buff_tracks.len());
                    // load new song to buffer
                    let buff_max_future = self.buff_tracks.len() / 2;
                    if let Some(song) = self.queue_songs.get(buff_max_future) {
                        if let Link::Local(path) = song.link {
                            let load_pos = (self.buff_pos + buff_max_future).rem_euclid(self.buff_tracks.len());
                            if let Err(_) = self.buff_tracks[load_pos].load(&std::path::Path::new(path)) { todo!() }
                        }
                    }
                    // looking for the first instance of the song album changing
                    if new_song.album != old_song.album { 
                        self.current_song = Some(new_song);
                        self.player.play(&self.buff_tracks[self.buff_pos]);
                        if !self.playing {
                            self.player.set_pause_all(true);
                        }
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

    pub fn devance_song(&mut self) {
        // prevent devancing to a none song
        if let Some(new_song) = self.history_songs.pop_back() {
            if let Some(old_song) = self.current_song.clone() {
                self.queue_songs.push_front(old_song);
            }
            self.current_song = Some(new_song);

            // devance buffer
            self.player.stop_all();
            self.buff_pos -= 1;
            self.buff_pos = self.buff_pos.rem_euclid(self.buff_tracks.len());
            // resume playback
            self.player.play(&self.buff_tracks[self.buff_pos]);
            if !self.playing {
                self.player.set_pause_all(true);
            }
            // load new song to buffer
            let buff_max_past = self.buff_tracks.len() / 2;
            if let Some(song) = self.queue_songs.get(buff_max_past) {
                if let Link::Local(path) = song.link {
                    let load_pos = (self.buff_pos - buff_max_past).rem_euclid(self.buff_tracks.len());
                    if let Err(_) = self.buff_tracks[load_pos].load(&std::path::Path::new(path)) { todo!() }
                }
            }
        }
    }

    pub fn devance_album(&mut self) {
        if let Some(prev_song) = self.history_songs.back() {
            self.player.stop_all();
            // NOTE get the album of the previous song
            //  this means devancing on any song after the first
            //  in an album would restart it, but on the first song
            //  it would go to the previous album
            let devance_album = prev_song.album;

            loop {
                // prevent devancing to a none song
                if let Some(new_song) = self.history_songs.pop_back() {
                    if let Some(old_song) = self.current_song.clone() {
                        if new_song.album != devance_album {
                            self.history_songs.push_back(new_song);
                            // resume playback
                            self.player.play(&self.buff_tracks[self.buff_pos]);
                            if !self.playing {
                                self.player.set_pause_all(true);
                            }
                            break
                        }
                        self.queue_songs.push_front(old_song);
                    }
                    self.current_song = Some(new_song);
                    // devance buffer
                    self.buff_pos -= 1;
                    self.buff_pos = self.buff_pos.rem_euclid(self.buff_tracks.len());
                    // load new song to buffer
                    let buff_max_past = self.buff_tracks.len() / 2;
                    if let Some(song) = self.queue_songs.get(buff_max_past) {
                        if let Link::Local(path) = song.link {
                            let load_pos = (self.buff_pos - buff_max_past).rem_euclid(self.buff_tracks.len());
                            if let Err(_) = self.buff_tracks[load_pos].load(&std::path::Path::new(path)) { todo!() }
                        }
                    }
                }
                else {
                    // resume playback
                    self.player.play(&self.buff_tracks[self.buff_pos]);
                    if !self.playing {
                        self.player.set_pause_all(true);
                    }
                    break
                }
            }
        }

        // TODO resume playback
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

        // TODO resume playback
    }

    pub fn check_completed(&self) -> bool {
        if self.current_song.is_some() {
            if self.playing && self.player.voice_count() == 0 {
                return true;
            }
        }
        false
    }

    pub fn check_playing(&self) -> bool {
        self.playing.clone()
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

    // Interactions
    pub fn toggle_play(&mut self) {
        self.player.set_pause_all(self.playing);
        self.playing = !self.playing;
    }
    pub fn stop(&mut self) {
        // TODO idk what i want to do with this
        self.player.set_pause_all(true);
        //self.player.stop_all();
        self.playing = false;
    }

    pub fn volume_up(&mut self, increment: f32) {
        let new_volume = {
            let new_volume = self.player.global_volume() + increment;
            // NOTE 3 seems to be the highest volume that applies
            //  (this is just to make ux a bit better)
            if new_volume > 3.0 { 3.0 }
            else { new_volume }
        };
        self.player.set_global_volume(new_volume);
    }

    pub fn volume_down(&mut self, increment: f32) {
        let new_volume = {
            let new_volume = self.player.global_volume() - increment;
            if new_volume < 0.0 { 0.0 }
            else { new_volume }
        };
        self.player.set_global_volume(new_volume);
    }
}
