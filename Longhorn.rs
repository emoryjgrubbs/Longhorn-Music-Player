// TODO remove once significantly implemented
#[allow(dead_code)]

#[derive(Debug)]
struct Song<'a> {
    title: &'a str,
    album: &'a str,
    artist: &'a str,
    link: Link<'a>,
}

#[derive(Debug)]
enum Link<'a> {
    Local(usize),
    Plex(&'a str),
    Spotify(&'a str),
    Youtube(&'a str),
}



mod player {
}



fn main() {
    let song = Song { title: "title 1", album: "Album 1", artist: "Artist 1", link: Link::Spotify("Spotify Link 1") };
    println!("Song : {:?}", song);
}
