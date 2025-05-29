// TODO remove once significantly implemented
#[allow(dead_code)]
#[allow(unused_parens)]

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
use settings::Settings;

mod player;
use player::Player;


fn main() {
    let mut user_settings = Settings::new();
    user_settings.read_config(None, true);

    let mut player = Player::new(user_settings.history_len());

    player.test_play();
    /*
    let song_one = Song { title: "track 1", album: "Album 1", artist: "Artist 1", link: Link::Spotify("Spotify Link 1") };
    let song_two = Song { title: "track 2", album: "Album 1", artist: "Artist 1", link: Link::Spotify("Spotify Link 2") };
    let song_three = Song { title: "track 3", album: "Album 2", artist: "Artist 2", link: Link::Spotify("Spotify Link 3") };
    let song_four = Song { title: "track 4", album: "Album 3", artist: "Artist 3", link: Link::Spotify("Spotify Link 4") };
    let song_five = Song { title: "track 5", album: "Album 3", artist: "Artist 3", link: Link::Spotify("Spotify Link 5") };

    let new_songs_one = vec![ song_one, song_two, song_three ];
    let new_songs_two = vec![ song_four, song_five ];

    player.queue_after_album(new_songs_one);
    player.queue_after_album(new_songs_two);
    println!("");
    player.test_print();

    println!("");
    player.jump_to_song(2);
    player.test_print();

    println!("");
    player.jump_to_song(-1);
    player.test_print();
    */
}
