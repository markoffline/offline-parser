pub mod lexer;
pub mod preprocessor;

#[cfg(test)]
mod tests {
    use crate::{lexer::TokenClass, preprocessor::{Preprocessor, LongCommentActivation}};

    use super::*;

    use std::{fs::File, io::Read};

    use lexer::Lexer;

    #[test]
    fn example() {
        let mut file = File::open("example.txt").expect("failed to open example.txt");
        let mut raw_data = vec![];
        file.read_to_end(&mut raw_data).expect("failed to read example.txt");

        let comment_line_activations = Vec::from([String::from("//")]);
        let long_comment_activations = Vec::from([
            LongCommentActivation {
                start: String::from("/*"),
                end: String::from("*/"),
            }
        ]);

        let preprocessor = Preprocessor::create(comment_line_activations, long_comment_activations);
        let mut preprocessor_errors = Vec::new();

        let data = preprocessor.process(&raw_data, Some(&mut preprocessor_errors));

        std::fs::write("Preprocess Output.txt", &data).unwrap();

        let _classes = [
            TokenClass {look: String::from("\n"), token_type: String::from("_new_line"), symbol: true, write: false},
            TokenClass {look: String::from("\r"), token_type: String::from("_carriage_return"), symbol: true, write: false},
            TokenClass {look: String::from(" "), token_type: String::from("_space"), symbol: true, write: false},
        ];
        let classes = Vec::from(_classes);
        
        let mut lexer_errors = Vec::new();
        let lexer = Lexer::create(classes);
        let tokens = lexer.tokenize(&data, &mut Some(lexer_errors));

        let mut token_output = String::new();
        token_output += &format!("TOKEN(S) ({} in total): \n", tokens.len());
        for token in &tokens {
            token_output += &format!("\t{}\n", token);
        }

        std::fs::write("Lexer Output.txt", &token_output).unwrap();

    }
}
