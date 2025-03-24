use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::VecDeque;

pub struct Lexer {
    machine: Machine,
    tokens: VecDeque<LexicalUnit>
}
impl Lexer {
    pub fn process_file(config_path: &str) -> std::io::Result<VecDeque<LexicalUnit>> {
        // create lexer struct to contain tokens & leximes and the dfa
        let mut lexer = Lexer { machine: Machine::new(), tokens: VecDeque::new() };
        // set up file
        let file = File::open(config_path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();

        // 
        reader.read_to_string(&mut contents)?;
        let mut lexime = String::new();
        for symbol in contents.chars() {
            let token = lexer.machine.transition(symbol);
            if let Some(token) = token {
                if [Token::Slash,
                    Token::Star,
                    Token::Num,
                    Token::String,
                    Token::WhiteSpace].contains(&token) {
                    lexer.tokens.push_back(LexicalUnit {token, lexime: lexime.clone()});
                    lexime.clear();
                    {
                        let token = lexer.machine.transition(symbol);
                        if let Some(token) = token {
                            lexime.push(symbol);
                            lexer.tokens.push_back(LexicalUnit {token, lexime: lexime.clone()});
                            lexime.clear();
                        }
                        else {
                            lexime.push(symbol);
                        }
                    }
                }
                else {
                    lexime.push(symbol);
                    lexer.tokens.push_back(LexicalUnit {token, lexime: lexime.clone()});
                    lexime.clear();
                }
            }
            else { lexime.push(symbol); }
        }

        // return tokens & leximes
        Ok(lexer.tokens)
    }
}

#[derive(Debug)]
pub struct LexicalUnit {
    token: Token,
    lexime: String,
}
impl LexicalUnit {
    pub fn get_token(&self) -> Token { self.token.clone() }
    pub fn get_lexime(&self) -> String { self.lexime.clone() }
}

struct Machine {
    state: SubMachine,
}
impl Machine{
    fn new() -> Machine {
        Machine { state: SubMachine::StartMachine }
    }

    // exterior function, calls submachine transition functions
    fn transition(&mut self, symbol: char) -> Option<Token> {
        match self.state.clone() {
            SubMachine::StartMachine => { self.start_machine(symbol) },
            SubMachine::CommStartMachine => { self.comm_start_machine(symbol) },
            SubMachine::CommEndMachine => { self.comm_end_machine(symbol) },
            SubMachine::WSMachine => { self.white_space_machine(symbol) },
            SubMachine::NumMachine => { self.num_machine(symbol) },
            SubMachine::FileMachine(state) => { self.key_file_machine(state, symbol) },
            SubMachine::LibPathMachine(state) => { self.key_lib_path_machine(state, symbol) },
            SubMachine::MaxHistMachine(state) => { self.key_max_hist_machine(state, symbol) },
            SubMachine::TopMachine(state) => { self.key_top_machine(state, symbol) },
            SubMachine::TopSngLenMachine(state) => { self.key_sng_len_machine(state, symbol) },
            SubMachine::TopAlbLenMachine(state) => { self.key_alb_len_machine(state, symbol) },
            SubMachine::TopArtLenMachine(state) => { self.key_art_len_machine(state, symbol) },
            SubMachine::TopDecayMachine(state) => { self.key_decay_machine(state, symbol) },
            SubMachine::StringMachine => { self.string_machine(symbol) }
        }
    }

