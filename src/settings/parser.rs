use super::Settings;
use super::lexer::LexicalUnit;
use super::lexer::Token;
use std::collections::VecDeque;
use std::vec;

#[derive(Debug)]
pub struct Error {
    line_number: i32,
    message: String,
}
impl Error {
    pub fn get_line(&self) -> i32 {
        self.line_number.clone()
    }
    pub fn get_message(&self) -> String{
        self.message.clone()
    }
}

pub struct Parser<'p> {
    settings: &'p mut Settings,
    sub_files: &'p mut Vec<String>,
    stack: Vec<Rule>,
    tokens: VecDeque<LexicalUnit>,
    line_number: i32,
    errors: Vec<Error>
}
impl Parser<'_> {
    pub fn process_tokens(settings: &mut Settings, sub_files: &mut Vec<String>, tokens: VecDeque<LexicalUnit>) -> Result<Vec<Error>, ()> {
        let mut parser = Parser { settings, sub_files, stack: vec![Rule::Config], tokens , line_number: 1, errors: vec![] };
        loop {
            let rule = parser.stack.pop();
            let result = parser.parse_line(rule);
            if let Some(Status::LineError(message)) = result.clone() {
                let error = Error { line_number: parser.line_number, message };
                parser.line_number += 1;
                println!("{:3}. Syntax Error: {}", &error.line_number, &error.message);
                parser.errors.push(error);
                // clear stack of remaining rules from the error line
                while let Some(rule) = parser.stack.pop() {
                    if rule == Rule::Config { parser.stack.push(Rule::Config); break }
                }
                // advance past tokens in the error line
                while let Some(lexical_unit) = parser.tokens.pop_front() {
                    // continue to igore comments (comments can allow statements to cross lines)
                    if lexical_unit.get_token() == Token::OpenComm { parser.stack.push(Rule::CloseComm); parser.parse_comment(); }
                    if lexical_unit.get_token() == Token::EndL { break }
                }
            }
            if result == Some(Status::Done) { break }
        }
        Ok(parser.errors)
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
                            let path_result = self.parse_path();
                            match path_result {
                                Ok(paths) => {
                                    for path in paths {
                                        println!("{:3}. New File: \"{}\"", &self.line_number, &path);
                                        self.sub_files.push(path);
                                    }
                                    self.line_number += 1;
                                    None
                                },
                                Err(message) => {
                                    Some(Status::LineError(message))
                                },
                            }
                        },
                        Token::LibPath => {
                            let path_result = self.parse_path();
                            match path_result {
                                Ok(paths) => {
                                    for path in paths {
                                        println!("{:3}. New Lib Path: \"{}\"", &self.line_number, &path);
                                        self.settings.library_paths.push(path);
                                    }
                                    self.line_number += 1;
                                    None
                                },
                                Err(message) => {
                                    Some(Status::LineError(message))
                                },
                            }
                        },
                        Token::MaxHist => {
                            let int_result = self.parse_int();
                            match int_result {
                                Ok(number) => {
                                    println!("{:3}. New Max Hist: {}", &self.line_number, &number);
                                    self.settings.max_history = number;
                                    self.line_number += 1;
                                    None
                                },
                                Err(message) => {
                                    Some(Status::LineError(message))
                                },
                            }
                        },
                        Token::TopSngLen => {
                            let int_result = self.parse_int();
                            match int_result {
                                Ok(number) => {
                                    println!("{:3}. New Top Sng Len: {}", &self.line_number, &number);
                                    self.settings.top_songs_len = number;
                                    self.line_number += 1;
                                    None
                                },
                                Err(message) => {
                                    Some(Status::LineError(message))
                                },
                            }
                        },
                        Token::TopAlbLen => {
                            let int_result = self.parse_int();
                            match int_result {
                                Ok(number) => {
                                    println!("{:3}. New Top Alb Len: {}", &self.line_number, &number);
                                    self.settings.top_albums_len = number;
                                    self.line_number += 1;
                                    None
                                },
                                Err(message) => {
                                    Some(Status::LineError(message))
                                },
                            }
                        },
                        Token::TopArtLen => {
                            let int_result = self.parse_int();
                            match int_result {
                                Ok(number) => {
                                    println!("{:3}. New Top Art Len: {}", &self.line_number, &number);
                                    self.settings.top_artists_len = number;
                                    self.line_number += 1;
                                    None
                                },
                                Err(message) => {
                                    Some(Status::LineError(message))
                                },
                            }
                        },
                        Token::TopDecay => {
                            let float_result = self.parse_float();
                            match float_result {
                                Ok(number) => {
                                    println!("{:3}. New Top Decay Rate: {}", &self.line_number, &number);
                                    self.settings.top_decay = number;
                                    self.line_number += 1;
                                    None
                                },
                                Err(message) => {
                                    Some(Status::LineError(message))
                                },
                            }
                        },
                        // empty line
                        Token::EndL => {
                            println!("{:3}. Empty", &self.line_number);
                            self.line_number += 1;
                            None
                        }
                        _ => {
                            Some(Status::LineError("Unexpected Token While Parsing Start of Line".to_string()))
                        }
                    }
                }
                else { 
                    Some(Status::LineError("Unexpected End of File While Parsing Line".to_string()))
                }
            },
            _ => {
                Some(Status::LineError("Unexpected Rule While Parsing Line".to_string()))
            },
        }
    }

    fn parse_path(&mut self) -> Result<Vec<String>, String> {
        if let Err(message) = self.parse_asn() { return Err(message) }

        self.stack.push(Rule::Char);

        let mut paths = vec!["".to_string()];
        let mut list_items: Vec<String> = vec![];
        let mut list_item = vec!["".to_string()];
        let mut not_comma = false;
        let mut pos_ws = "".to_string();

        loop {
            // get rule
            let rule = self.stack.pop();
            match rule {
                // looking for a standard path segment
                Some(Rule::Char) => {
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        let token = lexical_unit.get_token();
                        match &token {
                            // found the end of the line/statement
                            Token::EndL => { return Ok(paths) }
                            // escaped characters are all just the lexime associated
                            Token::Esc => {
                                self.stack.push(Rule::EscapedChar);
                            }
                            // add whitespace to special string to possibly be added
                            Token::WhiteSpace => {
                                self.stack.push(Rule::Char);
                                pos_ws.push_str(&lexical_unit.get_lexime());
                            }
                            // ignore comment and keep parsing
                            Token::OpenComm => {
                                self.stack.push(Rule::Char);
                                self.stack.push(Rule::CloseComm);
                                self.parse_comment();
                            }
                            // move to parsing path segments as elements of a list
                            Token::OpenSquare => {
                                if let Err(message) = self.parse_clear_garbage() { return Err(message) }
                                self.stack.push(Rule::Char);
                                self.stack.push(Rule::CloseSquare);
                            }
                            // enter math block, and append the parsed results
                            Token::OpenCurl => {
                                self.stack.push(Rule::Char);
                                self.stack.push(Rule::CloseCurl);
                                let math_return = self.parse_math();
                                match math_return {
                                    Ok(math) => {
                                        let mut new_paths = vec![];
                                        for path in paths {
                                            for number in &math {
                                                let string_number = number.to_string();
                                                let new_path = path.clone() + &string_number;
                                                new_paths.push(new_path);
                                            }
                                        }
                                        paths = new_paths;
                                    }
                                    Err(message) => { return Err(message) }
                                }
                            }
                            // anything else is a path character and should have its lexime added
                            _ => {
                                self.stack.push(Rule::Char);
                                let chars = lexical_unit.get_lexime();
                                for path in &mut paths {
                                    path.push_str(&pos_ws);
                                    path.push_str(&chars);
                                }
                                pos_ws.clear();
                            }
                        }
                    }
                    else { return Err("Unexpected End of File While Parsing Path".to_string()) }
                },
                // just add lexime to path
                Some(Rule::EscapedChar) => {
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        self.stack.push(Rule::Char);
                        let chars = lexical_unit.get_lexime();
                        for path in &mut paths {
                            path.push_str(&pos_ws);
                            path.push_str(&chars);
                        }
                        pos_ws.clear();
                    }
                },
                // parse path segments as a list
                Some(Rule::CloseSquare) => {
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        let token = lexical_unit.get_token();
                        match &token {
                            // the end of a line cannot appear in a list
                            //  NOTE may just increment line count to allow lists to be split over
                            //  multiple lines
                            Token::EndL => { self.tokens.push_front(lexical_unit); return Err("Unexpected New-Line in Path List".to_string()) }
                            Token::Esc => {
                                self.stack.push(Rule::CloseSquareEscaped);
                            }
                            Token::WhiteSpace => {
                                self.stack.push(Rule::CloseSquare);
                                pos_ws.push_str(&lexical_unit.get_lexime());
                            }
                            Token::OpenComm => {
                                self.stack.push(Rule::CloseSquare);
                                self.stack.push(Rule::CloseComm);
                                self.parse_comment();
                            }
                            Token::OpenSquare => {
                                not_comma = false;
                                if let Err(message) = self.parse_clear_garbage() { return Err(message) }
                                self.stack.push(Rule::CloseSquare);
                            }
                            Token::Comma => {
                                // simplify readability by combining the rules for lists
                                //  allowing commas and disallowing
                                if not_comma { return Err("Consecutive Commas in Path List".to_string()) }
                                pos_ws.clear();
                                if let Err(message) = self.parse_clear_garbage() { return Err(message) }
                                list_items.append(&mut list_item);
                                list_item = vec!["".to_string()];
                                not_comma = true;
                                self.stack.push(Rule::CloseSquare);
                            },
                            Token::CloseSquare => {
                                pos_ws.clear();
                                not_comma = false;
                                let next_rule = self.stack.last();
                                if next_rule != Some(&Rule::CloseSquare) && next_rule != Some(&Rule::CloseSquareNotComma) {
                                    if list_item.len() == 1 && list_item[0] != "".to_string() { list_items.append(&mut list_item); }
                                    let mut new_paths = vec![];
                                    for path in paths {
                                        for list_item in &list_items {
                                            let new_path = path.clone() + &list_item;
                                            new_paths.push(new_path);
                                        }
                                    }
                                    paths = new_paths;
                                    list_items.clear();
                                    list_item = vec!["".to_string()];
                                }
                                else { if let Err(message) = self.parse_clear_garbage() { return Err(message) } }
                            },
                            Token::OpenCurl => {
                                not_comma = false;
                                self.stack.push(Rule::CloseSquare);
                                self.stack.push(Rule::CloseCurl);
                                let math_return = self.parse_math();
                                match math_return {
                                    Ok(math) => {
                                        let mut new_list_items = vec![];
                                        for list_item in list_items {
                                            for number in &math {
                                                let string_number = number.to_string();
                                                let new_path = list_item.clone() + &string_number;
                                                new_list_items.push(new_path);
                                            }
                                        }
                                        list_items = new_list_items;
                                    },
                                    Err(message) => { return Err(message) },
                                }
                            }
                            _ => {
                                not_comma = false;
                                self.stack.push(Rule::CloseSquare);
                                let chars = lexical_unit.get_lexime();
                                for path_seg in &mut list_item {
                                    path_seg.push_str(&pos_ws);
                                    path_seg.push_str(&chars);
                                }
                                pos_ws.clear();
                            }
                        }
                    }
                    else { return Err("Unexpected End of File While Parsing Path".to_string()) }
                },
                Some(Rule::CloseSquareEscaped) => {
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        not_comma = false;
                        self.stack.push(Rule::CloseSquare);
                        let chars = lexical_unit.get_lexime();
                        for path_seg in &mut list_item {
                            path_seg.push_str(&pos_ws);
                            path_seg.push_str(&chars);
                        }
                        pos_ws.clear();
                    }
                },
                _ => { return Err("Unexpected End of File While Parsing Path".to_string()) }
            }
        }
    }

    fn parse_int(&mut self) -> Result<i32, String> {
        if let Err(message) = self.parse_asn() { return Err(message)}
        let int;
        self.stack.push(Rule::CloseMath);
        match self.parse_math() {
            Ok(number) => {
                int = number[number.len()-1].round() as i32;
            },
            Err(message) => { return Err(message) },
        }
        if let Err(message) = self.parse_to_endl() { return Err(message) }
        Ok(int)
    }
    fn parse_float(&mut self) -> Result<f64, String> {
        if let Err(message) = self.parse_asn() { return Err(message)}
        let float;
        self.stack.push(Rule::CloseMath);
        match self.parse_math() {
            Ok(number) => {
                float = number[number.len()-1];
            },
            Err(message) => { return Err(message) },
        }
        if let Err(message) = self.parse_to_endl() { return Err(message) }
        Ok(float)
    }

    //  lines: 431 - 711 |NOTE| keep up to date
    fn parse_math(&mut self) -> Result<Vec<f64>, String> {
        self.stack.push(Rule::Term);

        // operators
        let mut op_stack = vec![];
        // terms
        let mut term_stack = vec![];
        // number of terms (used with lists)
        let mut term_len_stack = vec![];
        // number of terms in the current list
        let mut list_len_stack = vec![];
        // length of levels
        let mut len_stack = vec![];

        let mut list_len = 1;
        let mut level_len = 1;
        loop {
            // remove any comments or white space between terms
            if let Err(message) = self.parse_clear_garbage() { return Err(message) }
            // get rule
            let rule = self.stack.pop();
            match rule {
                // expression needs a term
                Some(Rule::Term) => {
                    // get term token
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        let mut number = lexical_unit.get_lexime();
                        match lexical_unit.get_token() {
                            // number term (may have decimal that is parsed by parse_num)
                            Token::Num => {
                                match self.parse_num() {
                                    Ok(result) => {
                                        number.push_str(&result);
                                        match number.parse::<f64>() {
                                            Ok(parsed_number) => { 
                                                term_stack.push(parsed_number);
                                                term_len_stack.push(0);
                                            },
                                            Err(message) => { return Err(message.to_string()) }
                                        }
                                    },
                                    Err(message) => { return Err(message) }
                                }
                            },
                            // decimal term
                            Token::Dot => {
                                match self.parse_decimal() {
                                    Ok(result) => {
                                        number.push_str(&result);
                                        match number.parse::<f64>() {
                                            Ok(parsed_number) => { 
                                                term_stack.push(parsed_number);
                                                term_len_stack.push(0);
                                            },
                                            Err(message) => { return Err(message.to_string()) }
                                        }
                                    },
                                    Err(message) => { return Err(message) }
                                }
                            },
                            // negative term
                            Token::Dash => {
                                level_len += 1;
                                term_stack.push(-1.0);
                                term_len_stack.push(0);
                                // special token to tell calculate_level
                                //  so that -1 will be multiplied on the first pass
                                op_stack.push(Token::NegativeMult);
                                self.stack.push(Rule::Term);
                            }
                            // list term: add a list to the stack as a term
                            Token::OpenSquare => {
                                list_len_stack.push(list_len);
                                list_len = 1;
                                self.stack.push(Rule::CloseSquare);
                                self.stack.push(Rule::Term);
                            },
                            // go a level deeper
                            Token::OpenCurl => {
                                len_stack.push(level_len);
                                level_len = 1;
                                self.stack.push(Rule::CloseCurl);
                                self.stack.push(Rule::Term);
                            },
                            Token::OpenParen => {
                                len_stack.push(level_len);
                                level_len = 1;
                                self.stack.push(Rule::CloseParen);
                                self.stack.push(Rule::Term);
                            },
                            _ => { return Err("Unexpected Token While Parsing Math Term".to_string()) }
                        }
                    }
                },
                // return from the implicit int/float math block
                Some(Rule::CloseMath) => {
                    // there is another token
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        // the token is a operator (add the ending bracket back to the stack
                        //  along with another term
                        if [Token::Plus,
                            Token::Dash,
                            Token::Star,
                            Token::Slash,
                            Token::Carrot,
                            Token::Percent].contains(&lexical_unit.get_token()) {
                            level_len += 1;
                            self.stack.push(Rule::CloseMath);
                            self.stack.push(Rule::Term);
                            op_stack.push(lexical_unit.get_token());
                        }
                        else { 
                            self.tokens.push_front(lexical_unit);
                            // calculate the value of the bracketed expression
                            let level_result = self.calculate_level(&mut term_stack, &mut term_len_stack, &mut op_stack, level_len); 
                            // if the result is valid push it to the stack
                            //  5 + (2 * 3) -> 5 + 6
                            match level_result {
                                Ok(level_stack) => {
                                    let len = level_stack.len();
                                    for term in level_stack {
                                        term_stack.push(term);
                                        term_len_stack.push(0);
                                    }
                                    if len > 1 {
                                        term_len_stack.push(len);
                                    }
                                }
                                Err(message) => { return Err(message) }
                            }
                            // get the length of that level's expression
                            if let Some(len) = len_stack.pop() {
                                level_len = len;
                            }
                            // if there is no length to pop, check if the math block has reached a
                            //  valid end
                            else {
                                if let Some(len) = term_len_stack.pop() {
                                    if len == 0 && term_stack.len() != 1 {
                                        return Err("Unexpected Term List When Returning From Math".to_string())
                                    }
                                    if len != 0 && term_stack.len() != len {
                                        return Err("Unexpected Number of Terms in List When Returning From Math".to_string())
                                    }
                                    if op_stack.len() != 0 {
                                        return Err("Unexpected Operators in Stack When Returning From Math".to_string())
                                    }
                                    return Ok(term_stack)
                                }
                                else { return Err("Unexpected End of Term Len Stack When Returning From Math".to_string()) }
                            }
                        }
                    }
                    else { return Err("Unexpected End of File While Parsing Math".to_string()) }
                },
                // return from curly brace as a term (may end math block)
                Some(Rule::CloseCurl) => {
                    // there is another token
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        // the token is a operator (add the ending bracket back to the stack
                        //  along with another term
                        if [Token::Plus,
                            Token::Dash,
                            Token::Star,
                            Token::Slash,
                            Token::Carrot,
                            Token::Percent].contains(&lexical_unit.get_token()) {
                            level_len += 1;
                            self.stack.push(Rule::CloseCurl);
                            self.stack.push(Rule::Term);
                            op_stack.push(lexical_unit.get_token());
                        }
                        else { 
                            // check that the needed ending bracket matches the token
                            if lexical_unit.get_token() != Token::CloseCurl { return Err("Unexpected Token When Closing Curly Brace".to_string())}
                            // calculate the value of the bracketed expression
                            let level_result = self.calculate_level(&mut term_stack, &mut term_len_stack, &mut op_stack, level_len); 
                            // if the result is valid push it to the stack
                            //  5 + (2 * 3) -> 5 + 6
                            match level_result {
                                Ok(level_stack) => {
                                    let len = level_stack.len();
                                    for term in level_stack {
                                        term_stack.push(term);
                                        term_len_stack.push(0);
                                    }
                                    if len > 1 {
                                        term_len_stack.push(len);
                                    }
                                }
                                Err(message) => { return Err(message) }
                            }
                            // get the length of that level's expression
                            if let Some(len) = len_stack.pop() {
                                level_len = len;
                            }
                            // if there is no length to pop, check if the math block has reached a
                            //  valid end
                            else {
                                if let Some(len) = term_len_stack.pop() {
                                    if len == 0 && term_stack.len() != 1 {
                                        return Err("Unexpected Term List When Returning From Math".to_string())
                                    }
                                    if len != 0 && term_stack.len() != len {
                                        return Err("Unexpected Number of Terms in List When Returning From Math".to_string())
                                    }
                                    if op_stack.len() != 0 {
                                        return Err("Unexpected Operators in Stack When Returning From Math".to_string())
                                    }
                                    return Ok(term_stack)
                                }
                                else { return Err("Unexpected End of Term Len Stack When Returning From Math".to_string()) }
                            }
                        }
                    }
                    else { return Err("Unexpected End of File While Parsing Math".to_string()) }
                },
                // return from parenthesis as a term
                Some(Rule::CloseParen) => {
                    if let Some(lexical_unit) = self.tokens.pop_front() {
                        if [Token::Plus,
                            Token::Dash,
                            Token::Star,
                            Token::Slash,
                            Token::Carrot,
                            Token::Percent].contains(&lexical_unit.get_token()) {
                            level_len += 1;
                            self.stack.push(Rule::CloseParen);
                            self.stack.push(Rule::Term);
                            op_stack.push(lexical_unit.get_token());
                        }
                        else { 
                            if lexical_unit.get_token() != Token::CloseParen { return Err("Unexpected Token When Closing Parenthesis".to_string())}
                            let level_result = self.calculate_level(&mut term_stack, &mut term_len_stack, &mut op_stack, level_len); 
                            match level_result {
                                Ok(level_stack) => {
                                    let len = level_stack.len();
                                    for term in level_stack {
                                        term_stack.push(term);
                                        term_len_stack.push(0);
                                    }
                                    if len > 1 {
                                        term_len_stack.push(len);
                                    }
                                }
                                Err(message) => { return Err(message) }
                            }
                            if let Some(len) = len_stack.pop() {
                                level_len = len;
                            }
                            // math block always ends with a close curly
                            else { return Err("Unexpected End of Term Len Stack When Closing Parenthesis".to_string()) }
                        }
                    }
                    else { return Err("Unexpected End of File While Parsing Math".to_string()) }
                },
                Some(Rule::CloseSquare) => {
                    if let Some(lexical_unit) = self.tokens.front() {
                        if lexical_unit.get_token() == Token::Comma {
                            self.stack.push(Rule::CloseSquareNotComma);
                            self.tokens.pop_front();
                        }
                        else if lexical_unit.get_token() == Token::CloseSquare {
                            self.tokens.pop_front();
                            term_len_stack.push(list_len);
                            if let Some(len) = list_len_stack.pop() {
                                list_len = len;
                            }
                            else { return Err("Unexpected End of List Len Stack When Closing Square Bracket".to_string()) }
                        }
                        else {
                            self.stack.push(Rule::CloseSquare);
                            self.stack.push(Rule::Term);
                            list_len += 1;
                        }
                    }
                    else { return Err("Unexpected End of File While Parsing Math".to_string()) }
                },
                // prevent empty commas [ ..., , ... ]
                //  while allowing trailing commas [ ..., ]
                Some(Rule::CloseSquareNotComma) => {
                    if let Some(lexical_unit) = self.tokens.front() {
                        if lexical_unit.get_token() == Token::CloseSquare {
                            self.tokens.pop_front();
                            term_len_stack.push(list_len);
                            if let Some(len) = list_len_stack.pop() {
                                list_len = len;
                            }
                            else { return Err("Unexpected End of List Len Stack When Closing Square Bracket".to_string()) }
                        }
                        else {
                            self.stack.push(Rule::CloseSquare);
                            self.stack.push(Rule::Term);
                            list_len += 1;
                        }
                    }
                    else { return Err("Unexpected End of File While Parsing Math".to_string()) }
                },
                None => { return Err("Unexpected End of Rule Stack While Parsing Math".to_string()) },
                _ => { 
                    // protection for further lines
                    if let Some(rule) = rule {
                        self.stack.push(rule);
                    }
                    return Err("Unexpected Rule in Rule Stack While Parsing Math".to_string()) 
                },
            }
        }
    }
    //  lines: 712 - 1057 |NOTE| keep up to date
    fn calculate_level(&mut self, term_stack: &mut Vec<f64>, term_len_stack: &mut Vec<usize>, op_stack: &mut Vec<Token>, level_len: usize) -> Result<Vec<f64>, String> {
        let mut level_op_stack = VecDeque::new();
        let mut level_term_stack = VecDeque::new();
        let mut level_term_len_stack = VecDeque::new();

        if let Err(message) = self.level_stack_add_term_list(term_stack, &mut level_term_stack, term_len_stack, &mut level_term_len_stack) { return Err(message) }

        // add all terms and ops for expression
        for _ in 1..level_len {
            if let Some(op) = op_stack.pop() {
                level_op_stack.push_front(op);
            }
            else { return Err("Unexpected End of Operators While Constructing Level Stack".to_string()) }

            if let Err(message) = self.level_stack_add_term_list(term_stack, &mut level_term_stack, term_len_stack, &mut level_term_len_stack) { return Err(message) }
        }

        // perform level math
        //  do loops for each operator in order of precidence
        // negative mult (make terms negative)
        let mut output_terms =  VecDeque::new();
        let mut output_term_lens = VecDeque::new();
        let mut unused_ops = VecDeque::new();
        while let Some(op) = level_op_stack.pop_front() {
            if op == Token::NegativeMult {
                // ensure term one is -1
                let mut term_one = vec![];
                if let Some(len) = level_term_len_stack.pop_front() {
                    if len == 0 {
                        if let Some(term) = level_term_stack.pop_front() {
                            if term == -1.0 {
                                term_one.push(-1.0);
                            }
                            else { return Err("Unexpected Term While Performing Negative Mult".to_string()) }
                        }
                        else { return Err("Unexpected End of Level Term Stack While Performing Negative Mult".to_string()) }
                    }
                    else { return Err("Unexpected Term Len While Performing Negative Mult".to_string()) }
                }
                else { return Err("Unexpected End of Level Term Len Stack While Performing Negative Mult".to_string()) }

                // build term two
                let term_two;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        term_two = term;
                    },
                    Err(message) => { return Err(message) }
                }


                // perform operation on all elements
                let len = term_two.len();
                if len > 1 { output_term_lens.push_back(len); }
                for element in term_two {
                    output_terms.push_back(-element);
                    output_term_lens.push_back(0);
                }
            }
            else { 
                let unused_term;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        unused_term = term;
                    },
                    Err(message) => { return Err(message) }
                }

                let len = unused_term.len();
                if len > 1 { output_term_lens.push_back(len); }
                for element in unused_term {
                    output_terms.push_back(element);
                    output_term_lens.push_back(0);
                }

                unused_ops.push_back(op);
            }
        }
        // if there is a last unused term
        let unused_term;
        if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
            unused_term = term;
        }
        else { unused_term = vec![]; }

        let len = unused_term.len();
        if len > 1 { output_term_lens.push_back(len); }
        for element in unused_term {
            output_terms.push_back(element);
            output_term_lens.push_back(0);
        }

        level_term_stack = output_terms.clone();
        level_term_len_stack = output_term_lens.clone();
        level_op_stack = unused_ops.clone();

        // exponentials
        output_terms.clear();
        output_term_lens.clear();
        unused_ops.clear();
        while let Some(op) = level_op_stack.pop_front() {
            if op == Token::Carrot {
                let term_one;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        term_one = term;
                    },
                    Err(message) => { return Err(message) }
                }

                let term_two;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        term_two = term;
                    },
                    Err(message) => { return Err(message) }
                }

                // perform operation
                let len = term_one.len() * term_two.len();
                if len > 1 { output_term_lens.push_back(len); }
                for element_one in term_one {
                    for element_two in term_two.clone() {
                        output_terms.push_back(element_one.powf(element_two));
                        output_term_lens.push_back(0);
                    }
                }
            }
            else {
                let unused_term;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        unused_term = term;
                    },
                    Err(message) => { return Err(message) }
                }

                let len = unused_term.len();
                if len > 1 { output_term_lens.push_back(len); }
                for element in unused_term {
                    output_terms.push_back(element);
                    output_term_lens.push_back(0);
                }

                unused_ops.push_back(op);
            }
        }
        // if there is a last unused term
        let unused_term;
        if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
            unused_term = term;
        }
        else { unused_term = vec![]; }

        let len = unused_term.len();
        if len > 1 { output_term_lens.push_back(len); }
        for element in unused_term {
            output_terms.push_back(element);
            output_term_lens.push_back(0);
        }

        level_term_stack = output_terms.clone();
        level_term_len_stack = output_term_lens.clone();
        level_op_stack = unused_ops.clone();

        // mult, div, mod
        output_terms.clear();
        output_term_lens.clear();
        unused_ops.clear();
        while let Some(op) = level_op_stack.pop_front() {
            if [Token::Star, Token::Slash, Token::Percent].contains(&op) {
                let term_one;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        term_one = term;
                    },
                    Err(message) => { return Err(message) }
                }

                let term_two;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        term_two = term;
                    },
                    Err(message) => { return Err(message) }
                }

                // perform operation
                let len = term_one.len() * term_two.len();
                if len > 1 { output_term_lens.push_back(len); }
                if Token::Star == op {
                    for element_one in term_one {
                        for element_two in term_two.clone() {
                            output_terms.push_back(element_one * element_two);
                            output_term_lens.push_back(0);
                        }
                    }
                }
                else if Token::Slash == op {
                    for element_one in term_one {
                        for element_two in term_two.clone() {
                            output_terms.push_back(element_one / element_two);
                            output_term_lens.push_back(0);
                        }
                    }
                }
                else if Token::Percent == op {
                    for element_one in term_one {
                        for element_two in term_two.clone() {
                            output_terms.push_back(element_one.rem_euclid(element_two));
                            output_term_lens.push_back(0);
                        }
                    }
                }
            }
            else {
                let unused_term;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        unused_term = term;
                    },
                    Err(message) => { return Err(message) }
                }

                let len = unused_term.len();
                if len > 1 { output_term_lens.push_back(len); }
                for element in unused_term {
                    output_terms.push_back(element);
                    output_term_lens.push_back(0);
                }

                unused_ops.push_back(op);
            }
        }
        // if there is a last unused term
        let unused_term;
        if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
            unused_term = term;
        }
        else { unused_term = vec![]; }

        let len = unused_term.len();
        if len > 1 { output_term_lens.push_back(len); }
        for element in unused_term {
            output_terms.push_back(element);
            output_term_lens.push_back(0);
        }

        level_term_stack = output_terms.clone();
        level_term_len_stack = output_term_lens.clone();
        level_op_stack = unused_ops.clone();

        // plus, minus
        output_terms.clear();
        output_term_lens.clear();
        unused_ops.clear();
        while let Some(op) = level_op_stack.pop_front() {
            if op == Token::Plus || op == Token::Dash {
                let term_one;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        term_one = term;
                    },
                    Err(message) => { return Err(message) }
                }

                let term_two;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        term_two = term;
                    },
                    Err(message) => { return Err(message) }
                }

                // perform operation
                let len = term_one.len() * term_two.len();
                if len > 1 { output_term_lens.push_back(len); }
                if Token::Plus == op {
                    for element_one in term_one {
                        for element_two in term_two.clone() {
                            output_terms.push_back(element_one + element_two);
                            output_term_lens.push_back(0);
                        }
                    }
                }
                else if Token::Dash == op {
                    for element_one in term_one {
                        for element_two in term_two.clone() {
                            output_terms.push_back(element_one - element_two);
                            output_term_lens.push_back(0);
                        }
                    }
                }
            }
            else {
                let unused_term;
                match self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    Ok(term) =>  {
                        unused_term = term;
                    },
                    Err(message) => { return Err(message) }
                }

                let len = unused_term.len();
                if len > 1 { output_term_lens.push_back(len); }
                for element in unused_term {
                    output_terms.push_back(element);
                    output_term_lens.push_back(0);
                }

                unused_ops.push_back(op);
            }
        }
        // if there is a last unused term
        let unused_term;
        if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
            unused_term = term;
        }
        else { unused_term = vec![]; }

        let len = unused_term.len();
        if len > 1 { output_term_lens.push_back(len); }
        for element in unused_term {
            output_terms.push_back(element);
            output_term_lens.push_back(0);
        }

        level_term_stack = output_terms;
        level_term_len_stack = output_term_lens;
        level_op_stack = unused_ops;

        if let Some(len) = level_term_len_stack.pop_front() {
            if len == 0 && level_term_stack.len() != 1 {
                return Err("Unexpected Term List When Returning From Calculate Level".to_string())
            }
            if len != 0 && level_term_stack.len() != len {
                return Err("Unexpected Number of Terms in List When Returning From Calculate Level".to_string())
            }
            if level_op_stack.len() != 0 {
                return Err("Unexpected Operators in Stack When Returning From Calculate Level".to_string())
            }
            Ok(level_term_stack.into())
        }
        else { Err("Unexpected End of Term Len Stack When Returning From Calculate Level".to_string()) }
    }
    fn level_stack_add_term_list(&mut self, term_stack: &mut Vec<f64>, level_term_stack: &mut VecDeque<f64>, term_len_stack: &mut Vec<usize>, level_term_len_stack: &mut VecDeque<usize>) -> Result<(), String> {
        let mut current_term_len_stack = vec![];
        let mut current_term_len = 0;
        let mut remaining_len_stack = vec![];
        let mut remaining_len;
        // it's only necessary to reverse where list lengths are stored
        //  if left to right expression evalution is desireable
        if let Some(term_len) = term_len_stack.pop() {
            // first term is a single value
            if term_len == 0 {
                level_term_len_stack.push_front(term_len);
                if let Some(term) = term_stack.pop() {
                    level_term_stack.push_front(term);
                }
                else { return Err("Unexpected End of Term Stack While Constructing Level Term".to_string()) }
            }
            // first term is a list
            else {
                current_term_len_stack.push(current_term_len);
                current_term_len = term_len;
                remaining_len_stack.push(current_term_len);
                remaining_len = current_term_len;
                // add entire list term
                loop {
                    remaining_len -= 1;
                    // get a list item and add it to the term list
                    if let Some(term_len) = term_len_stack.pop() {
                        // single term item
                        if term_len == 0 {
                            level_term_len_stack.push_front(term_len);
                            if let Some(term) = term_stack.pop() {
                                level_term_stack.push_front(term);
                            }
                            else { return Err("Unexpected End of Term Stack While Constructing Level List Term".to_string()) }
                        }
                        // sub-list term item
                        else {
                            current_term_len_stack.push(current_term_len);
                            remaining_len_stack.push(remaining_len);
                            current_term_len = term_len;
                            remaining_len = current_term_len;
                        }
                    }
                    else { return Err("Unexpected End of Term Len Stack While Constructing Level List Term".to_string()) }
                    // return from added sub-lists
                    while remaining_len == 0 {
                        level_term_len_stack.push_front(current_term_len);
                        if let Some(len) = current_term_len_stack.pop() {
                            current_term_len = len;
                            // a single item is not a list and cannot have sublists
                            //  this is the bottom of the recursive list term stack
                            if current_term_len == 0 { break }
                            else {
                                if let Some(sub_len) = level_term_len_stack.pop_front() {
                                    current_term_len += sub_len - 1;
                                }
                                else { return Err("Unexpected End of Level Term Len Stack While Constructing Level List Term".to_string()) }
                            }
                        }
                        else { return Err("Unexpected End of Current Term Len Stack While Constructing Level List Term".to_string()) }
                        if let Some(len) = remaining_len_stack.pop() {
                            remaining_len = len;
                        }
                        else { return Err("Unexpected End of Remaining Len Stack While Constructing Level List Term".to_string()) }
                    }
                    // break from outer loop after fully adding list
                    if current_term_len == 0 { break }
                }
            }
        }
        else { return Err("Unexpected End of Term Len Stack While Constructing Level Term".to_string()) }
        Ok(())
    }
    fn build_list_term(&self, level_term_stack: &mut VecDeque<f64>, level_term_len_stack: &mut VecDeque<usize>) -> Result<Vec<f64>, String> {
        let mut term = vec![];
        if let Some(len) = level_term_len_stack.pop_front() {
            if len == 0 {
                if let Some(element) = level_term_stack.pop_front() {
                    term.push(element);
                }
                else { return Err("Unexpected End of Level Term Stack While Constructing Term".to_string()) }
            }
            else{
                for _ in 0..len {
                    if let Some(element) = level_term_stack.pop_front() {
                        term.push(element);
                    }
                    else { return Err("Unexpected End of Level Term Stack While Constructing List Term".to_string()) }
                    if let None = level_term_len_stack.pop_front() { return Err("Unexpected End of Level Term Len Stack While Constructing List Term".to_string()) }
                }
            }
        }
        else { return Err("Unexpected End of Level Term Len Stack While Constructing Term".to_string()) }
        Ok(term)
    }
    fn parse_num(&mut self) -> Result<String, String> {
        let mut num = "".to_string();
        if let Some(digit) = self.tokens.front() {
            if digit.get_token() == Token::Dot {
                self.tokens.pop_front();
                num.push('.');
                match self.parse_decimal() {
                    Ok(result) => {
                        num.push_str(&result);
                        Ok(num)
                    },
                    Err(message) => { return Err(message) }
                }
            }
            else { Ok(num) }
        }
        else { Err("Unexpected End of File While Parsing Num".to_string()) }
    }
    fn parse_decimal(&mut self) -> Result<String, String> {
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
        else { Err("Unexpected End of File While Parsing Decimal".to_string()) }
    }

    fn parse_asn(&mut self) -> Result<(), String> {
        if let Err(message) = self.parse_clear_garbage() { return Err(message) }
        if let Some(lexical_unit) = self.tokens.pop_front() {
            if lexical_unit.get_token() != Token::Asn { return Err("Missing Assignment Token".to_string()) }
        }
        else { return Err("Unexpected End of File While Parsing Asn".to_string()) }
        if let Err(message) = self.parse_clear_garbage() { return Err(message) }
        Ok(())
    }

    fn parse_to_endl(&mut self) -> Result<(), String> {
        if let Err(message) = self.parse_clear_garbage() { return Err(message) }
        if let Some(lexical_unit) = self.tokens.pop_front() {
            if lexical_unit.get_token() != Token::EndL { return Err("Missing End of Line Token".to_string()) }
        }
        else { return Err("Unexpected End of File While Parsing EndL".to_string()) }
        Ok(())
    }

    fn parse_clear_garbage(&mut self) -> Result<(), String> {
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
            else { return Err("Unexpected End of File While Parsing Garbage".to_string()) }
        }
    }

    fn parse_comment(&mut self) -> Status {
        loop {
            if let Some(lexical_unit) = self.tokens.pop_front() {
                if lexical_unit.get_token() == Token::OpenComm { self.stack.push(Rule::CloseComm); }
                else if lexical_unit.get_token() == Token::EndL { self.line_number += 1; }
                else if lexical_unit.get_token() == Token::CloseComm { 
                    if let Some(rule) = self.stack.pop() {
                        if rule != Rule::CloseComm { return Status::LineError("Missing Close Comment Rule on Stack".to_string()) }
                    }
                    else { return Status::LineError("Unexpected End of Rule Stack While Parsing Comment".to_string()) }

                    if let Some(rule) = self.stack.last() {
                        if rule != &Rule::CloseComm { return Status::Done }
                    }
                    else { return Status::LineError("Unexpected End of Rule Stack While Parsing Comment".to_string()) }
                }
            }
            else { return Status::LineError("Unexpected End of File While Parsing Comment".to_string()) }
        }
    }
}
 
#[derive(Clone, PartialEq)]
enum Status {
    LineError(String),
    Done
}

#[derive(Debug, Clone, PartialEq)]
enum Rule {
    // start rule
    Config,

    // individule assignment
    Line,

    // intermediate for path
    Char,
    EscapedChar,
    CloseSquareEscaped,

    // intermediate for a sub math block/number
    Term,
    CloseMath,

    // terminals
    CloseComm,
    CloseCurl,
    CloseSquare,
    CloseSquareNotComma,
    CloseParen,
}
