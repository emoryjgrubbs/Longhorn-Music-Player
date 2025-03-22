use super::Settings;
use super::lexer::LexicalUnit;
use std::collections::VecDeque;

pub struct Parser<'p> {
    settings: &'p mut Settings<'p>,
    sub_files: Vec<String>,
    stack: Vec<Rule>,
    tokens: VecDeque<LexicalUnit>
}
impl<'p> Parser <'_> {
    pub fn process_tokens(settings: &'p mut Settings<'p>, tokens: VecDeque<LexicalUnit>) {
        let mut parser = Parser { settings, sub_files: vec![], stack: vec![Rule::Config], tokens };
        // for error reporting purposes
        let mut line_number = 0;
        let mut error_lines = vec![];
        // loop over all elements in the tokens vec
        loop {
            let rule = parser.stack.pop();
            match rule {
                Some(Rule::Config) => {
                    if parser.tokens.is_empty() {
                        // stop parsing
                        break
                    }
                    else {
                        // try to keep parsing lines
                        parser.stack.push(Rule::Config);
                        parser.stack.push(Rule::Line);
                    }
                },
                Some(Rule::Line) => {
                    if let Some(_) = parser.tokens.front() {
                    }
                    else {
                        error_lines.push(line_number);
                    }

                    line_number += 1;
                },
                _ => { error_lines.push(line_number); }
            }
        }
    }
}

enum Rule {
    // start rule
    Config,

    // individule assignment
    Line,

    // top level rules
    File,
    LibPath,
    MaxHist,
    TopSngLen,
    TopAlbLen,
    TopArtLen,
    TopDecay,

    // path type rules
    PathAsn,
    // int type rules
    // float type rules
    
    // terminals
    //TermF,
    //TermLP,
    //TermMH,
    //TermTSL,
    //TermTAlbL,
    //TermTArtL,
    //TermTD,
    //// list
    //TermOS,
    //TermCS,
    //// math
    //TermOCl,
    //TermCCl,
    //// parenthesis
    //TermOP,
    //TermCP,
    //// comments
    //TermOCm,
    //TermCCm,
    //// ops
    //TermAsn,
    //TermPlus,
    //TermDash,
    //TermStar,
    //TermSlash,
    //TermCarrot,
    //TermPercent,
    //TermDot,
    //TermEsc,
    //TermEndL,
    //// lexime needed
    //TermNum,
    //TermString,
    //TermWS,
}