    // all transitions from start state
    fn start_machine(&mut self, symbol: char) -> Option<Token> {
        match symbol {
            // single character tokens
            '[' => { Some(Token::OpenSquare) },
            ']' => { Some(Token::CloseSquare) },
            '{' => { Some(Token::OpenCurl) },
            '}' => { Some(Token::CloseCurl) },
            '(' => { Some(Token::OpenParen) },
            ')' => { Some(Token::CloseParen) },
            '=' => { Some(Token::Asn) },
            '+' => { Some(Token::Plus) },
            '-' => { Some(Token::Dash) },
            '^' => { Some(Token::Carrot) },
            '%' => { Some(Token::Percent) },
            '\\' => { Some(Token::Esc) },
            '.' => { Some(Token::Dot) },
            // possible comment
            '*' => {
                self.state = SubMachine::CommEndMachine;
                None
            },
            '/' => {
                self.state = SubMachine::CommStartMachine;
                None
            },
            // multiple symbols satisfy
            _ => {
                if ((symbol as i32) >= ('\t' as i32) 
                    && (symbol as i32) <= ('\r' as i32)) 
                    || (symbol == ' ') {
                        // end of line
                        if symbol == '\n' || symbol == '\r' { Some(Token::EndL) }
                        // whitespace
                        else {
                            self.state = SubMachine::WSMachine;
                            None
                        }
                }
                // num
                else if (symbol as i32) >= ('0' as i32) 
                    && (symbol as i32) <= ('9' as i32) {
                        self.state = SubMachine::NumMachine;
                        None
                }
                // file
                else if symbol == 'F' || symbol == 'f' {
                    self.state = SubMachine::FileMachine(FileState::F);
                    None
                }
                // libpath
                else if symbol == 'L' || symbol == 'l' {
                    self.state = SubMachine::LibPathMachine(LibPathState::L);
                    None
                }
                // maxhist
                else if symbol == 'M' || symbol == 'm' {
                    self.state = SubMachine::MaxHistMachine(MaxHistState::M);
                    None
                }
                // top...
                else if symbol == 'T' || symbol == 't' {
                    self.state = SubMachine::TopMachine(TopState::T);
                    None
                }
                // garbage, anything not matching another symbol is a string
                else {
                    self.state = SubMachine::StringMachine;
                    None
                }
            }
        }
    }

    // checks to see if the symbols form a comment or just a slash ('/') or star ('*')
    fn comm_start_machine(&mut self, symbol: char) -> Option<Token> {
        if symbol == '*' {
            self.state = SubMachine::StartMachine;
            Some(Token::OpenComm)
        }
        else {
            self.state = SubMachine::StartMachine;
            Some(Token::Slash)
        }
    }
    fn comm_end_machine(&mut self, symbol: char) -> Option<Token> {
        if symbol == '/' {
            self.state = SubMachine::StartMachine;
            Some(Token::CloseComm)
        }
        else {
            self.state = SubMachine::StartMachine;
            Some(Token::Star)
        }
    }

    // continues whitespace token until end of whitespace sequence
    fn white_space_machine(&mut self, symbol: char) -> Option<Token> {
        if ((symbol as i32) >= ('\t' as i32) 
            && (symbol as i32) <= ('\r' as i32))
            || (symbol == ' ') {
                // end of line
                if symbol == '\n' || symbol == '\r' { 
                    self.state = SubMachine::StartMachine;
                    Some(Token::WhiteSpace) 
                }
                // whitespace
                else { None }
        }
        else { 
            self.state = SubMachine::StartMachine;
            Some(Token::WhiteSpace)
        }
    }

    // continues number token until end of number sequence
    fn num_machine(&mut self, symbol: char) -> Option<Token> {
        if (symbol as i32) >= ('0' as i32) && (symbol as i32) <= ('9' as i32) {
            None
        }
        else{
            self.state = SubMachine::StartMachine;
            Some(Token::Num)
        }
    }
    
    // continues string token until end of string sequence
    fn string_machine(&mut self, symbol: char) -> Option<Token> {
        // check if a symbol generates a single-character token
        if let Some(_) = self.start_machine(symbol) {
            self.state = SubMachine::StartMachine;
            Some(Token::String)
        }
        // check if the symbol is recognized by a multi-character
        //  machine, which excludes it from being a string
        else if [SubMachine::CommStartMachine,
                SubMachine::CommEndMachine,
                SubMachine::WSMachine,
                SubMachine::NumMachine].contains(&self.state) {
            self.state = SubMachine::StartMachine;
            Some(Token::String)
        }
        // otherwise stay in string machine
        //  keyword contain characters from strings, so reset state to string
        else {
            self.state = SubMachine::StringMachine;
            None
        }
    }

