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

    #[derive(Debug)]
    pub struct Queue<'a> {
        start: u32,
        current: u32,
        end: u32,
        list: Vec<Song<'a>>,
    }

    impl<'q> Queue<'q> {
        pub fn new() -> Queue<'q> {
            Queue { start: 0, current: 0, end: 0, list: Vec::new() }
        }

        // pub fn clear() {}

        /*
        *pub fn add_after_song() {}
        *
        *pub fn add_after_album() {}
        */

        pub fn add_after_queue(&mut self, new_song: &Song<'q>) {
            self.end += 1;
            self.list.push(new_song.clone());
        }
    }
}



fn main() {
    let song_one = Song { title: "title 1", album: "Album 1", artist: "Artist 1", link: Link::Spotify("Spotify Link 1") };
    let song_two = Song { title: "title 2", album: "Album 2", artist: "Artist 2", link: Link::Spotify("Spotify Link 2") };
    let song_three = Song { title: "title 3", album: "Album 3", artist: "Artist 3", link: Link::Spotify("Spotify Link 3") };
    println!("Song: {:?}", song_one);
    let mut queue = player::Queue::new();
    println!("Queue: {:#?}", queue);
    queue.add_after_queue(&song_one);
    queue.add_after_queue(&song_two);
    queue.add_after_queue(&song_three);
    println!("Queue: {:#?}", queue);
}
