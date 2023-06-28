use crate::lexer::Position;


#[derive(Default)]
pub struct Preprocessor {
    pub line_comment_activators: Vec<String>,
    pub long_comment_activators: Vec<LongCommentActivation>,
}

impl Preprocessor {
    pub fn create(line_comment_activators: Vec<String>, long_comment_activators: Vec<LongCommentActivation>) -> Self {
        Preprocessor {
            line_comment_activators: line_comment_activators,
            long_comment_activators: long_comment_activators,
        }
    }

    pub fn process(&self, raw_data: &Vec<u8>, pre_errors: Option<&mut Vec<PreprocessorError>>) -> Vec<u8> {
        let mut st = String::from_utf8(raw_data.to_vec()).expect("failed to read a file! make sure it is UTF-8!");

        let mut temp_errors = Vec::new();
        let errors = if let Some(e) = pre_errors {
            e
        } else {
            &mut temp_errors
        };

        // Convert potental CRLF file to LF
        st = st.replace("\r\n", "\n");

        let mut temp_st = st.clone();

        struct CommentStart {
            w: usize,
            range: usize,
        }

        let mut comments = Vec::new();

        // Remove comments like this one
        for line_ch in &self.line_comment_activators {
            loop { // Loop until error.
                if let Some(comment_position) = temp_st.find(line_ch.as_str()) {
                    let mut last_char = '\0';
                    let mut range = comment_position;
                    while last_char != '\n' {
                        range += 1;
                        last_char = if let(Some(last)) = temp_st.chars().nth(range) {
                            last
                        } else {
                            '\n'
                        };
                    }
                    comments.push(
                        CommentStart {
                            w: comment_position,
                            range: range
                        }
                    );
                    replace_with(&mut temp_st, 7 as char, comment_position, range).expect("failed to remove text from a string!");
                } else {
                    break;
                }
            }
        }

        /* Remove Comments like these ones */
        'comment: for act in &self.long_comment_activators {
            let start_ch = act.start.clone();
            let end_ch = act.end.clone();

            loop { // Loop until error.
                if let Some(comment_position) = temp_st.find(start_ch.as_str())  {
                    let mut recorded_string = String::from("");
                    let mut range = comment_position + start_ch.len();

                    while !recorded_string.contains(&end_ch) {
                        let cur_char = if let(Some(ch)) = temp_st.chars().nth(range) {
                            ch
                        } else {
                            errors.push(
                                PreprocessorError {
                                    pos: Position { line: 0, column: 0 },
                                    what: String::from("comment does not terminate.")
                                }
                            );
                            break 'comment;
                        };
                        recorded_string.push(cur_char);
                        range += 1;
                    }
                    
                    comments.push(
                        CommentStart {
                            w: comment_position,
                            range: range
                        }
                    );
                    replace_with(&mut temp_st, 7 as char, comment_position, range).expect("failed to remove text from a string!");
                } else {
                    break;
                }
            }
        }



        //let mut data = st.into_bytes();
        let mut data = Vec::new();
        let mut i = 0_usize;
        let mut in_literal = false;
        loop {
            for comment in &comments {
                if comment.w == i && !in_literal {
                    replace_with(&mut st, 7 as char, comment.w, comment.range).expect("failed to remove text from a string!");
                }
            }
            // Make sure we dont overflow
            if i >= st.len() {
                break;
            }

            let current_char = st.chars().nth(i).unwrap();
            if current_char == '"' || current_char == '\'' {
                let mut do_work = true;
                if i != 0 {
                    if st.chars().nth(i - 1).unwrap() == '\\' {
                        do_work = false;
                    }

                    if do_work {
                        in_literal = !in_literal;
                    }

                    data.push(current_char as u8);
                }
            } else {
                if in_literal {
                    data.push(current_char as u8);
                } else {
                    data.push(temp_st.chars().nth(i).unwrap() as u8);
                }
            }



            i += 1;
            if i >= st.len() {
                break;
            }
        }
        

        data
    }
}


use std::error::Error;
use std::fmt::Display;
use std::mem::replace;
#[derive(Debug)]
pub struct PreprocessorError {
    pos: Position,
    what: String,
}

impl Display for PreprocessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("preprocessor[{}, {}]: error: {}", self.pos.line, self.pos.column, self.what).as_str())?;
        Ok(())
    }
}

impl Error for PreprocessorError {}

pub struct LongCommentActivation {
    pub start:  String,
    pub end:    String
}

fn remove_chars(string: &mut String, start: usize, end: usize) -> Result<String, String> {
    if start >= string.len() || end > string.len() {
        return Err(String::from("Invalid indices"));
    }

    let removed = string
        .drain(start..end)
        .collect::<String>();

    Ok(removed)
}

fn replace_with(string: &mut String, what: char, start: usize, end: usize) -> Result<(), String> {
    if start >= string.len() || end > string.len() {
        return Err(String::from("Invalid indices"));
    }

    let num_replacements = end - start;
    let replacement = what.to_string().repeat(num_replacements);

    string.replace_range(start..end, &replacement);

    Ok(())
}