    // keyword machines
    //  if a symbol matches the transition path, continue keyword, otherwise string
    //  lines: 241 - 685 |NOTE| keep up to date
    fn key_file_machine(&mut self, state: FileState, symbol: char) -> Option<Token> {
        match state {
            FileState::F => {
                if symbol == 'I' || symbol == 'i' {
                    self.state = SubMachine::FileMachine(FileState::Fi);
                    None
                }
                else { self.string_machine(symbol) }
            },
            FileState::Fi => {
                if symbol == 'L' || symbol == 'l' {
                    self.state = SubMachine::FileMachine(FileState::Fil);
                    None
                }
                else { self.string_machine(symbol) }
            },
            FileState::Fil => {
                if symbol == 'e' || symbol == 'e' {
                    self.state = SubMachine::StartMachine;
                    Some(Token::File)
                }
                else { self.string_machine(symbol) }
            },
        }
    }

    fn key_lib_path_machine(&mut self, state: LibPathState, symbol: char) -> Option<Token> {
        match state {
            LibPathState::L => {
                if symbol == 'I' || symbol == 'i' {
                    self.state = SubMachine::LibPathMachine(LibPathState::Li);
                    None
                }
                else { self.string_machine(symbol) }
            },
            LibPathState::Li => {
                if symbol == 'B' || symbol == 'b' {
                    self.state = SubMachine::LibPathMachine(LibPathState::Lib);
                    None
                }
                else { self.string_machine(symbol) }
            },
            LibPathState::Lib => {
                if symbol == 'R' || symbol == 'r' {
                    self.state = SubMachine::LibPathMachine(LibPathState::Libr);
                    None
                }
                else { self.string_machine(symbol) }
            },
            LibPathState::Libr => {
                if symbol == 'A' || symbol == 'a' {
                    self.state = SubMachine::LibPathMachine(LibPathState::Libra);
                    None
                }
                else { self.string_machine(symbol) }
            },
            LibPathState::Libra => {
                if symbol == 'R' || symbol == 'r' {
                    self.state = SubMachine::LibPathMachine(LibPathState::Librar);
                    None
                }
                else { self.string_machine(symbol) }
            },
            LibPathState::Librar => {
                if symbol == 'Y' || symbol == 'y' {
                    self.state = SubMachine::LibPathMachine(LibPathState::Library);
                    None
                }
                else { self.string_machine(symbol) }
            },
            LibPathState::Library => {
                if symbol == 'P' || symbol == 'p' {
                    self.state = SubMachine::LibPathMachine(LibPathState::LibraryP);
                    None
                }
                else { self.string_machine(symbol) }
            },
            LibPathState::LibraryP => {
                if symbol == 'A' || symbol == 'a' {
                    self.state = SubMachine::LibPathMachine(LibPathState::LibraryPa);
                    None
                }
                else { self.string_machine(symbol) }
            },
            LibPathState::LibraryPa => {
                if symbol == 'T' || symbol == 't' {
                    self.state = SubMachine::LibPathMachine(LibPathState::LibraryPat);
                    None
                }
                else { self.string_machine(symbol) }
            },
            LibPathState::LibraryPat => {
                if symbol == 'H' || symbol == 'h' {
                    self.state = SubMachine::StartMachine;
                    Some(Token::LibPath)
                }
                else { self.string_machine(symbol) }
            },
        }
    }

