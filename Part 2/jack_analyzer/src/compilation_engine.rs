use core::panic;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::jack_tokenizer::*;

/// Gets its input from a JackTokenizer, and emits its output to an output file, using parsing
/// routines. Each parsing routine compile_x is responsible for handling all the tokens that make up
/// x, advancing the tokenizer exactly beyond these tokens, and outputting the parsing of x.
struct CompilationEngine {
    tokenizer: JackTokenizer,
    output_file: File,
}

impl CompilationEngine {
    pub fn new(output_file: PathBuf, tokenizer: JackTokenizer) -> Self {
        let output_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_file)
            .expect("Should be able to open output file.");

        Self {
            tokenizer,
            output_file,
        }
    }

    /*-----------------------------------PROGRAM STRUCTURE-----------------------------------*/

    /// Compiles a complete class.
    ///
    /// class: 'class' className '{' classVarDec* subroutineDec* '}'
    fn compile_class(&mut self) {
        let class_kw = match self.tokenizer.next() {
            Some(t) if t == Token::Keyword("class".into()) => Token::process(t),
            _ => panic!("Expected 'class' Token::Keyword."),
        };

        let Some(Token::Identifier(class_name)) = self.tokenizer.next() else {
            panic!("Expected className Token::Identifier");
        };

        let open_brace = match self.tokenizer.next() {
            Some(t) if t == Token::Symbol('{') => Token::process(t),
            _ => panic!("Expected '{{' Token::Symbol."),
        };

        self.write(format!(
            "<class>\n\t{}",
            class_kw + &class_name + &open_brace
        ));

        self.compile_class_var_dec();

        self.compile_subroutine();

        let close_brace = match self.tokenizer.next() {
            Some(t) if t == Token::Symbol('}') => Token::process(t),
            _ => panic!("Expected '}}' Token::Symbol."),
        };

        self.write(close_brace + "</class>\n");
    }

    /// Compiles a static variable declaration, or a field declaration.
    ///
    /// classVarDec: ('static' | 'field') type varName (',' varName)* ';'
    /// type: 'int' | 'char' | 'boolean' | className
    fn compile_class_var_dec(&mut self) {
        let static_or_field = match self.tokenizer.next() {
            Some(Token::Keyword(kw)) if ["static", "field"].contains(&kw.as_str()) => {
                Token::process(Token::Keyword(kw))
            }
            _ => panic!("Expected ('static' | 'field') Token::Keyword."),
        };

        let var_type = match self.tokenizer.next() {
            Some(Token::Keyword(kw)) if ["int", "char", "boolean"].contains(&kw.as_str()) => {
                Token::process(Token::Keyword(kw))
            }
            Some(Token::Identifier(name)) => Token::process(Token::Identifier(name)),
            _ => panic!("Expected type: 'int' | 'char' | 'boolean' Token::Keyword or className Token::Identifier")
        };

        let Some(Token::Identifier(var_name)) = self.tokenizer.next() else {
            panic!("Expected varName Token::Identifier");
        };

        let mut aux = false;

        let comma_y_var_name_pairs = self
            .tokenizer
            .take_while_ref(|t| {
                let n_aux = match t {
                    Token::Symbol(',') => true,
                    Token::Identifier(_) => false,
                    _ => return false,
                };
                let ret = aux != n_aux;
                aux = n_aux;

                ret
            })
            .map(Token::process)
            .collect::<String>();

        let semicolon = match self.tokenizer.next() {
            Some(t) if t == Token::Symbol(';') => Token::process(t),
            _ => panic!("Expected ';' Token::Symbol."),
        };

        self.write(format!(
            "<classVarDec>\n\t{}</classVarDec>\n",
            static_or_field + &var_type + &var_name + &comma_y_var_name_pairs + &semicolon
        ));
    }

    /// Compiles a complete method, function, or constructor.
    ///
    /// subroutineDec: ('method' | 'function' | 'constructor') ('void' | type) subroutineName '('
    /// parameterList ')' subroutineBody
    /// type: 'int' | 'char' | 'boolean' | varName
    fn compile_subroutine(&mut self) -> String {
        String::new()
    }

    /// Compiles a (possibly empty) parameter list.
    ///
    /// Does not handle the enclosing parentheses tokens ( and ).
    ///
    /// parameterList: ((type varName) (',' type varName)*)?
    /// type: 'int' | 'char' | 'boolean' | className
    fn compile_parameter_list() {}

    /// Compiles a subroutine's body.
    ///
    /// subroutineBody: '{' varDec* statements '}'
    fn compile_subroutine_body() {}

    /// Compiles a var declaration.
    ///
    /// varDec: 'var' type varName
    /// type: 'int' | 'char' | 'boolean' | className
    fn compile_var_dec() {}

    /*-----------------------------------STATEMENTS-----------------------------------*/

    /// Compiles a sequence of statements.
    ///
    /// Does not handle the enclosing curly bracket tokens { and }.
    ///
    /// statements: (letStatement | ifStatement | whileStatement | doStatement | returnStatement)*
    fn compile_statements() {}

    /// Compiles a let statement.
    ///
    /// letStatement: 'let' varName ('\[' expression '\]')? '=' expression ';'
    fn compile_let() {}

    /// Compiles an if statement, possibly with a trailing else clause.
    ///
    /// ifStatement: 'if' '(' expression ')' '{' statements '}'
    fn compile_if() {}

    /// Compiles a while statement.
    ///
    /// whileStatement: 'while' '(' expression ')' '{' statements '}'
    fn compile_while() {}

    /// Compiles a do statement.
    ///
    /// doStatement: 'do' subroutineCall ';'
    /// subroutineCall: subroutineName '(' expressionList ')' | (className | varName) '.'
    /// subroutineName '(' expressionList ')'
    fn compile_do() {}

    /// Compiles a return statement.
    ///
    /// returnStatement: 'return' expression? ';'
    fn compile_return() {}

    /*-----------------------------------EXPRESSIONS-----------------------------------*/

    /// Compiles an expression.
    ///
    /// expression: term (op term)*
    /// op: '+' | '-' | '*' | '/' | '&' | '|' | '<' | '>' | '='
    fn compile_expression() {}

    /// Compiles a term.
    ///
    /// If the current token is an identifier, the routine must resolve it into a variable, an
    /// array entry, or a subroutine call. A single lookahead token, which may be [, {, or .,
    /// suffices to distinguish between the possibilities. Any other token is not part of this term
    /// and should not be advanced over.
    ///
    /// term: integerConstant | stringConstant | keywordConstant | varName | varName '\['
    /// expression '\]' | subroutineCall | '(' expression ')' | unaryOp term
    /// keywordConstant: 'true' | 'false' | 'null' | 'this'
    /// unaryOp: '-' | '~'
    fn compile_term() {}

    /// Compiles a (possibly empty) comma-seperated list of expressions.
    ///
    /// Returns the number of expressions in the list.
    ///
    /// expressionList: (expression (',' expression)*)?
    fn compile_expression_list() -> i32 {
        0
    }

    fn write(&mut self, s: String) {
        self.output_file
            .write_all(s.as_bytes())
            .expect("Should be able to write to output file.");
    }
}
