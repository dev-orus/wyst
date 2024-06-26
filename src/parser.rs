use crate::{
    lexer::{LexerState, Token, TokenType},
    variable::Variables,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstType {
    Ref,
    FunctionDeceleration,
    StructDeceleration,
    StructCall,
    StructVar,
    VoidFunctionDeceleration,
    Namespace,
    VariableDeceleration,
    PointerDeceleration,
    MutVariableDeceleration,
    State3,
    State2,
    Include,
    IncludeLocal,
    CodeBlock,
    Json,
    Impl,
    StaticExecution,
    Other,
}

pub fn is_decl(ast: &Ast) -> bool {
    ast.ast_type == AstType::FunctionDeceleration
        || ast.ast_type == AstType::VoidFunctionDeceleration
        || ast.ast_type == AstType::Namespace
        || ast.ast_type == AstType::VariableDeceleration
        || ast.ast_type == AstType::PointerDeceleration
        || ast.ast_type == AstType::MutVariableDeceleration
        || ast.ast_type == AstType::StructDeceleration
}

pub struct Ast {
    pub tokens: Vec<Token>,
    pub ast_type: AstType,
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1b[36m{:?}:\x1b[0m [\n", self.ast_type)?;
        for (i, token) in self.tokens.iter().enumerate() {
            if i < self.tokens.len() - 1 {
                write!(f, "    {},\n", token)?;
            } else {
                write!(f, "    {}\n", token)?;
            }
        }
        write!(f, "]")
    }
}

impl fmt::Debug for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: [\n", self.ast_type)?;
        for (i, token) in self.tokens.iter().enumerate() {
            if i < self.tokens.len() - 1 {
                write!(f, "    {:?},\n", token)?;
            } else {
                write!(f, "    {:?}\n", token)?;
            }
        }
        write!(f, "]")
    }
}

pub struct Parser {
    pub tokens: Vec<Token>,
    pub index: u32,
    pub include_regex: Lazy<Regex>,
    pub include_regex_local: Lazy<Regex>,
    pub variables: Variables,
    pub json: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, variables: Variables) -> Parser {
        Parser {
            tokens: tokens,
            index: 0,
            include_regex: Lazy::new(|| Regex::new(r"^(#include *)<(.*?)>").unwrap()),
            include_regex_local: Lazy::new(|| Regex::new(r#"^(#include *)"(.*?)""#).unwrap()),
            variables: variables,
            json: false,
        }
    }
    pub fn parse(&mut self) -> Vec<Ast> {
        let mut full_ast: Vec<Ast> = Vec::new();
        while self.tokens.len() > self.index as usize {
            let mut ast_res: Ast = Ast {
                tokens: vec![],
                ast_type: AstType::Other,
            };
            let index = self.index as usize;
            if index == self.tokens.len() {
                panic!("Reached the end of tokens")
            }
            let token = &self.tokens[index];
            if self.json
                && self.tokens.len() - (self.index as usize) > 2
                && self.tokens[index + 1].value == ":"
            {
                ast_res.ast_type = AstType::Json;
                ast_res.tokens.push(self.tokens[index].clone());
                ast_res.tokens.push(self.tokens[index + 2].clone());
                self.index += 2;
            } else if self.tokens.len() - index > 1
                && self.tokens[index].value == "&"
                && self.tokens[index + 1].token_type == TokenType::Identifier
            {
                ast_res.tokens.push(self.tokens[index + 1].clone());
                ast_res.ast_type = AstType::Ref;
                self.index += 1;
            } else if self.tokens.len() - index > 2
                && self.tokens[index].value == "struct"
                && self.tokens[index + 1].token_type == TokenType::Identifier
                && self.tokens[index + 2].token_type == TokenType::Curly
            {
                ast_res.tokens.push(self.tokens[index + 1].clone());
                ast_res.tokens.push(self.tokens[index + 2].clone());
                ast_res.ast_type = AstType::StructDeceleration;
                self.index += 2;
                let mut desc = String::new();
                if index > 0 && self.tokens[index - 1].token_type == TokenType::Comment {
                    desc = self.tokens[index - 1].value.clone()
                }
                self.variables.new_struct(
                    self.tokens[index + 1].clone().value,
                    LexerState {
                        line: self.tokens[index + 1].clone().line,
                        column: self.tokens[index + 1].clone().column,
                    },
                    desc,
                );
            } else if self.tokens.len() - index > 2
                && self.tokens[index].value == "namespace"
                && self.tokens[index + 1].token_type == TokenType::Identifier
                && self.tokens[index + 2].token_type == TokenType::Curly
            {
                ast_res.tokens.push(self.tokens[index + 1].clone());
                ast_res.tokens.push(self.tokens[index + 2].clone());
                ast_res.ast_type = AstType::Namespace;
                self.index += 2;
                let mut desc = String::new();
                if index > 0 && self.tokens[index - 1].token_type == TokenType::Comment {
                    desc = self.tokens[index - 1].value.clone()
                }
                self.variables.new_namespace(
                    self.tokens[index + 1].clone().value,
                    LexerState {
                        line: self.tokens[index + 1].clone().line,
                        column: self.tokens[index + 1].clone().column,
                    },
                    desc,
                );
            } else if self.tokens.len() - index > 2
                && self.tokens[index].value == "impl"
                && self.tokens[index + 1].token_type == TokenType::Identifier
                && self.tokens[index + 2].token_type == TokenType::Curly
            {
                ast_res.tokens.push(self.tokens[index + 1].clone());
                ast_res.tokens.push(self.tokens[index + 2].clone());
                ast_res.ast_type = AstType::Impl;
                self.index += 2;
            } else if self.tokens.len() - index > 2
                && self.tokens[index].token_type == TokenType::Keyword1
                && self.tokens[index + 1].token_type == TokenType::Round
                && self.tokens[index + 2].token_type == TokenType::Curly
            {
                ast_res.tokens.push(self.tokens[index].clone());
                ast_res.tokens.push(self.tokens[index + 1].clone());
                ast_res.tokens.push(self.tokens[index + 2].clone());
                ast_res.ast_type = AstType::State3;
                self.index += 2;
            } else if self.tokens.len() - index > 1
                && self.tokens[index].token_type == TokenType::Keyword2
                && self.tokens[index + 1].token_type == TokenType::Curly
            {
                ast_res.tokens.push(self.tokens[index].clone());
                ast_res.tokens.push(self.tokens[index + 1].clone());
                ast_res.ast_type = AstType::State2;
                self.index += 1;
            } else {
                match token.token_type {
                    TokenType::Identifier => {
                        ast_res.tokens.push(self.tokens[index].clone());
                        if self.tokens.len() - index > 3
                            && self.tokens[index + 1].token_type == TokenType::Identifier
                            && self.tokens[index + 2].token_type == TokenType::Round
                            && self.tokens[index + 3].token_type == TokenType::Curly
                        {
                            ast_res.tokens.push(self.tokens[index + 1].clone());
                            ast_res.tokens.push(self.tokens[index + 2].clone());
                            ast_res.tokens.push(self.tokens[index + 3].clone());
                            if token.value == "void" {
                                ast_res.ast_type = AstType::VoidFunctionDeceleration;
                            } else {
                                ast_res.ast_type = AstType::FunctionDeceleration;
                            }
                            self.index += 3;
                            let mut desc = String::new();
                            if index > 0 && self.tokens[index - 1].token_type == TokenType::Comment
                            {
                                desc = self.tokens[index - 1].value.clone()
                            }
                            self.variables.new_func(
                                self.tokens[index + 1].clone().value,
                                LexerState {
                                    line: self.tokens[index + 1].clone().line,
                                    column: self.tokens[index + 1].clone().column,
                                },
                                desc,
                            );
                        } else if self.tokens.len() - index > 1
                            && self.tokens[index + 1].token_type == TokenType::Curly
                        {
                            ast_res.tokens.push(self.tokens[index + 1].clone());
                            ast_res.ast_type = AstType::StructCall;
                            self.index += 1;
                        } else if self.tokens.len() - index > 2
                            && self.tokens[index + 1].token_type == TokenType::Identifier
                            && self.tokens[index + 2].token_type == TokenType::Curly
                        {
                            ast_res.tokens.push(self.tokens[index + 1].clone());
                            ast_res.tokens.push(self.tokens[index + 2].clone());
                            ast_res.ast_type = AstType::StructVar;
                            self.index += 2;
                            let mut desc = String::new();
                            if index > 0 && self.tokens[index - 1].token_type == TokenType::Comment
                            {
                                desc = self.tokens[index - 1].value.clone()
                            }
                            self.variables.new_var(
                                self.tokens[index + 1].clone().value,
                                LexerState {
                                    line: self.tokens[index + 1].clone().line,
                                    column: self.tokens[index + 1].clone().column,
                                },
                                desc,
                            );
                        } else if self.tokens.len() - index > 1 {
                            if self.tokens[index + 1].token_type == TokenType::Identifier {
                                ast_res.tokens.push(self.tokens[index + 1].clone());
                                ast_res.ast_type = AstType::VariableDeceleration;
                                self.index += 1;
                                let mut desc = String::new();
                                if index > 0
                                    && self.tokens[index - 1].token_type == TokenType::Comment
                                {
                                    desc = self.tokens[index - 1].value.clone()
                                }
                                self.variables.new_var(
                                    self.tokens[index + 1].clone().value,
                                    LexerState {
                                        line: self.tokens[index + 1].clone().line,
                                        column: self.tokens[index + 1].clone().column,
                                    },
                                    desc,
                                );
                            } else if self.tokens.len() - index > 2
                                && self.tokens[index + 2].token_type == TokenType::Identifier
                                && self.tokens[index + 1].token_type == TokenType::Angle
                            {
                                ast_res.tokens.push(self.tokens[index + 2].clone());
                                ast_res.tokens[0].value += "<";
                                ast_res.tokens[0].value += self.tokens[index + 1].value.as_str();
                                ast_res.tokens[0].value += ">";
                                ast_res.ast_type = AstType::VariableDeceleration;
                                self.index += 2;
                                let mut desc = String::new();
                                if index > 0
                                    && self.tokens[index - 1].token_type == TokenType::Comment
                                {
                                    desc = self.tokens[index - 1].value.clone()
                                }
                                self.variables.new_var(
                                    self.tokens[index + 1].clone().value,
                                    LexerState {
                                        line: self.tokens[index + 1].clone().line,
                                        column: self.tokens[index + 1].clone().column,
                                    },
                                    desc,
                                );
                            } else if self.tokens.len() - index > 2
                                && self.tokens[index + 1].value == "*"
                                && self.tokens[index + 2].token_type == TokenType::Identifier
                            {
                                ast_res.tokens.push(self.tokens[index + 2].clone());
                                ast_res.ast_type = AstType::PointerDeceleration;
                                self.index += 2;
                                let mut desc = String::new();
                                if index > 0
                                    && self.tokens[index - 1].token_type == TokenType::Comment
                                {
                                    desc = self.tokens[index - 1].value.clone()
                                }
                                self.variables.new_var(
                                    self.tokens[index + 1].clone().value,
                                    LexerState {
                                        line: self.tokens[index + 1].clone().line,
                                        column: self.tokens[index + 1].clone().column,
                                    },
                                    desc,
                                );
                            }
                        }
                    }
                    TokenType::Include => {
                        if let Some(caps) = self.include_regex.captures(&token.value) {
                            ast_res.tokens.push(Token {
                                token_type: TokenType::String,
                                value: caps[2].to_owned().to_string(),
                                line: 0,
                                column: 0,
                            });
                            ast_res.ast_type = AstType::Include;
                        } else if let Some(caps) = self.include_regex_local.captures(&token.value) {
                            ast_res.tokens.push(Token {
                                token_type: TokenType::String,
                                value: caps[2].to_owned().to_string(),
                                line: 0,
                                column: 0,
                            });
                            ast_res.ast_type = AstType::IncludeLocal;
                        } else {
                            ast_res.tokens.push(token.clone());
                            ast_res.ast_type = AstType::Include;
                        }
                    }
                    TokenType::Keyword => {
                        if token.value == "cb"
                            && self.tokens[index + 1].token_type == TokenType::Curly
                        {
                            ast_res.tokens.push(self.tokens[index + 1].clone());
                            ast_res.ast_type = AstType::CodeBlock;
                            self.index += 1;
                        } else {
                            ast_res.tokens.push(token.clone());
                        }
                    }
                    TokenType::StaticExecution => {
                        if self.tokens[index + 1].token_type == TokenType::Square {
                            ast_res.tokens.push(self.tokens[index + 1].clone());
                            ast_res.ast_type = AstType::StaticExecution;
                        }
                    }
                    _ => {
                        ast_res.tokens.push(token.clone());
                    }
                }
            }
            self.index += 1;
            full_ast.push(ast_res);
        }
        full_ast
    }
}