    fn key_max_hist_machine(&mut self, state: MaxHistState, symbol: char) -> Option<Token> {
        match state {
            MaxHistState::M => {
                if symbol == 'A' || symbol == 'a' {
                    self.state = SubMachine::MaxHistMachine(MaxHistState::Ma);
                    None
                }
                else { self.string_machine(symbol) }
            },
            MaxHistState::Ma => {
                if symbol == 'X' || symbol == 'x' {
                    self.state = SubMachine::MaxHistMachine(MaxHistState::Max);
                    None
                }
                else { self.string_machine(symbol) }
            },
            MaxHistState::Max => {
                if symbol == 'H' || symbol == 'h' {
                    self.state = SubMachine::MaxHistMachine(MaxHistState::MaxH);
                    None
                }
                else { self.string_machine(symbol) }
            },
            MaxHistState::MaxH => {
                if symbol == 'I' || symbol == 'i' {
                    self.state = SubMachine::MaxHistMachine(MaxHistState::MaxHi);
                    None
                }
                else { self.string_machine(symbol) }
            },
            MaxHistState::MaxHi => {
                if symbol == 'S' || symbol == 's' {
                    self.state = SubMachine::MaxHistMachine(MaxHistState::MaxHis);
                    None
                }
                else { self.string_machine(symbol) }
            },
            MaxHistState::MaxHis => {
                if symbol == 'T' || symbol == 't' {
                    self.state = SubMachine::MaxHistMachine(MaxHistState::MaxHist);
                    None
                }
                else { self.string_machine(symbol) }
            },
            MaxHistState::MaxHist => {
                if symbol == 'O' || symbol == 'o' {
                    self.state = SubMachine::MaxHistMachine(MaxHistState::MaxHisto);
                    None
                }
                else { self.string_machine(symbol) }
            },
            MaxHistState::MaxHisto => {
                if symbol == 'R' || symbol == 'r' {
                    self.state = SubMachine::MaxHistMachine(MaxHistState::MaxHistor);
                    None
                }
                else { self.string_machine(symbol) }
            },
            MaxHistState::MaxHistor => {
                if symbol == 'Y' || symbol == 'y' {
                    self.state = SubMachine::StartMachine;
                    Some(Token::MaxHist)
                }
                else { self.string_machine(symbol) }
            },
        }
    }

