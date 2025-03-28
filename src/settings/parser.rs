// TODO remove debug println!s
use super::Settings;
use super::lexer::LexicalUnit;
use super::lexer::Token;
use std::collections::VecDeque;
use std::result;
use std::vec;

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
            // TODO implement real line error func
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

    fn parse_path(&mut self) -> Option<String> {
        if let Err(()) = self.parse_asn() { return None }
        let path;
        // TODO implement
        path = "".to_string();
        
        if let Err(()) = self.parse_to_endl() { return None }
        Some(path)
    }
    // TODO change these to automatically be in a math block
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
                        int = result[result.len()-1] as i32;
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
                        float = result[result.len()-1];
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

    //  lines: 285 - 518 |NOTE| keep up to date
    fn parse_math(&mut self) -> Result<Vec<f64>, ()> {
        self.stack.push(Rule::CloseCurl);
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
            if let Err(()) = self.parse_clear_garbage() { return Err(()) }
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
                                if let Ok(result) = self.parse_num() {
                                    number.push_str(&result);
                                    if let Ok(parsed_number) = number.parse::<f64>() { 
                                        term_stack.push(parsed_number);
                                        term_len_stack.push(0);
                                    }
                                    else { return Err(()) }
                                }
                                else { return Err(()) }
                            },
                            // decimal term
                            Token::Dot => {
                                if let Ok(result) = self.parse_decimal() {
                                    number.push_str(&result);
                                    if let Ok(parsed_number) = number.parse::<f64>() { 
                                        term_stack.push(parsed_number);
                                        term_len_stack.push(0);
                                    }
                                    else { return Err(()) }
                                }
                                else { return Err(()) }
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
                            _ => { return Err(()) }
                        }
                    }
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
                            if lexical_unit.get_token() != Token::CloseCurl { return Err(())}
                            // calculate the value of the bracketed expression
                            let level_result = self.calculate_level(&mut term_stack, &mut term_len_stack, &mut op_stack, level_len); 
                            // if the result is valid push it to the stack
                            //  5 + (2 * 3) -> 5 + 6
                            if let Ok(level_stack) = level_result {
                                let len = level_stack.len();
                                for term in level_stack {
                                    term_stack.push(term);
                                    term_len_stack.push(0);
                                }
                                if len > 1 {
                                    term_len_stack.push(len);
                                }
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
                                        return Err(())
                                    }
                                    if len != 0 && term_stack.len() != len {
                                        return Err(())
                                    }
                                    if op_stack.len() != 0 {
                                        return Err(())
                                    }
                                    return Ok(term_stack)
                                }
                                else{ return Err(()) }
                            }
                        }
                    }
                    else { return Err(()) }
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
                            if lexical_unit.get_token() != Token::CloseParen { return Err(())}
                            let level_result = self.calculate_level(&mut term_stack, &mut term_len_stack, &mut op_stack, level_len); 
                            if let Ok(level_stack) = level_result {
                                let len = level_stack.len();
                                for term in level_stack {
                                    term_stack.push(term);
                                    term_len_stack.push(0);
                                }
                                if len > 1 {
                                    term_len_stack.push(len);
                                }
                            }
                            if let Some(len) = len_stack.pop() {
                                level_len = len;
                            }
                            // math block always ends with a close curly
                            else { return Err(()) }
                        }
                    }
                    else { return Err(()) }
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
                            else { return Err(()) }
                        }
                        else {
                            self.stack.push(Rule::CloseSquare);
                            self.stack.push(Rule::Term);
                            list_len += 1;
                        }
                    }
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
                            else { return Err(()) }
                        }
                        else {
                            self.stack.push(Rule::CloseSquare);
                            self.stack.push(Rule::Term);
                            list_len += 1;
                        }
                    }
                },
                None => { return Err(()) },
                _ => { 
                    // protection for further lines
                    if let Some(rule) = rule {
                        self.stack.push(rule);
                    }
                    return Err(()) 
                },
            }
        }
    }
    //  lines: 519 - 843 |NOTE| keep up to date
    fn calculate_level(&mut self, term_stack: &mut Vec<f64>, term_len_stack: &mut Vec<usize>, op_stack: &mut Vec<Token>, level_len: usize) -> Result<Vec<f64>, ()> {
        let mut level_op_stack = VecDeque::new();
        let mut level_term_stack = VecDeque::new();
        let mut level_term_len_stack = VecDeque::new();

        if let Ok(_) = self.level_stack_add_term_list(term_stack, &mut level_term_stack, term_len_stack, &mut level_term_len_stack) {}
        else { return Err(()) }

        // add all terms and ops for expression
        for _ in 1..level_len {
            if let Some(op) = op_stack.pop() {
                level_op_stack.push_front(op);
            }
            else { return Err(()) }

            if let Ok(_) = self.level_stack_add_term_list(term_stack, &mut level_term_stack, term_len_stack, &mut level_term_len_stack) {}
            else { return Err(()) }
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
                            else { return Err(()) }
                        }
                        else { return Err(()) }
                    }
                }
                else { return Err(()) }

                // build term two
                let term_two;
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    term_two = term;
                }
                else { return Err(()) }


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
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    unused_term = term;
                }
                else { return Err(()) }

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
        else { return Err(()) }

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
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    term_one = term;
                }
                else { return Err(()) }

                let term_two;
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    term_two = term;
                }
                else { return Err(()) }

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
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    unused_term = term;
                }
                else { return Err(()) }

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
        else { return Err(()) }

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
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    term_one = term;
                }
                else { return Err(()) }

                let term_two;
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    term_two = term;
                }
                else { return Err(()) }

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
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    unused_term = term;
                }
                else { return Err(()) }

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
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    term_one = term;
                }
                else { return Err(()) }

                let term_two;
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    term_two = term;
                }
                else { return Err(()) }

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
                if let Ok(term) = self.build_list_term(&mut level_term_stack, &mut level_term_len_stack) {
                    unused_term = term;
                }
                else { return Err(()) }

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
                return Err(())
            }
            if len != 0 && level_term_stack.len() != len {
                return Err(())
            }
            if level_op_stack.len() != 0 {
                return Err(())
            }
            Ok(level_term_stack.into())
        }
        else { Err(()) }
    }
    fn level_stack_add_term_list(&mut self, term_stack: &mut Vec<f64>, level_term_stack: &mut VecDeque<f64>, term_len_stack: &mut Vec<usize>, level_term_len_stack: &mut VecDeque<usize>) -> Result<(), ()> {
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
                else { return Err(()) }
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
                            else { return Err(()) }
                        }
                        // sub-list term item
                        else {
                            current_term_len_stack.push(current_term_len);
                            remaining_len_stack.push(remaining_len);
                            current_term_len = term_len;
                            remaining_len = current_term_len;
                        }
                    }
                    else { return Err(()) }
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
                                else { return Err(()) }
                            }
                        }
                        else { return Err(()) }
                        if let Some(len) = remaining_len_stack.pop() {
                            remaining_len = len;
                        }
                        else { return Err(()) }
                    }
                    // break from outer loop after fully adding list
                    if current_term_len == 0 { break }
                }
            }
        }
        else { return Err(()) }
        Ok(())
    }
    fn build_list_term(&self, level_term_stack: &mut VecDeque<f64>, level_term_len_stack: &mut VecDeque<usize>) -> Result<Vec<f64>, ()> {
        let mut term = vec![];
        if let Some(len) = level_term_len_stack.pop_front() {
            if len == 0 {
                if let Some(element) = level_term_stack.pop_front() {
                    term.push(element);
                }
                else { return Err(()) }
            }
            else{
                for _ in 0..len {
                    if let Some(element) = level_term_stack.pop_front() {
                        term.push(element);
                    }
                    else { return Err(()) }
                    if let None = level_term_len_stack.pop_front() { return Err(()) }
                }
            }
        }
        else { return Err(()) }
        Ok(term)
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

    fn parse_comment(&mut self) -> Status {
        loop {
            if let Some(lexical_unit) = self.tokens.pop_front() {
                if lexical_unit.get_token() == Token::OpenComm { self.stack.push(Rule::CloseComm); }
                else if lexical_unit.get_token() == Token::CloseComm { 
                    if let Some(rule) = self.stack.pop() {
                        if rule != Rule::CloseComm { return Status::LineError }
                    }
                    else { return Status::LineError }

                    if let Some(rule) = self.stack.last() {
                        if rule != &Rule::CloseComm { return Status::Done }
                    }
                    else { return Status::LineError }
                }

            }
            else { return Status::LineError }
        }
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
    CloseSquareNotComma,
    CloseParen,
    Comma,
    Op,
    Asn,
    EndL,
}
