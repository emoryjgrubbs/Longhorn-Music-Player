// TODO remove once significantly implemented
#[allow(dead_code)]

#[derive(Debug, Clone)]
struct Song<'a> {
    title: &'a str,
    album: &'a str,
    artist: &'a str,
    link: Link<'a>,
}

#[derive(Debug, Clone)]
enum Link<'a> {
    Local(usize),
    Plex(&'a str),
    Spotify(&'a str),
    Youtube(&'a str),
}



mod player {
    use std::ptr::NonNull;
    use std::boxed::Box;
    use Song;

    #[derive(Debug, Clone)]
    pub struct Queue<'a> {
        len_history: i32,
        len_queue: i32,
        start: Option<NonNull<Element<'a>>>,
        current: Option<NonNull<Element<'a>>>,
        end: Option<NonNull<Element<'a>>>,
    }

    impl<'q> Queue<'q> {
        pub fn new() -> Queue<'q> {
            Queue { len_history: 0, len_queue: 0, start: None, current: None, end: None }
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
                self.add_element_block_after_album(new_element_ptr);
            }
        }
        unsafe fn add_element_block_after_album(&mut self, new_element: NonNull<Element<'q>>) {
            unsafe {
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

        // TODO full implement
        pub fn advance_song(&mut self) {
            if let Some(current) = self.current {
                let self_next;
                unsafe { self_next = (*current.as_ptr()).next; }
                if let Some(_) = self_next {
                    self.current = self_next;
                    self.len_history += 1;
                }
            }
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
}



fn main() {
    let song_one = Song { title: "title 1", album: "Album 1", artist: "Artist 1", link: Link::Spotify("Spotify Link 1") };
    let song_two = Song { title: "title 2", album: "Album 2", artist: "Artist 2", link: Link::Spotify("Spotify Link 2") };
    let song_three = Song { title: "title 3", album: "Album 3", artist: "Artist 3", link: Link::Spotify("Spotify Link 3") };
    //println!("Song: {:?}", song_one);
    /*
    let mut queue = player::Queue::new();
    println!("Queue: {:#?}", queue);
    queue.add_after_queue(&song_one);
    queue.add_after_queue(&song_two);
    queue.add_after_queue(&song_three);
    println!("Queue: {:#?}", queue);
    */
    /*
    let mut prev = player::Element::new(&song_one);
    let mut current = player::Element::new(&song_two);
    let mut next = player::Element::new(&song_three);
    prev.mod_next(&current);
    current.mod_prev(&prev);
    current.mod_next(&next);
    next.mod_prev(&current);
    let next_of_prev = prev.get_next();
    println!("{:?}", prev.get_next());
    println!("{:?}", prev.get_next());
    prev.mod_next(&next);
    let next_again = prev.get_next();
    println!("{:?}", prev.get_next());
    println!("{:?}", prev.get_next());
    prev.mod_next(&current);
    prev.mod_next(&next);
    prev.mod_next(&current);
    println!("{:?}", prev.get_next());
    println!("{:?}", prev.get_next());
    */
    /*
    let mut queue = player::Queue::new();
    queue.add_after_queue(song_one);
    println!("{:?}", queue.get_current_song());
    println!("{:?}", queue.get_current_song());
    println!("{:?}", queue.get_current_song());
    println!("{:?}", queue);
    //let current_song = queue.get_current_song();
    //println!("{:?}", queue.get_current_song());
    */
/*
    let mut queue = player::Queue::new();
    queue.add_song_after_current(song_one);
    queue.add_song_after_current(song_two);
    queue.add_song_after_current(song_three);
    println!("{:?}", queue.current_song());
    queue.advance_song();
    println!();
    println!("{:?}", queue.current_song());
    println!("song_one, {:?}", queue.relative_song(-1));
    println!("song_three, {:?}", queue.relative_song(0));
    println!("song_two, {:?}", queue.relative_song(1));
    queue.advance_song();
    println!();
    println!("{:?}", queue.current_song());
    println!("song_one, {:?}", queue.relative_song(-2));
    println!("song_three, {:?}", queue.relative_song(-1));
    println!("song_two, {:?}", queue.relative_song(0));
    println!();
    queue.deadvance_song();
    queue.deadvance_song();
    println!("{:?}", queue.current_song());
    println!("song_one, {:?}", queue.relative_song(0));
    println!("song_three, {:?}", queue.relative_song(1));
    println!("song_two, {:?}", queue.relative_song(2));
*/
    let vec = vec![song_one, song_two, song_three];
    let mut queue = player::Queue::new();
    queue.add_song_block_after_current(vec);

    println!("song_one, {:?}", queue.current_song());
    println!("song_two, {:?}", queue.relative_song(1));
    println!("song_three, {:?}", queue.end_song());
}
