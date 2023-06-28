
pub struct Token {
    pub token_type: String,
    pub token_type_id: usize,
    pub value: String,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value = self.value.clone();

        if self.token_type != "text_literal" {
            value = value.replace(" ", "[Space]");
        }
        value = value.replace("\r\n", "[New Line]");
        value = value.replace("\n", "[New Line]");
        value = value.replace("\r", "[Carriage Return]");
        value = value.replace("\t", "[Tab]");

        f.write_str(format!("token: {}, value: '{}'", self.token_type, value).as_str())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

pub struct TokenClass {
    pub look: String,
    pub token_type: String,
    pub symbol: bool,
    pub write: bool,
}

pub struct Lexer {
    pub token_classes: Vec<TokenClass>,
}

impl Lexer {
    pub fn create(classes: Vec<TokenClass>) -> Self {
        Self {
            token_classes: classes,
        }
    }

    pub fn tokenize(&self, data: &Vec<u8>, lex_errors: Option<&mut Vec<LexerError>>) -> Vec<Token> {
        let mut output = Vec::new();

        let mut temp_errors = Vec::new();
        let errors = if let Some(e) = lex_errors {
            e
        } else {
            &mut temp_errors
        };

        let mut i = 0_usize;
        'main_lex: loop {
            let mut current_char = data[i] as char;

            // Wrote this after the comment under this but i lied... Lets find special shit first...
            match current_char {
                '"' => {
                    let mut comp_text = String::new();
                    loop {
                        i += 1;
                        if i >= data.len() {
                            errors.push(
                                LexerError {
                                    pos: Position { line: 0, column: 0 },
                                    what: String::from("string does not terminate."),
                                }
                            );
                            break;
                        }

                        let curr_text_char = data[i] as char;
                        if curr_text_char == '"' {
                            if i != 0 {
                                if data[i - 1] as char == '\\' {
                                    comp_text.push(curr_text_char);
                                    continue;
                                }
                            }
                            break;
                        } else {
                            comp_text.push(curr_text_char);
                        }
                    }

                    comp_text = comp_text.replace("\\n", "\n");
                    comp_text = comp_text.replace("\\r", "\r");
                    comp_text = comp_text.replace("\\t", "\t");
                    comp_text = comp_text.replace("\\\\", "\\");

                    comp_text = comp_text.replace("\\\"", "\"");
                    comp_text = comp_text.replace("\\\'", "\'");

                    let token = Token {
                        token_type: String::from("text_literal"),
                        token_type_id: 0,
                        value: comp_text
                    };
                    output.push(token);
                    i += 1;
                    if i >= data.len() {
                        break;
                    }
                    continue;
                }

                _ => {}     // Nothing
            }

            // Try to find symbols first
            let mut id = 1_usize;
            for class in &self.token_classes {
                if !class.symbol {continue;}

                if current_char.to_string() == class.look {
                    let token = Token {
                        token_type: class.token_type.clone(),
                        token_type_id: id,
                        value: String::from(current_char)
                    };
                    if class.write {
                        output.push(token);
                    }
                    i += 1;
                    if i >= data.len() {
                        break 'main_lex;
                    }
                    continue 'main_lex; 
                }
                id += 1;
            }

            if !current_char.is_alphanumeric() {
                let token = Token {
                    token_type: String::from("undefined_char"),
                    token_type_id: 0,
                    value: String::from(current_char),
                };

                if current_char != 7 as char { // Filler Char. hopefully nobody uses the BEL character..
                    output.push(token);
                }

                i += 1;
                if i >= data.len() {
                    break;
                }
                continue;
            }

            // Find strings next
            let mut current_string = String::new();
            loop {
                current_char = data[i] as char;
                if ending_char(current_char) {
                    break;
                } else {
                    current_string.push(current_char);
                    i += 1;
                }

                if i >= data.len() {
                    break;
                }
            }

            let token = Token {
                token_type: String::from("undefined_string"),
                token_type_id: 0,
                value: current_string,
            };
            output.push(token);
        }

        output
    }
}

use std::error::Error;
#[derive(Debug)]
pub struct LexerError {
    pos: Position,
    what: String
}

impl Error for LexerError {}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("lex[{}, {}]: error: {}", self.pos.line, self.pos.column, self.what).as_str())?;
        Ok(())
    }
}

fn ending_char(c: char) -> bool {
    match c {
        ' ' => true,
        '\n' => true,
        '\t' => true,
        '\0' => true,
        _ => {
            if c.is_alphanumeric() {
                false
            } else {
                true
            }
        }
    }
}