    // recognize the start of 'Top...' keywords
    fn key_top_machine(&mut self, state: TopState, symbol: char) -> Option<Token> {
        match state {
            TopState::T => {
                if symbol == 'O' || symbol == 'o' {
                    self.state = SubMachine::TopMachine(TopState::To);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopState::To => {
                if symbol == 'P' || symbol == 'p' {
                    self.state = SubMachine::TopMachine(TopState::Top);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopState::Top => {
                if symbol == 'A' || symbol == 'a' {
                    self.state = SubMachine::TopMachine(TopState::TopA);
                    None
                }
                else if symbol == 'D' || symbol == 'd' {
                    self.state = SubMachine::TopDecayMachine(TopDecayState::D);
                    None
                }
                else if symbol == 'S' || symbol == 's' {
                    self.state = SubMachine::TopSngLenMachine(TopSngLenState::S);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopState::TopA => {
                if symbol == 'L' || symbol == 'l' {
                    self.state = SubMachine::TopAlbLenMachine(TopAlbLenState::Al);
                    None
                }
                else if symbol == 'R' || symbol == 'r' {
                    self.state = SubMachine::TopArtLenMachine(TopArtLenState::Ar);
                    None
                }
                else { self.string_machine(symbol) }
            },
        }
    }

    fn key_sng_len_machine(&mut self, state: TopSngLenState, symbol: char) -> Option<Token> {
        match state {
            TopSngLenState::S => {
                if symbol == 'O' || symbol == 'o' {
                    self.state = SubMachine::TopSngLenMachine(TopSngLenState::So);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopSngLenState::So => {
                if symbol == 'N' || symbol == 'n' {
                    self.state = SubMachine::TopSngLenMachine(TopSngLenState::Son);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopSngLenState::Son => {
                if symbol == 'G' || symbol == 'g' {
                    self.state = SubMachine::TopSngLenMachine(TopSngLenState::Song);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopSngLenState::Song => {
                if symbol == 'S' || symbol == 's' {
                    self.state = SubMachine::TopSngLenMachine(TopSngLenState::Songs);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopSngLenState::Songs => {
                if symbol == 'L' || symbol == 'l' {
                    self.state = SubMachine::TopSngLenMachine(TopSngLenState::SongsL);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopSngLenState::SongsL => {
                if symbol == 'E' || symbol == 'e' {
                    self.state = SubMachine::TopSngLenMachine(TopSngLenState::SongsLe);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopSngLenState::SongsLe => {
                if symbol == 'N' || symbol == 'n' {
                    self.state = SubMachine::StartMachine;
                    Some(Token::TopSngLen)
                }
                else { self.string_machine(symbol) }
            },
        }
    }

    fn key_alb_len_machine(&mut self, state: TopAlbLenState, symbol: char) -> Option<Token> {
        match state {
            TopAlbLenState::Al => {
                if symbol == 'B' || symbol == 'b' {
                    self.state = SubMachine::TopAlbLenMachine(TopAlbLenState::Alb);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopAlbLenState::Alb => {
                if symbol == 'U' || symbol == 'u' {
                    self.state = SubMachine::TopAlbLenMachine(TopAlbLenState::Albu);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopAlbLenState::Albu => {
                if symbol == 'M' || symbol == 'm' {
                    self.state = SubMachine::TopAlbLenMachine(TopAlbLenState::Album);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopAlbLenState::Album => {
                if symbol == 'S' || symbol == 's' {
                    self.state = SubMachine::TopAlbLenMachine(TopAlbLenState::Albums);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopAlbLenState::Albums => {
                if symbol == 'L' || symbol == 'l' {
                    self.state = SubMachine::TopAlbLenMachine(TopAlbLenState::AlbumsL);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopAlbLenState::AlbumsL => {
                if symbol == 'E' || symbol == 'e' {
                    self.state = SubMachine::TopAlbLenMachine(TopAlbLenState::AlbumsLe);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopAlbLenState::AlbumsLe => {
                if symbol == 'N' || symbol == 'n' {
                    self.state = SubMachine::StartMachine;
                    Some(Token::TopAlbLen)
                }
                else { self.string_machine(symbol) }
            },
        }
    }

    fn key_art_len_machine(&mut self, state: TopArtLenState, symbol: char) -> Option<Token> {
        match state {
            TopArtLenState::Ar => {
                if symbol == 'T' || symbol == 't' {
                    self.state = SubMachine::TopArtLenMachine(TopArtLenState::Art);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopArtLenState::Art => {
                if symbol == 'I' || symbol == 'i' {
                    self.state = SubMachine::TopArtLenMachine(TopArtLenState::Arti);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopArtLenState::Arti => {
                if symbol == 'S' || symbol == 's' {
                    self.state = SubMachine::TopArtLenMachine(TopArtLenState::Artis);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopArtLenState::Artis => {
                if symbol == 'T' || symbol == 't' {
                    self.state = SubMachine::TopArtLenMachine(TopArtLenState::Artist);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopArtLenState::Artist => {
                if symbol == 'S' || symbol == 's' {
                    self.state = SubMachine::TopArtLenMachine(TopArtLenState::Artists);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopArtLenState::Artists => {
                if symbol == 'L' || symbol == 'l' {
                    self.state = SubMachine::TopArtLenMachine(TopArtLenState::ArtistsL);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopArtLenState::ArtistsL => {
                if symbol == 'E' || symbol == 'e' {
                    self.state = SubMachine::TopArtLenMachine(TopArtLenState::ArtistsLe);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopArtLenState::ArtistsLe => {
                if symbol == 'N' || symbol == 'n' {
                    self.state = SubMachine::StartMachine;
                    Some(Token::TopArtLen)
                }
                else { self.string_machine(symbol) }
            },
        }
    }

    fn key_decay_machine(&mut self, state: TopDecayState, symbol: char) -> Option<Token> {
        match state {
            TopDecayState::D => {
                if symbol == 'E' || symbol == 'e' {
                    self.state = SubMachine::TopDecayMachine(TopDecayState::De);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopDecayState::De => {
                if symbol == 'C' || symbol == 'c' {
                    self.state = SubMachine::TopDecayMachine(TopDecayState::Dec);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopDecayState::Dec => {
                if symbol == 'A' || symbol == 'a' {
                    self.state = SubMachine::TopDecayMachine(TopDecayState::Deca);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopDecayState::Deca => {
                if symbol == 'Y' || symbol == 'y' {
                    self.state = SubMachine::TopDecayMachine(TopDecayState::Decay);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopDecayState::Decay => {
                if symbol == 'R' || symbol == 'r' {
                    self.state = SubMachine::TopDecayMachine(TopDecayState::DecayR);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopDecayState::DecayR => {
                if symbol == 'A' || symbol == 'a' {
                    self.state = SubMachine::TopDecayMachine(TopDecayState::DecayRa);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopDecayState::DecayRa => {
                if symbol == 'T' || symbol == 't' {
                    self.state = SubMachine::TopDecayMachine(TopDecayState::DecayRat);
                    None
                }
                else { self.string_machine(symbol) }
            },
            TopDecayState::DecayRat => {
                if symbol == 'E' || symbol == 'e' {
                    self.state = SubMachine::StartMachine;
                    Some(Token::TopDecay)
                }
                else { self.string_machine(symbol) }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum SubMachine {
    StartMachine,
    CommStartMachine,
    CommEndMachine,
    WSMachine,
    NumMachine,
    FileMachine(FileState),
    LibPathMachine(LibPathState),
    MaxHistMachine(MaxHistState),
    TopMachine(TopState),
    TopSngLenMachine(TopSngLenState),
    TopAlbLenMachine(TopAlbLenState),
    TopArtLenMachine(TopArtLenState),
    TopDecayMachine(TopDecayState),
    StringMachine,
}

#[derive(Debug, Clone, PartialEq)]
enum FileState {
    F,
    Fi,
    Fil,
}

#[derive(Debug, Clone, PartialEq)]
enum LibPathState {
    L,
    Li,
    Lib,
    Libr,
    Libra,
    Librar,
    Library,
    LibraryP,
    LibraryPa,
    LibraryPat,
}

#[derive(Debug, Clone, PartialEq)]
enum MaxHistState {
    M,
    Ma,
    Max,
    MaxH,
    MaxHi,
    MaxHis,
    MaxHist,
    MaxHisto,
    MaxHistor,
}

#[derive(Debug, Clone, PartialEq)]
enum TopState {
    T,
    To,
    Top,
    TopA,
}

#[derive(Debug, Clone, PartialEq)]
enum TopSngLenState {
    S,
    So,
    Son,
    Song,
    Songs,
    SongsL,
    SongsLe,
}

#[derive(Debug, Clone, PartialEq)]
enum TopAlbLenState {
    Al,
    Alb,
    Albu,
    Album,
    Albums,
    AlbumsL,
    AlbumsLe,
}

#[derive(Debug, Clone, PartialEq)]
enum TopArtLenState {
    Ar,
    Art,
    Arti,
    Artis,
    Artist,
    Artists,
    ArtistsL,
    ArtistsLe,
}

#[derive(Debug, Clone, PartialEq)]
enum TopDecayState {
    D,
    De,
    Dec,
    Deca,
    Decay,
    DecayR,
    DecayRa,
    DecayRat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // keywords
    File,
    LibPath,
    MaxHist,
    TopSngLen,
    TopAlbLen,
    TopArtLen,
    TopDecay,
    // list
    OpenSquare,
    CloseSquare,
    // math
    OpenCurl,
    CloseCurl,
    // parenthesis
    OpenParen,
    CloseParen,
    // comments
    OpenComm,
    CloseComm,
    // ops
    Asn,
    Plus,
    Dash,
    Star,
    Slash,
    Carrot,
    Percent,
    Dot,
    Esc,
    EndL,
    // lexime needed
    Num,
    String,
    WhiteSpace,
    // special
    NegativeMult
}
