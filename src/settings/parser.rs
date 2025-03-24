// TODO remove debug println!s
use super::Settings;
use super::lexer::LexicalUnit;
use super::lexer::Token;
use std::collections::VecDeque;
use std::result;

pub struct Parser {
    settings: Settings,
    sub_files: Vec<String>,
    stack: Vec<Rule>,
    tokens: VecDeque<LexicalUnit>,
    line_number: i32,
    error_lines: Vec<i32>
}
impl Parser {
    pub fn process_tokens(tokens: VecDeque<LexicalUnit>) -> Result<Parser, ()> {
        /*
        for token in tokens {
            println!("{:?}", token);
        }
        Err(())
        */
        let mut parser = Parser { settings: Settings::new(), sub_files: vec![], stack: vec![Rule::Config], tokens , line_number: 0, error_lines: vec![] };
        // for error reporting purposes
        // loop over all elements in the tokens vec
        loop {
            let rule = parser.stack.pop();
            let result = parser.parse_line(rule);
            if result == Some(Status::Done) { break }
        }
        Ok(parser)
    }

    fn parse_line(&mut self, rule: Option<Rule>) -> Option<Status> {
        match rule {
            Some(Rule::Config) => {
                // try to parse another line
                if let Some(_) = self.tokens.front() {
                    self.stack.push(Rule::Config);
                    self.stack.push(Rule::Line);
                    None
                }
                // end parsing
                else {
                    Some(Status::Done)
                }
            },
            Some(Rule::Line) => {
                if let Some(lexical_unit) = self.tokens.pop_front() {
                    match lexical_unit.get_token() {
                        // ignore
                        Token::WhiteSpace => {
                            self.stack.push(Rule::Line);
                            None
                        },
                        Token::OpenComm => {
                            self.stack.push(Rule::Line);
                            self.stack.push(Rule::CloseComm);
                            self.parse_comment();
                            None
                        },
                        // valid line assignment
                        Token::File => {
                            self.stack.push(Rule::EndL);
                            self.stack.push(Rule::Path);
                            self.stack.push(Rule::Asn);

                            if let Some(result) = self.parse_path() {
                                println!("New File: {:?}", &result);
                                self.sub_files.push(result);
                                self.line_number += 1;
                                None
                            }
                            else { Some(Status::LineError) }
                        },
                        Token::LibPath => {
                            if let Some(result) = self.parse_path() {
                                println!("New Lib Path: {:?}", &result);
                                self.settings.library_paths.push(result);
                                self.line_number += 1;
                                None
                            }
                            else { Some(Status::LineError) }
                        },
                        Token::MaxHist => {
                            if let Some(result) = self.parse_int() {
                                println!("New Max Hist: {:?}", &result);
                                self.settings.max_history = result;
                                self.line_number += 1;
                                None
                            }
                            else { Some(Status::LineError) }
                        },
                        Token::TopSngLen => {
                            if let Some(result) = self.parse_int() {
                                println!("New Top Sng Len: {:?}", &result);
                                self.settings.top_songs_len = result;
                                self.line_number += 1;
                                None
                            }
                            else { Some(Status::LineError) }
                        },
                        Token::TopAlbLen => {
                            if let Some(result) = self.parse_int() {
                                println!("New Top Alb Len: {:?}", &result);
                                self.settings.top_albums_len = result;
                                self.line_number += 1;
                                None
                            }
                            else { Some(Status::LineError) }
                        },
                        Token::TopArtLen => {
                            if let Some(result) = self.parse_int() {
                                println!("New Top Art Len: {:?}", &result);
                                self.settings.top_artists_len = result;
                                self.line_number += 1;
                                None
                            }
                            else { Some(Status::LineError) }
                        },
                        Token::TopDecay => {
                            if let Some(result) = self.parse_float() {
                                println!("New Top Decay Rate: {:?}", &result);
                                self.settings.top_decay = result;
                                self.line_number += 1;
                                None
                            }
                            else { Some(Status::LineError) }
                        },
                        // empty line
                        Token::EndL => {
                            println!("Empty Line");
                            self.line_number += 1;
                            None
                        }
                        _ => {
                            Some(Status::LineError)
                        }
                    }
                }
                else { 
                    Some(Status::LineError)
                }
            },
            _ => {
                Some(Status::LineError)
            },
        }
    }

