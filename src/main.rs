// TODO remove once significantly implemented
#[allow(dead_code)]
#[allow(unused_parens)]

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

use std::{time::Duration};

mod settings;
use settings::Settings;

mod player;
use player::Player;



fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}



#[derive(Debug, Clone)]
struct Song<'a> {
    title: &'a str,
    album: &'a str,
    artist: &'a str,
    link: Link<'a>,
}

#[derive(Debug, Clone)]
enum Link<'a> {
    Local(&'a str),
    Plex(&'a str),
    Spotify(&'a str),
    Youtube(&'a str),
}



struct App<'a> {
    player: Player<'a>,
    settings: Settings,

    exit: bool,
}
impl<'a> App<'a> {
    // setup
    fn default() -> App<'a> {
        let mut settings = Settings::new();
        settings.read_config(None, false);
        let player = Player::new(settings.history_len(), 0.1, 9);
        App { settings, player, exit: false }
    }

    // main interactivity loop
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if self.player.check_completed() {
            self.player.advance_song();
        }
        // make sure there is an event, so read doesn't just block
        if let Ok(true) = event::poll(Duration::from_millis(20)) {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => { self.exit = true },
            // movement keys
            KeyCode::Char('h') => {},
            KeyCode::Char('j') => {},
            KeyCode::Char('k') => {},
            KeyCode::Char('l') => {},
            // player controls
            KeyCode::Char('a') => { self.player.devance_song(); },
            KeyCode::Char('s') => { self.player.stop() },
            KeyCode::Char('d') => { self.player.toggle_play() },
            KeyCode::Char('f') => { self.player.advance_song(); },
            KeyCode::Char('A') => { self.player.devance_album(); },
            // TODO this increment should come from user settings
            KeyCode::Char('S') => { self.player.volume_down(0.01); },
            // TODO this increment should come from user settings
            KeyCode::Char('D') => { self.player.volume_up(0.01); },
            KeyCode::Char('F') => { self.player.advance_album(); },

            // TODO REMOVE
            KeyCode::Char(' ') => {
                let test_songs = vec![
                    Song { title: "Why You'd Want to Live Here", album: "The Photo Album", artist: "Death Cab for Cutie", link: Link::Local("./test-track-one.flac") },
                    Song { title: "Everytime You Touch Me", album: "Disk", artist: "Moby", link: Link::Local("./test-track-two.wav") },
                    Song { title: "Invisible Face", album: "Nonagon Infinity", artist: "King Gizzard & The Lizard Wizard", link: Link::Local("./test-track-four.flac") },
                    Song { title: "Wah Wah", album: "Nonagon Infinity", artist: "King Gizzard & The Lizard Wizard", link: Link::Local("./test-track-five.flac") },
                    Song { title: "The Lost Art of Keeping A Secret", album: "Rated R", artist: "Queens of the Stone Age", link: Link::Local("./test-track-three.mp3") },
                ];

                self.player.queue_after_current(test_songs);
            },
            _ => {}
        }
    }
}

impl Widget for &App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Longhorn Music Player ".bold());
        let play_status = {
            if self.player.check_playing() { "" }
            else { "(PAUSED) " }
        };
        let instructions = {
            if let Some(song) = &self.player.current_song() {
                Line::from(vec![
                    " ".into(),
                    song.artist.into(),
                    "/".into(),
                    song.album.into(),
                    "/".into(),
                    song.title.into(),
                    " ".into(),
                    play_status.into()
                ])
            }
            else {
                Line::from(" QUEUE EMPTY ")
            }
        };
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions)
            .border_set(border::THICK)
            .render(area, buf);
    }
}
