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

mod settings;

mod player;

fn main() {
    let mut user_settings = settings::Reader::new();

    println!("{:?}", user_settings.history_len());
    println!("{:?}", user_settings.library_paths());

    user_settings.read();
    println!();
    println!("{:?}", user_settings.history_len());

    let mut queue = player::Queue::new(10);

    let song_one = Song { title: "track 1", album: "Album 1", artist: "Artist 1", link: Link::Spotify("Spotify Link 1") };
    let song_two = Song { title: "track 2", album: "Album 1", artist: "Artist 1", link: Link::Spotify("Spotify Link 2") };
    let song_three = Song { title: "track 3", album: "Album 1", artist: "Artist 1", link: Link::Spotify("Spotify Link 3") };
    let song_four = Song { title: "track 4", album: "Album 2", artist: "Artist 2", link: Link::Spotify("Spotify Link 4") };
    let song_five = Song { title: "track 5", album: "Album 2", artist: "Artist 2", link: Link::Spotify("Spotify Link 5") };
}
