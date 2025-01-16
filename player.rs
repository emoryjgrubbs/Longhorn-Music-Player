use std::ptr::NonNull;
use std::boxed::Box;
use Song;

#[derive(Debug, Clone)]
pub struct Queue<'a> {
    max_history: i32,
    len_history: i32,
    len_queue: i32,
    start: Option<NonNull<Element<'a>>>,
    current: Option<NonNull<Element<'a>>>,
    end: Option<NonNull<Element<'a>>>,
}

impl<'q> Queue<'q> {
    pub fn new(max_history: i32) -> Queue<'q> {
        Queue { max_history: max_history, len_history: 0, len_queue: 0, start: None, current: None, end: None }
    }



    pub fn set_max(&mut self, new_max_history: i32) {
        self.max_history = new_max_history;
        self.cull_excess_history();
    }



    pub fn start_song(&self) -> Option<&Song> {
        unsafe { self.start.as_ref().map(|element| &element.as_ref().song) }
    }

    pub fn current_song(&self) -> Option<&Song> {
        unsafe { self.current.as_ref().map(|element| &element.as_ref().song) }
    }

    pub fn end_song(&self) -> Option<&Song> {
        unsafe { self.end.as_ref().map(|element| &element.as_ref().song) }
    }

    pub fn relative_song(&self, relative_pos: i32) -> Option<&Song> {
        if relative_pos == 0 { self.current_song() }
        else if (relative_pos < 0) && (relative_pos.abs() <= self.len_history) { unsafe { self.history_song(relative_pos.abs()) } }
        else if (relative_pos > 0) && (relative_pos < { self.len_queue - self.len_history }) { unsafe { self.queue_song(relative_pos) } }
        else { None }
    }
    unsafe fn history_song(&self, relative_pos: i32) -> Option<&Song> {
        let mut relative_element = self.current;
        for _ in 0..relative_pos {
            match relative_element {
                None => { panic!("Queue Element has Gone Missing"); },
                Some(_) => { relative_element = relative_element.as_ref().map(|element| element.as_ref().prev)?; },
            }
        }
        match relative_element {
            None => { panic!("Queue Element has Gone Missing"); },
            Some(_) => { relative_element.as_ref().map(|element| &element.as_ref().song) },
        }
    }
    unsafe fn queue_song(&self, relative_pos: i32) -> Option<&Song> {
        let mut relative_element = self.current;
        for _ in 0..relative_pos {
            match relative_element {
                None => { panic!("Queue Element has Gone Missing"); },
                Some(_) => { relative_element = relative_element.as_ref().map(|element| element.as_ref().next)?; },
            }
        }
        match relative_element {
            None => { panic!("Queue Element has Gone Missing"); },
            Some(_) => { relative_element.as_ref().map(|element| &element.as_ref().song) },
        }
    }

    fn end_of_current_album(&self) -> Option<NonNull<Element>> {
        let option_play_album = unsafe { self.current.as_ref().map(|element| &element.as_ref().song.album) };
        match option_play_album {
            None => { None },
            Some(play_album) => unsafe {
                let mut last_element_of_album = self.current;
                while let Some(element_album) = last_element_of_album.as_ref().map(|element| element.as_ref().next)?.as_ref().map(|element| &element.as_ref().song.album) {
                    if play_album != element_album { break }
                    last_element_of_album = last_element_of_album.as_ref().map(|element| element.as_ref().next)?;
                }
                
                last_element_of_album
            },
        }
    }



    pub fn add_song_after_current(&mut self, new_song: Song<'q>) {
        let new_element = Box::new(Element::new(new_song));
        let new_element_ptr = NonNull::from(Box::leak(new_element));
        unsafe {
            self.add_element_block_after_current(new_element_ptr, new_element_ptr, 1);
        }
    }
    pub fn add_song_block_after_current(&mut self, mut new_songs: Vec<Song<'q>>) {
        new_songs.reverse();
        let len_block = new_songs.len() as i32;
        let start_block_song = new_songs.pop().expect("new_songs vec should always contain at least 1 song");
        let start_block_element = Box::new(Element::new(start_block_song));
        let start_block_element_ptr = NonNull::from(Box::leak(start_block_element));
        let end_block_element_ptr = {
            let mut current_block_element_ptr = start_block_element_ptr;
            let mut option_current_song = new_songs.pop();
            while let Some(current_song) = option_current_song {
                let previous_block_element_ptr = current_block_element_ptr;
                let current_block_element = Box::new(Element::new(current_song));
                current_block_element_ptr = NonNull::from(Box::leak(current_block_element));
                unsafe {
                    (*previous_block_element_ptr.as_ptr()).next = Some(current_block_element_ptr);
                    (*current_block_element_ptr.as_ptr()).prev = Some(previous_block_element_ptr);
                }
                option_current_song = new_songs.pop();
            }
            current_block_element_ptr
        };
        unsafe {
            self.add_element_block_after_current(start_block_element_ptr, end_block_element_ptr, len_block);
        }
    }
    unsafe fn add_element_block_after_current(&mut self, start_block_element: NonNull<Element<'q>>, end_block_element: NonNull<Element<'q>>, len_block: i32) {
        unsafe {
            match self.current {
                None => {
                    let start_block_element = Some(start_block_element);
                    let end_block_element = Some(end_block_element);
                    self.start = start_block_element;
                    self.current = start_block_element;
                    self.end = end_block_element;
                },
                Some(current) => {
                    let self_next = (*current.as_ptr()).next;
                    (*start_block_element.as_ptr()).prev = self.current;
                    match self_next {
                        None => {
                            let end_block_element = Some(end_block_element);
                            self.end = end_block_element;
                        }
                        Some(next) => {
                            (*end_block_element.as_ptr()).next = self_next;
                            let end_block_element = Some(end_block_element);
                            (*next.as_ptr()).prev = end_block_element;
                        }
                    }
                    let start_block_element = Some(start_block_element);
                    (*current.as_ptr()).next = start_block_element;
                },
            }

            self.len_queue += len_block;
        }
    }

    pub fn add_song_after_album(&mut self, new_song: Song<'q>) {
        let new_element = Box::new(Element::new(new_song));
        let new_element_ptr = NonNull::from(Box::leak(new_element));
        unsafe {
            self.add_element_block_after_album(new_element_ptr, new_element_ptr, 1);
        }
    }
    pub fn add_song_block_after_album(&mut self, mut new_songs: Vec<Song<'q>>) {
        new_songs.reverse();
        let len_block = new_songs.len() as i32;
        let start_block_song = new_songs.pop().expect("new_songs vec should always contain at least 1 song");
        let start_block_element = Box::new(Element::new(start_block_song));
        let start_block_element_ptr = NonNull::from(Box::leak(start_block_element));
        let end_block_element_ptr = {
            let mut current_block_element_ptr = start_block_element_ptr;
            let mut option_current_song = new_songs.pop();
            while let Some(current_song) = option_current_song {
                let previous_block_element_ptr = current_block_element_ptr;
                let current_block_element = Box::new(Element::new(current_song));
                current_block_element_ptr = NonNull::from(Box::leak(current_block_element));
                unsafe {
                    (*previous_block_element_ptr.as_ptr()).next = Some(current_block_element_ptr);
                    (*current_block_element_ptr.as_ptr()).prev = Some(previous_block_element_ptr);
                }
                option_current_song = new_songs.pop();
            }
            current_block_element_ptr
        };
        unsafe {
            self.add_element_block_after_album(start_block_element_ptr, end_block_element_ptr, len_block);
        }
    }
    unsafe fn add_element_block_after_album(&mut self, start_block_element: NonNull<Element<'q>>, end_block_element: NonNull<Element<'q>>, len_block: i32) {
        unsafe {
            let self_current_album = self.end_of_current_album();
            match self_current_album {
                None => {
                    let start_block_element = Some(start_block_element);
                    let end_block_element = Some(end_block_element);
                    self.start = start_block_element;
                    self.current = start_block_element;
                    self.end = end_block_element;
                },
                Some(current_album) => {
                    let self_next_album = (*current_album.as_ptr()).next;
                    (*start_block_element.as_ptr()).prev = self_current_album;
                    match self_next_album {
                        None => {
                            let start_block_element = Some(start_block_element);
                            (*current_album.as_ptr()).next = start_block_element;
                            let end_block_element = Some(end_block_element);
                            self.end = end_block_element;
                        }
                        Some(next_album) => {
                            let start_block_element = Some(start_block_element);
                            (*current_album.as_ptr()).next = start_block_element;
                            (*end_block_element.as_ptr()).next = self_next_album;
                            let end_block_element = Some(end_block_element);
                            (*next_album.as_ptr()).prev = end_block_element;
                        }
                    }
                },
            }

            self.len_queue += len_block;
        }
    }

    pub fn add_song_after_queue(&mut self, new_song: Song<'q>) {
        let new_element = Box::new(Element::new(new_song));
        let new_element_ptr = NonNull::from(Box::leak(new_element));
        unsafe {
            self.add_element_block_after_queue(new_element_ptr, new_element_ptr, 1);
        }
    }
    pub fn add_song_block_after_queue(&mut self, mut new_songs: Vec<Song<'q>>) {
        new_songs.reverse();
        let len_block = new_songs.len() as i32;
        let start_block_song = new_songs.pop().expect("new_songs vec should always contain at least 1 song");
        let start_block_element = Box::new(Element::new(start_block_song));
        let start_block_element_ptr = NonNull::from(Box::leak(start_block_element));
        let end_block_element_ptr = {
            let mut current_block_element_ptr = start_block_element_ptr;
            let mut option_current_song = new_songs.pop();
            while let Some(current_song) = option_current_song {
                let previous_block_element_ptr = current_block_element_ptr;
                let current_block_element = Box::new(Element::new(current_song));
                current_block_element_ptr = NonNull::from(Box::leak(current_block_element));
                unsafe {
                    (*previous_block_element_ptr.as_ptr()).next = Some(current_block_element_ptr);
                    (*current_block_element_ptr.as_ptr()).prev = Some(previous_block_element_ptr);
                }
                option_current_song = new_songs.pop();
            }
            current_block_element_ptr
        };
        unsafe {
            self.add_element_block_after_queue(start_block_element_ptr, end_block_element_ptr, len_block);
        }
    }
    unsafe fn add_element_block_after_queue(&mut self, start_block_element: NonNull<Element<'q>>, end_block_element: NonNull<Element<'q>>, len_block: i32) {
        unsafe {
            (*start_block_element.as_ptr()).prev = self.end;
            let start_block_element = Some(start_block_element);
            let end_block_element = Some(end_block_element);

            match self.end {
                None => {
                    self.start = start_block_element;
                    self.current = start_block_element;
                },
                Some(end) => {
                    (*end.as_ptr()).next = start_block_element;
                },
            }

            self.end = end_block_element;
            self.len_queue += len_block;
        }
    }

    pub fn advance_song(&mut self) {
        if let Some(current) = self.current {
            let self_next;
            unsafe { self_next = (*current.as_ptr()).next; }
            if let Some(_) = self_next {
                self.current = self_next;
                self.len_history += 1;
            }
        }
        self.cull_excess_history();
    }
    pub fn advance_album(&mut self) {
        let option_play_album = unsafe { self.current.as_ref().map(|element| &element.as_ref().song.album) };
        if let Some(play_album) = option_play_album {
            let mut last_element_of_album = self.current;
            unsafe {
                while let Some(element_album) = last_element_of_album.as_ref().map(|element| element.as_ref().next).expect("last iteration's next disapeared").as_ref().map(|element| &element.as_ref().song.album) {
                    if play_album != element_album { break }
                    last_element_of_album = last_element_of_album.as_ref().map(|element| element.as_ref().next).expect("next element has disapeared");
                    self.len_history += 1;
                }
            
                match last_element_of_album.as_ref().map(|element| element.as_ref().next).expect("optional next has disapeared") {
                    None => { self.current = last_element_of_album; },
                    Some(_) => { 
                        self.current = last_element_of_album.as_ref().map(|element| element.as_ref().next).expect("next element has disapeared"); 
                        self.len_history += 1;
                    }
                }
            }
        }
        self.cull_excess_history();
    }

    pub fn deadvance_song(&mut self) {
        if let Some(current) = self.current {
            let self_prev;
            unsafe { self_prev = (*current.as_ptr()).prev; }
            if let Some(_) = self_prev {
                self.current = self_prev;
                self.len_history -= 1;
            }
        }
        // restart play
    }

    pub fn deadvance_album(&mut self) {
        if let Some(self_prev) = unsafe { self.current.as_ref().map(|element| &element.as_ref().prev) } {
            let option_prev_album = unsafe { self_prev.as_ref().map(|element| &element.as_ref().song.album) };
            if let Some(prev_album) = option_prev_album {
                let mut first_element_of_album = self.current;
                unsafe {
                    while let Some(element_album) = first_element_of_album.as_ref().map(|element| element.as_ref().prev).expect("last iteration's prev disapeared").as_ref().map(|element| &element.as_ref().song.album) {
                        if prev_album != element_album { break }
                        first_element_of_album = first_element_of_album.as_ref().map(|element| element.as_ref().prev).expect("prev element has disapeared");
                        self.len_history -= 1;
                    }
                
                    self.current = first_element_of_album;
                }
            }
        }
    }


    fn cull_excess_history(&mut self) {
        if self.max_history >= 0 && self.current.is_some() {
            let history_overage: i32 = self.len_history - self.max_history;
            for _ in 0..history_overage {
                match self.start {
                    None => { panic!("history element has disapeared, no self.start to alter"); },
                    Some(start) => unsafe {
                        let self_next = (*start.as_ptr()).next;
                        match self_next {
                            None => { panic!("history element has disapeared, cannot advance pointer"); },
                            Some(_) => {
                                self.start = self_next;
                                self.len_queue -= 1;
                                self.len_history -= 1;
                            },
                        }
                    },
                }
            }
            match self.start {
                None => { panic!("history element has disapeared, cannot set start.prev to None"); },
                Some(start) => unsafe { (*start.as_ptr()).prev = None; },
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Element<'a> {
    prev: Option<NonNull<Element<'a>>>,
    song: Song<'a>,
    next: Option<NonNull<Element<'a>>>,
}

impl<'s> Element<'s> {
    pub fn new(song: Song<'s>) -> Element<'s> {
        Element { prev: None, song: song.clone(), next: None }
    }
}