    fn parse_comment(&mut self) -> Status {
        let mut comment: String = "/*".to_string();
        loop {
            if let Some(lexical_unit) = self.tokens.pop_front() {
                comment.push_str(&lexical_unit.get_lexime());

                if lexical_unit.get_token() == Token::OpenComm { self.stack.push(Rule::CloseComm); }
                else if lexical_unit.get_token() == Token::CloseComm { 
                    if let Some(rule) = self.stack.pop() {
                        if rule != Rule::CloseComm { return Status::LineError }
                    }
                    else { return Status::LineError }

                    if let Some(rule) = self.stack.last() {
                        if rule != &Rule::CloseComm { println!("{}", comment); return Status::Done }
                    }
                    else { return Status::LineError }
                }

            }
            else { return Status::LineError }
        }
    }

    fn parse_path(&mut self) -> Option<String> {
        if let Err(()) = self.parse_asn() { return None }
        let path;
        // TODO implement
        path = "".to_string();
        
        if let Err(()) = self.parse_to_endl() { return None }
        Some(path)
    }
    fn parse_int(&mut self) -> Option<i32> {
        if let Err(()) = self.parse_asn() { return None }
        let int;
        if let Some(lexical_unit) = self.tokens.pop_front() {
            let mut number = lexical_unit.get_lexime();
            match lexical_unit.get_token() {
                Token::Num => {
                    if let Ok(result) = self.parse_num() {
                        number.push_str(&result);
                        if let Ok(parsed_result) = number.parse::<f64>() { int = parsed_result.round() as i32; }
                        else { return None }
                    }
                    else { return None }
                },
                Token::Dot => {
                    if let Ok(result) = self.parse_decimal() {
                        number.push_str(&result);
                        if let Ok(parsed_result) = number.parse::<f64>() { int = parsed_result.round() as i32; }
                        else { return None }
                    }
                    else { return None }
                },
                Token::Dash => { 
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        number.push_str(&lexical_unit.get_lexime());
                        match lexical_unit.get_token() {
                            Token::Num => {
                                if let Ok(result) = self.parse_num() {
                                    number.push_str(&result);
                                    if let Ok(parsed_number) = number.parse::<f64>() { int = parsed_number.round() as i32; }
                                    else { return None }
                                }
                                else { return None }
                            },
                            Token::Dot => {
                                if let Ok(result) = self.parse_decimal() {
                                    number.push_str(&result);
                                    if let Ok(parsed_number) = number.parse::<f64>() { int = parsed_number.round() as i32; }
                                    else { return None }
                                }
                                else { return None }
                            },
                            _ => { return None },
                        }
                    }
                    else { return None }
                },
                Token::OpenCurl => {
                    if let Ok(result) = self.parse_math() {
                        int = result as i32;
                    }
                    else { return None }
                },
                _ => { return None },
            }
        }
        else { return None }
        if let Err(()) = self.parse_to_endl() { return None }
        Some(int)
    }
    fn parse_float(&mut self) -> Option<f64> {
        if let Err(()) = self.parse_asn() { return None }
        let float;
        if let Some(lexical_unit) = self.tokens.pop_front() {
            let mut number = lexical_unit.get_lexime();
            match lexical_unit.get_token() {
                Token::Num => {
                    if let Ok(result) = self.parse_num() {
                        number.push_str(&result);
                        if let Ok(parsed_number) = number.parse::<f64>() { float = parsed_number; }
                        else { return None }
                    }
                    else { return None }
                },
                Token::Dot => {
                    if let Ok(result) = self.parse_decimal() {
                        number.push_str(&result);
                        if let Ok(parsed_number) = number.parse::<f64>() { float = parsed_number; }
                        else { return None }
                    }
                    else { return None }
                },
                Token::Dash => { 
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        number.push_str(&lexical_unit.get_lexime());
                        match lexical_unit.get_token() {
                            Token::Num => {
                                if let Ok(result) = self.parse_num() {
                                    number.push_str(&result);
                                    if let Ok(parsed_number) = number.parse::<f64>() { float = parsed_number; }
                                    else { return None }
                                }
                                else { return None }
                            },
                            Token::Dot => {
                                if let Ok(result) = self.parse_decimal() {
                                    number.push_str(&result);
                                    if let Ok(parsed_number) = number.parse::<f64>() { float = parsed_number; }
                                    else { return None }
                                }
                                else { return None }
                            },
                            _ => { return None },
                        }
                    }
                    else { return None }
                },
                Token::OpenCurl => {
                    if let Ok(result) = self.parse_math() {
                        float = result;
                    }
                    else { return None }
                },
                _ => { return None },
            }
        }
        else { return None }
        if let Err(()) = self.parse_to_endl() { return None }
        Some(float)
    }

    fn parse_asn(&mut self) -> Result<(), ()> {
        if let Err(()) = self.parse_clear_garbage() { return Err(()) }
        if let Some(lexical_unit) = self.tokens.pop_front() {
            if lexical_unit.get_token() != Token::Asn { return Err(()) }
        }
        else { return Err(()) }
        if let Err(()) = self.parse_clear_garbage() { return Err(()) }
        Ok(())
    }

    fn parse_to_endl(&mut self) -> Result<(), ()> {
        if let Err(()) = self.parse_clear_garbage() { return Err(()) }
        if let Some(lexical_unit) = self.tokens.pop_front() {
            if lexical_unit.get_token() != Token::EndL { return Err(()) }
        }
        Ok(())
    }

    fn parse_clear_garbage(&mut self) -> Result<(), ()> {
        loop {
            if let Some(lexical_unit) = self.tokens.front() {
                if lexical_unit.get_token() == Token::WhiteSpace { self.tokens.pop_front(); }
                else if lexical_unit.get_token() == Token::OpenComm {
                    self.tokens.pop_front();
                    self.stack.push(Rule::CloseComm);
                    self.parse_comment();
                }
                else { return Ok(()) }
            } 
            else { return Err(()) }
        }
    }

    fn parse_math(&mut self) -> Result<f64, ()> {
        self.stack.push(Rule::CloseCurl);
        self.stack.push(Rule::Term);

        // operators
        let mut op_stack = vec![];
        // terms
        let mut term_stack = vec![];
        // length of levels
        let mut len_stack = vec![];

        let mut level_len = 1;
        loop {
            if let Err(()) = self.parse_clear_garbage() { return Err(()) }
            match self.stack.pop() {
                Some(Rule::Term) => {
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        let mut number = lexical_unit.get_lexime();
                        match lexical_unit.get_token() {
                            Token::Num => {
                                level_len += 1;
                                if let Ok(result) = self.parse_num() {
                                    number.push_str(&result);
                                    if let Ok(parsed_number) = number.parse::<f64>() { 
                                        term_stack.push(parsed_number);
                                    }
                                    else { return Err(()) }
                                }
                                else { return Err(()) }
                            },
                            Token::Dot => {
                                level_len += 1;
                                if let Ok(result) = self.parse_decimal() {
                                    number.push_str(&result);
                                    if let Ok(parsed_number) = number.parse::<f64>() { 
                                        term_stack.push(parsed_number);
                                    }
                                    else { return Err(()) }
                                }
                                else { return Err(()) }
                            },
                            // negative number
                            Token::Dash => {
                                level_len += 1;
                                term_stack.push(-1.0);
                                // special token to tell calculate_level
                                //  so that -1 will be multiplied on the first pass
                                op_stack.push(Token::NegativeMult);
                            }
                            // go a level deeper
                            Token::OpenCurl => {
                                len_stack.push(level_len);
                                level_len = 0;
                                self.stack.push(Rule::CloseCurl);
                                self.stack.push(Rule::Term);
                            },
                            Token::OpenSquare => {
                                len_stack.push(level_len);
                                level_len = 0;
                                self.stack.push(Rule::CloseSquare);
                                self.stack.push(Rule::Term);
                            },
                            Token::OpenParen => {
                                len_stack.push(level_len);
                                level_len = 0;
                                self.stack.push(Rule::CloseParen);
                                self.stack.push(Rule::Term);
                            },
                            _ => { return Err(()) }
                        }
                    }
                },
                Some(Rule::CloseCurl) => {
                    if let Some(lexical_unit) = self.tokens.front() {
                        if [Token::Plus,
                            Token::Dash,
                            Token::Star,
                            Token::Slash,
                            Token::Carrot,
                            Token::Percent].contains(&lexical_unit.get_token()) {
                            self.stack.push(Rule::CloseCurl);
                            self.stack.push(Rule::Term);
                            op_stack.push(lexical_unit.get_token());
                        }
                        else { 
                            let level_result = self.calculate_level(&mut term_stack, &mut op_stack, level_len); 
                            if let Ok(level) = level_result {
                                term_stack.push(level);
                            }
                            else { return Err(()) }
                        }
                    }
                    else {
                        let level_result = self.calculate_level(&mut term_stack, &mut op_stack, level_len); 
                        if let Ok(level) = level_result {
                            term_stack.push(level);
                        }
                        else { return Err(()) }
                    }
                },
                Some(Rule::CloseSquare) => {
                    if let Some(lexical_unit) = self.tokens.front() {
                        if [Token::Plus,
                            Token::Dash,
                            Token::Star,
                            Token::Slash,
                            Token::Carrot,
                            Token::Percent].contains(&lexical_unit.get_token()) {
                            self.stack.push(Rule::CloseSquare);
                            self.stack.push(Rule::Term);
                            op_stack.push(lexical_unit.get_token());
                        }
                        else { 
                            let level_result = self.calculate_level(&mut term_stack, &mut op_stack, level_len); 
                            if let Ok(level) = level_result {
                                term_stack.push(level);
                            }
                            else { return Err(()) }
                        }
                    }
                    else {
                        let level_result = self.calculate_level(&mut term_stack, &mut op_stack, level_len); 
                        if let Ok(level) = level_result {
                            term_stack.push(level);
                        }
                        else { return Err(()) }
                    }
                },
                Some(Rule::CloseParen) => {
                    if let Some(lexical_unit) = self.tokens.front() {
                        if [Token::Plus,
                            Token::Dash,
                            Token::Star,
                            Token::Slash,
                            Token::Carrot,
                            Token::Percent].contains(&lexical_unit.get_token()) {
                            self.stack.push(Rule::CloseParen);
                            self.stack.push(Rule::Term);
                            op_stack.push(lexical_unit.get_token());
                        }
                        else { 
                            let level_result = self.calculate_level(&mut term_stack, &mut op_stack, level_len); 
                            if let Ok(level) = level_result {
                                term_stack.push(level);
                            }
                            else { return Err(()) }
                        }
                    }
                    else {
                        let level_result = self.calculate_level(&mut term_stack, &mut op_stack, level_len); 
                        if let Ok(level) = level_result {
                            term_stack.push(level);
                        }
                        else { return Err(()) }
                    }
                },
                None => { return Err(()) },
                _ => { return Err(()) },
            }
        }
    }
    fn calculate_level(&mut self, term_stack: &mut Vec<f64>, op_stack: &mut Vec<Token>, mut level_len: usize) -> Result<f64, ()> {
        let mut level_op_stack = VecDeque::new();
        let mut level_term_stack = VecDeque::new();

        if let Some(term) = term_stack.pop() {
            level_term_stack.push_front(term);
        }
        else { return Err(()) }
        // add all terms and ops for expression
        for _ in 1..level_len {
            if let Some(term) = term_stack.pop() {
                level_term_stack.push_front(term);
            }
            else { return Err(()) }
            if let Some(op) = op_stack.pop() {
                level_op_stack.push_front(op);
            }
            else { return Err(()) }
        }
        // perform level math
        //  do loops for each operator in order of precidence
        // negative mult (make terms negative)
        let mut i: usize = 1;
        while i < level_len {
            if Token::NegativeMult == level_op_stack[i] {
                level_term_stack[i-1] = -1.0 * (level_term_stack[i]);
                level_term_stack.remove(i);
                level_op_stack.remove(i);
                level_len -= 1;
            }
            else { i +=1; }
        }
        // exponentials
        let mut i = 1;
        while i < level_len {
            if Token::Carrot == level_op_stack[i] {
                // perform operation
                level_term_stack[i-1] = level_term_stack[i-1].powf(level_term_stack[i]);
                // reduce term and op stacks by 1
                level_term_stack.remove(i);
                level_op_stack.remove(i);
                // reduce level length
                level_len -= 1;
            }
            else { i +=1; }
        }
        // mult, div, mod
        let mut i = 1;
        while i < level_len {
            if Token::Star == level_op_stack[i] {
                level_term_stack[i-1] *= level_term_stack[i];
                level_term_stack.remove(i);
                level_op_stack.remove(i);
                level_len -= 1;
            }
            else if Token::Slash == level_op_stack[i] {
                level_term_stack[i-1] /= level_term_stack[i];
                level_term_stack.remove(i);
                level_op_stack.remove(i);
                level_len -= 1;
            }
            else if Token::Percent == level_op_stack[i] {
                level_term_stack[i-1] = level_term_stack[i-1].rem_euclid(level_term_stack[i]);
                level_term_stack.remove(i);
                level_op_stack.remove(i);
                level_len -= 1;
            }
            else { i +=1; }
        }
        // plus, minus
        let mut i = 1;
        while i < level_len {
            if Token::Plus == level_op_stack[i] {
                level_term_stack[i-1] += level_term_stack[i];
                level_term_stack.remove(i);
                level_op_stack.remove(i);
                level_len -= 1;
            }
            else if Token::Dash == level_op_stack[i] {
                level_term_stack[i-1] -= level_term_stack[i];
                level_term_stack.remove(i);
                level_op_stack.remove(i);
                level_len -= 1;
            }
            else { i +=1; }
        }
        // TODO check the math operations
        if level_len != 1 {
            println!("remaining length is not 1 but: {:?}", level_len);
        }
        if let Some(term) = level_term_stack.pop_front() {
            Ok(term)
        }
        else { Err(()) }
    }
    fn parse_num(&mut self) -> Result<String, ()> {
        let mut num = "".to_string();
        if let Some(digit) = self.tokens.front() {
            if digit.get_token() == Token::Dot {
                self.tokens.pop_front();
                num.push('.');
                if let Ok(result) = self.parse_decimal() {
                    num.push_str(&result);
                    Ok(num)
                }
                else { Err(()) }
            }
            else { Ok(num) }
        }
        else { Err(()) }
    }
    fn parse_decimal(&mut self) -> Result<String, ()> {
        let mut num = "".to_string();
        if let Some(digit) = self.tokens.front() {
            if digit.get_token() == Token::Num {
                let digit = digit.get_lexime().clone();
                self.tokens.pop_front();
                num.push_str(&digit);
                Ok(num)
            }
            else { Ok(num) }
        }
        else { Err(()) }
    }
}

#[derive(PartialEq)]
enum Status {
    LineError,
    Done
}

#[derive(Debug, Clone, PartialEq)]
enum Rule {
    // start rule
    Config,

    // individule assignment
    Line,

    // assignment types
    Path,
    Int,
    Float,

    // intermediate for a sub math block/number
    Term,

    // terminals
    CloseComm,
    CloseCurl,
    CloseSquare,
    CloseParen,
    Op,
    Asn,
    EndL,
}
