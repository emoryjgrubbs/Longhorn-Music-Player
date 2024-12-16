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
    use Song;

    #[derive(Debug, Clone)]
    pub struct Queue<'a> {
        len: u32,
        start: Option<NonNull<Element<'a>>>,
        current: Option<NonNull<Element<'a>>>,
        end: Option<NonNull<Element<'a>>>,
    }

    impl<'q> Queue<'q> {
        pub fn new() -> Queue<'q> {
            Queue { len: 0, start: None, current: None, end: None }
        }

        pub fn clear(&mut self) {
            self.len = 0;
            self.start = None;
            self.current = None;
            self.end = None;
        }

        /*
        *pub fn add_after_song() {}
        *
        *pub fn add_after_album() {}
        */

        pub fn add_after_queue(&mut self, new_song: &Song<'q>) {
            match self.end {
                None => {
                    self.len = 1;
                    let new_element = Element::new(&new_song);
                    self.start = Some((&new_element).into());
                    self.current = Some((&new_element).into());
                    self.end = Some((&new_element).into());
                }
                Some(end) => {
                    self.len += 1;
                    let new_element = Element::new(&new_song);
                    unsafe { end.as_ref().clone().mod_next(&new_element); }
                    self.end = Some((&new_element).into());
                }
            }
        }
    }



    use std::ptr::NonNull;
    #[derive(Debug, Clone)]
    struct Element<'a> {
        prev: Option<NonNull<Element<'a>>>,
        song: Song<'a>,
        next: Option<NonNull<Element<'a>>>,
    }

    impl<'s> Element<'s> {
        fn new(song: &Song<'s>) -> Element<'s> {
            Element { prev: None, song: song.clone(), next: None }
        }

        fn get_prev(&self) -> Option<Element> {
            match self.prev {
                None => { None },
                Some(prev) => unsafe { Some(prev.as_ref().clone()) },
            }
        }
        fn get_next(&self) -> Option<Element> {
            match self.next {
                None => { None },
                Some(next) => unsafe { Some(next.as_ref().clone()) },
            }
        }

        fn mod_prev(&mut self, new_prev: &Element<'s>) {
            self.prev = Some(new_prev.into());
        }
        fn mod_next(&mut self, new_next: &Element<'s>) {
            self.next = Some(new_next.into());
        }
    }
}



fn main() {
    let song_one = Song { title: "title 1", album: "Album 1", artist: "Artist 1", link: Link::Spotify("Spotify Link 1") };
    let song_two = Song { title: "title 2", album: "Album 2", artist: "Artist 2", link: Link::Spotify("Spotify Link 2") };
    let song_three = Song { title: "title 3", album: "Album 3", artist: "Artist 3", link: Link::Spotify("Spotify Link 3") };
    //println!("Song: {:?}", song_one);
    let mut queue = player::Queue::new();
    println!("Queue: {:#?}", queue);
    queue.add_after_queue(&song_one);
    queue.add_after_queue(&song_two);
    queue.add_after_queue(&song_three);
    println!("Queue: {:#?}", queue);
    /*
    let mut current = player::Element::new(&song_two);
    let prev = player::Element::new(&song_one);
    let next = player::Element::new(&song_three);
    println!("{:#?}", current.get_prev());
    current.mod_prev(&prev);
    current.mod_next(&next);
    println!("{:#?}", current.get_prev());
    */
}
