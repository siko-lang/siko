use super::expr::parse_ops;
use super::expr::parse_pattern;
use super::util::parse_parens;
use super::util::report_parser_error;
use super::util::report_unexpected_token;
use super::util::ParenParseResult;
use super::util::ParserErrorReason;
use crate::error::ParseError;
use crate::token::Token;
use crate::token::TokenInfo;
use crate::token::TokenKind;
use siko_constants::get_implicit_module_list;
use siko_constants::get_qualified_list_type_name;
use siko_constants::BuiltinOperator;
use siko_location_info::filepath::FilePath;
use siko_location_info::item::Item;
use siko_location_info::item::ItemInfo;
use siko_location_info::location::Location;
use siko_location_info::location_id::LocationId;
use siko_location_info::location_info::LocationInfo;
use siko_location_info::location_set::LocationSet;
use siko_location_info::span::Span;
use siko_syntax::actor::Actor;
use siko_syntax::actor::Protocol;
use siko_syntax::actor::ProtocolHandler;
use siko_syntax::class::Class;
use siko_syntax::class::Constraint;
use siko_syntax::class::Instance;
use siko_syntax::data::Adt;
use siko_syntax::data::Data;
use siko_syntax::data::DerivedClass;
use siko_syntax::data::Record;
use siko_syntax::data::RecordField;
use siko_syntax::data::Variant;
use siko_syntax::data::VariantId;
use siko_syntax::export_import::EIGroup;
use siko_syntax::export_import::EIItem;
use siko_syntax::export_import::EIItemInfo;
use siko_syntax::export_import::EIList;
use siko_syntax::export_import::EIMember;
use siko_syntax::export_import::EIMemberInfo;
use siko_syntax::expr::Expr;
use siko_syntax::expr::ExprId;
use siko_syntax::function::Function;
use siko_syntax::function::FunctionBody;
use siko_syntax::function::FunctionId;
use siko_syntax::function::FunctionType;
use siko_syntax::function::FunctionTypeId;
use siko_syntax::import::HiddenItem;
use siko_syntax::import::Import;
use siko_syntax::import::ImportId;
use siko_syntax::import::ImportKind;
use siko_syntax::module::Module;
use siko_syntax::module::ModuleId;
use siko_syntax::pattern::Pattern;
use siko_syntax::pattern::PatternId;
use siko_syntax::program::Program;
use siko_syntax::types::TypeSignature;
use siko_syntax::types::TypeSignatureId;
use siko_util::Counter;
use std::collections::BTreeMap;

enum FunctionOrFunctionType {
    Function(String, FunctionId),
    FunctionType(String, FunctionTypeId),
}

fn parse_class_constraint(parser: &mut Parser) -> Result<Constraint, ParseError> {
    let start_index = parser.get_index();
    let name = parser.parse_qualified_type_name()?;
    let arg = parser.var_identifier("type arg")?;
    let end_index = parser.get_index();
    let location_id = parser.get_location_id(start_index, end_index);
    let constraint = Constraint {
        class_name: name,
        arg: arg,
        location_id: location_id,
    };
    Ok(constraint)
}

pub struct Parser<'a> {
    file_path: FilePath,
    tokens: &'a [TokenInfo],
    index: usize,
    program: &'a mut Program,
    location_info: &'a mut LocationInfo,
    temp_var_counter: Counter,
}

impl<'a> Parser<'a> {
    pub fn new(
        file_path: FilePath,
        tokens: &'a [TokenInfo],
        program: &'a mut Program,
        location_info: &'a mut LocationInfo,
    ) -> Parser<'a> {
        Parser {
            file_path: file_path,
            tokens: tokens,
            index: 0,
            program: program,
            location_info: location_info,
            temp_var_counter: Counter::new(),
        }
    }

    pub fn get_file_path(&self) -> FilePath {
        self.file_path.clone()
    }

    pub fn get_program(&mut self) -> &mut Program {
        &mut self.program
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_location_id(&mut self, start: usize, end: usize) -> LocationId {
        let mut set = LocationSet::new(self.file_path.clone());
        for token in &self.tokens[start..end] {
            set.add(token.location.clone());
        }
        let li_item = Item::new(set);
        let location_id = self.location_info.add_item(li_item);
        location_id
    }

    pub fn is_done(&self) -> bool {
        self.index >= self.tokens.len()
    }

    pub fn get_last(&self) -> TokenInfo {
        self.tokens[self.tokens.len() - 1].clone()
    }

    pub fn advance(&mut self) -> Result<TokenInfo, ParseError> {
        if self.is_done() {
            let last = &self.tokens[self.tokens.len() - 1];
            return Err(ParseError::new(
                format!("Unexpected end of stream"),
                self.file_path.clone(),
                last.location.clone(),
            ));
        }
        let r = self.tokens[self.index].clone();
        self.index += 1;
        Ok(r)
    }

    fn restore(&mut self, index: usize) {
        self.index = index;
    }

    pub fn peek(&self) -> Option<TokenInfo> {
        if self.is_done() {
            None
        } else {
            let r = self.tokens[self.index].clone();
            Some(r)
        }
    }

    pub fn get_temp_var_name(&mut self) -> String {
        format!("${}", self.temp_var_counter.next())
    }

    pub fn irrefutable_pattern_follows(&self) -> bool {
        let mut index = self.index;
        while index < self.tokens.len() {
            if self.tokens[index].token.kind() == TokenKind::EndOfItem {
                return false;
            }
            if self.tokens[index].token.kind() == TokenKind::KeywordDo {
                return false;
            }
            if self.tokens[index].token.kind() == TokenKind::Op(BuiltinOperator::Bind) {
                return true;
            }
            index += 1;
        }
        false
    }

    fn constraint_follows(&self) -> bool {
        let mut index = self.index;
        while index < self.tokens.len() {
            if self.tokens[index].token.kind() == TokenKind::EndOfItem {
                return false;
            }
            if self.tokens[index].token.kind() == TokenKind::KeywordConstraint {
                return true;
            }
            index += 1;
        }
        false
    }

    pub fn type_identifier(&mut self, item: &str) -> Result<String, ParseError> {
        let token_info = self.peek().expect("Ran out of tokens");
        if let Token::TypeIdentifier(i) = token_info.token {
            self.advance()?;
            return Ok(i);
        } else {
            return report_unexpected_token(self, item.to_string());
        }
    }

    pub fn var_identifier(&mut self, item: &str) -> Result<String, ParseError> {
        let token_info = self.peek().expect("Ran out of tokens");
        if let Token::VarIdentifier(i) = token_info.token {
            self.advance()?;
            return Ok(i);
        } else {
            return report_unexpected_token(self, item.to_string());
        }
    }

    pub fn current(&self, token: TokenKind) -> bool {
        if self.is_done() {
            return false;
        }
        self.tokens[self.index].token.kind() == token
    }

    pub fn current_kind(&self) -> TokenKind {
        self.tokens[self.index].token.kind()
    }

    pub fn expect(&mut self, token: TokenKind) -> Result<TokenInfo, ParseError> {
        let t = self.current_kind();
        if t == token {
            let t = self.advance()?;
            return Ok(t);
        } else {
            if token == TokenKind::EndOfItem {
                let reason = ParserErrorReason::Custom {
                    msg: format!("unexpected {}", t.nice_name()),
                };
                return report_parser_error(self, reason);
            }
            return report_unexpected_token(self, token.nice_name());
        }
    }

    pub fn parse_pattern_args(&mut self) -> Result<Vec<PatternId>, ParseError> {
        let mut args = Vec::new();
        loop {
            match self.current_kind() {
                TokenKind::LParen
                | TokenKind::IntegerLiteral
                | TokenKind::FloatLiteral
                | TokenKind::StringLiteral
                | TokenKind::VarIdentifier
                | TokenKind::TypeIdentifier
                | TokenKind::Wildcard => {
                    let arg = parse_pattern(self)?;
                    args.push(arg);
                }
                _ => {
                    break;
                }
            }
        }
        Ok(args)
    }

    pub fn parse_lambda_args(&mut self) -> Result<Vec<PatternId>, ParseError> {
        let mut args = Vec::new();
        loop {
            let arg = parse_pattern(self)?;
            args.push(arg);
            if !self.current(TokenKind::Comma) {
                break;
            } else {
                self.expect(TokenKind::Comma)?;
                continue;
            }
        }
        Ok(args)
    }

    pub fn parse_list1_in_parens<T>(
        &mut self,
        parse_fn: fn(&mut Parser) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        self.expect(TokenKind::LParen)?;
        let mut items = Vec::new();
        loop {
            let item = parse_fn(self)?;
            items.push(item);
            let comma = if self.current(TokenKind::Comma) {
                self.expect(TokenKind::Comma)?;
                true
            } else {
                false
            };
            if self.current(TokenKind::RParen) {
                break;
            } else {
                if !comma {
                    return report_unexpected_token(self, format!("comma"));
                }
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(items)
    }

    fn parse_list0_in_parens<T>(
        &mut self,
        parse_fn: fn(&mut Parser) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        self.expect(TokenKind::LParen)?;
        let mut items = Vec::new();
        loop {
            if self.current(TokenKind::RParen) {
                break;
            }
            let item = parse_fn(self)?;
            items.push(item);
            if self.current(TokenKind::Comma) {
                self.expect(TokenKind::Comma)?;
            } else {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(items)
    }

    pub fn parse_list0_in_curly_parens<T>(
        &mut self,
        parse_fn: fn(&mut Parser) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        self.expect(TokenKind::LCurly)?;
        let mut items = Vec::new();
        loop {
            if self.current(TokenKind::RCurly) {
                break;
            }
            let item = parse_fn(self)?;
            items.push(item);
            if self.current(TokenKind::Comma) {
                self.expect(TokenKind::Comma)?;
            } else {
                break;
            }
        }
        self.expect(TokenKind::RCurly)?;
        Ok(items)
    }

    fn parse_args(&mut self) -> Result<Vec<(String, LocationId)>, ParseError> {
        let mut items = Vec::new();
        while !self.is_done() {
            if !self.current(TokenKind::VarIdentifier) {
                break;
            }
            let start_index = self.get_index();
            let item = self.var_identifier("argument")?;
            let end_index = self.get_index();
            let location_id = self.get_location_id(start_index, end_index);
            items.push((item, location_id));
        }
        Ok(items)
    }

    pub fn consume_op(
        &mut self,
        op_kinds: &[BuiltinOperator],
    ) -> Option<(BuiltinOperator, TokenInfo)> {
        if let Some(op_token) = self.peek() {
            if let Some(op) = op_token.token.get_op() {
                if op_kinds.contains(&op) {
                    self.index += 1;
                    return Some((op, op_token));
                }
            }
        }
        None
    }

    pub fn parse_expr(&mut self) -> Result<ExprId, ParseError> {
        let id = parse_ops(self)?;
        Ok(id)
    }

    pub fn add_expr(&mut self, expr: Expr, start_index: usize) -> ExprId {
        let end_index = self.get_index();
        let location_id = self.get_location_id(start_index, end_index);
        let id = self.program.exprs.get_id();
        let info = ItemInfo::new(expr, location_id);
        self.program.exprs.add_item(id, info);
        id
    }

    pub fn add_type_signature(
        &mut self,
        type_signature: TypeSignature,
        start_index: usize,
    ) -> TypeSignatureId {
        let end_index = self.get_index();
        let location_id = self.get_location_id(start_index, end_index);
        let id = self.program.type_signatures.get_id();
        let info = ItemInfo::new(type_signature, location_id);
        self.program.type_signatures.add_item(id, info);
        id
    }

    pub fn add_pattern(&mut self, pattern: Pattern, start_index: usize) -> PatternId {
        let end_index = self.get_index();
        let location_id = self.get_location_id(start_index, end_index);
        let id = self.program.patterns.get_id();
        let info = ItemInfo::new(pattern, location_id);
        self.program.patterns.add_item(id, info);
        id
    }

    fn parse_tuple_type(&mut self, allow_wildcard: bool) -> Result<TypeSignatureId, ParseError> {
        let start_index = self.get_index();
        let res = if allow_wildcard {
            parse_parens(self, |p| p.parse_function_type(false, true), "<type>")?
        } else {
            parse_parens(self, |p| p.parse_function_type(false, false), "<type>")?
        };
        match res {
            ParenParseResult::Single(t) => {
                return Ok(t);
            }
            ParenParseResult::Tuple(ts) => {
                let type_signature = TypeSignature::Tuple(ts);
                let id = self.add_type_signature(type_signature, start_index);
                return Ok(id);
            }
        }
    }

    pub fn parse_function_type(
        &mut self,
        parsing_variant: bool,
        allow_wildcard: bool,
    ) -> Result<TypeSignatureId, ParseError> {
        let start_index = self.get_index();
        let mut from = self.parse_type_part(parsing_variant, allow_wildcard)?;
        if let Some(next) = self.peek() {
            match next.token {
                Token::Op(BuiltinOperator::Arrow) => {
                    self.advance()?;
                    let to = self.parse_function_type(parsing_variant, allow_wildcard)?;
                    let ty = TypeSignature::Function(from, to);
                    let ty = self.add_type_signature(ty, start_index);
                    from = ty;
                }
                _ => {}
            }
        }
        Ok(from)
    }

    fn parse_type_part(
        &mut self,
        parsing_variant: bool,
        allow_wildcard: bool,
    ) -> Result<TypeSignatureId, ParseError> {
        let start_index = self.get_index();
        match self.peek() {
            Some(token_info) => match token_info.token {
                Token::LParen => {
                    return self.parse_tuple_type(allow_wildcard);
                }
                Token::LBracket => {
                    self.expect(TokenKind::LBracket)?;
                    let arg = self.parse_function_type(false, allow_wildcard)?;
                    self.expect(TokenKind::RBracket)?;
                    let ty = TypeSignature::Named(get_qualified_list_type_name(), vec![arg]);
                    let id = self.add_type_signature(ty, start_index);
                    return Ok(id);
                }
                Token::TypeIdentifier(_) => {
                    let name = self.parse_qualified_type_name()?;
                    let mut args = Vec::new();
                    loop {
                        match self.current_kind() {
                            TokenKind::TypeIdentifier => {
                                let arg_start_index = self.get_index();
                                let arg = self.parse_qualified_type_name()?;
                                let arg = self.add_type_signature(
                                    TypeSignature::Named(arg, Vec::new()),
                                    arg_start_index,
                                );
                                args.push(arg);
                            }
                            TokenKind::VarIdentifier => {
                                let arg_start_index = self.get_index();
                                let arg = self.var_identifier("type arg")?;
                                let arg = self.add_type_signature(
                                    TypeSignature::TypeArg(arg),
                                    arg_start_index,
                                );
                                args.push(arg);
                            }
                            TokenKind::LParen => {
                                let arg = self.parse_tuple_type(allow_wildcard)?;
                                args.push(arg);
                            }
                            TokenKind::LBracket => {
                                self.expect(TokenKind::LBracket)?;
                                let arg = self.parse_function_type(false, allow_wildcard)?;
                                self.expect(TokenKind::RBracket)?;
                                let ty =
                                    TypeSignature::Named(get_qualified_list_type_name(), vec![arg]);
                                let id = self.add_type_signature(ty, start_index);
                                args.push(id);
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    let ty = if parsing_variant {
                        TypeSignature::Variant(name, args)
                    } else {
                        TypeSignature::Named(name, args)
                    };
                    let id = self.add_type_signature(ty, start_index);
                    return Ok(id);
                }
                Token::VarIdentifier(_) => {
                    let name = self.var_identifier("type arg")?;
                    let ty = TypeSignature::TypeArg(name);
                    let id = self.add_type_signature(ty, start_index);
                    return Ok(id);
                }
                Token::Ampersand => {
                    self.expect(TokenKind::Ampersand)?;
                    let arg = self.parse_function_type(false, allow_wildcard)?;
                    let ty = TypeSignature::Ref(arg);
                    let id = self.add_type_signature(ty, start_index);
                    return Ok(id);
                }
                Token::Wildcard => {
                    if allow_wildcard {
                        self.expect(TokenKind::Wildcard)?;
                        let ty = TypeSignature::Wildcard;
                        let ty = self.add_type_signature(ty, start_index);
                        return Ok(ty);
                    } else {
                        let reason = ParserErrorReason::Custom {
                            msg: format!("wildcard is not allowed in this context"),
                        };
                        return report_parser_error(self, reason);
                    }
                }
                Token::Op(BuiltinOperator::Not) => {
                    self.expect(TokenKind::Op(BuiltinOperator::Not))?;
                    let ty = TypeSignature::Never;
                    let ty = self.add_type_signature(ty, start_index);
                    return Ok(ty);
                }
                _ => {
                    return report_unexpected_token(self, format!("type signature"));
                }
            },
            None => {
                return report_unexpected_token(self, format!("type signature"));
            }
        }
    }

    fn parse_function_or_function_type(&mut self) -> Result<FunctionOrFunctionType, ParseError> {
        let start_index = self.get_index();
        let name = self.var_identifier("function name")?;
        let name = name;
        let args = self.parse_pattern_args()?;
        if self.current(TokenKind::KeywordDoubleColon) {
            self.advance()?;
            let constraints = if self.constraint_follows() {
                let cs = self.parse_list1_in_parens(parse_class_constraint)?;
                self.expect(TokenKind::KeywordConstraint)?;
                cs
            } else {
                Vec::new()
            };
            let type_signature_id = self.parse_function_type(false, false)?;
            let full_type_signature_id = self.program.type_signatures.get_id();
            let end_index = self.get_index();
            let location_id = self.get_location_id(start_index, end_index);
            let id = self.program.function_types.get_id();
            let mut simple_args = Vec::new();
            for arg in args {
                let item_info = &self.program.patterns.get(&arg);
                let pattern = &item_info.item;
                let location_id = item_info.location_id;
                if let Pattern::Binding(n) = pattern {
                    simple_args.push((n.clone(), location_id))
                } else {
                    let location_set = self.location_info.get_item_location(&location_id);
                    let (line, ranges) = location_set.lines.iter().next().unwrap();
                    let location = Location {
                        line: *line,
                        span: Span {
                            start: ranges[0].start,
                            end: ranges[0].end,
                        },
                    };
                    let err = ParseError::new(
                        "Invalid type argument".to_string(),
                        self.file_path.clone(),
                        location,
                    );
                    return Err(err);
                }
            }
            let function_type = FunctionType {
                id: id,
                name: name.clone(),
                type_args: simple_args,
                constraints: constraints,
                full_type_signature_id: full_type_signature_id,
                type_signature_id: type_signature_id,
                location_id: location_id,
            };
            self.expect(TokenKind::EndOfItem)?;
            self.program.function_types.add_item(id, function_type);
            Ok(FunctionOrFunctionType::FunctionType(name, id))
        } else {
            let end_index = self.get_index();
            let location_id = self.get_location_id(start_index, end_index);
            self.expect(TokenKind::Equal)?;
            let mut temp_args = Vec::new();
            let body = if let Some(token) = self.peek() {
                if token.token.kind() == TokenKind::KeywordExtern {
                    self.expect(TokenKind::KeywordExtern)?;
                    for arg in args {
                        let item_info = &self.program.patterns.get(&arg);
                        let pattern = &item_info.item;
                        let location_id = item_info.location_id;
                        if let Pattern::Binding(n) = pattern {
                            temp_args.push((n.clone(), location_id))
                        } else {
                            let location_set = self.location_info.get_item_location(&location_id);
                            let (line, ranges) = location_set.lines.iter().next().unwrap();
                            let location = Location {
                                line: *line,
                                span: Span {
                                    start: ranges[0].start,
                                    end: ranges[0].end,
                                },
                            };
                            let err = ParseError::new(
                                "Cannot use patterns in extern function arguments".to_string(),
                                self.file_path.clone(),
                                location,
                            );
                            return Err(err);
                        }
                    }
                    FunctionBody::Extern
                } else {
                    let body_expr_id = self.parse_expr()?;
                    if args.is_empty() {
                        FunctionBody::Expr(body_expr_id)
                    } else {
                        let mut temp_arg_exprs = Vec::new();
                        for arg in args.iter() {
                            let location = self.get_program().patterns.get(arg).location_id;
                            let temp_arg_name = self.get_temp_var_name();
                            temp_args.push((temp_arg_name.clone(), location));
                            let path_expr = Expr::Path(temp_arg_name);
                            let path_expr_id = self.add_expr(path_expr, start_index);
                            temp_arg_exprs.push(path_expr_id);
                        }
                        let tuple_expr = Expr::Tuple(temp_arg_exprs);
                        let tuple_expr_id = self.add_expr(tuple_expr, start_index);
                        let tuple_pattern = Pattern::Tuple(args.clone());
                        let tuple_pattern_id = self.add_pattern(tuple_pattern, start_index);
                        let bind_expr = Expr::Bind(tuple_pattern_id, tuple_expr_id);
                        let bind_expr_id = self.add_expr(bind_expr, start_index);
                        let do_expr = Expr::Do(vec![bind_expr_id, body_expr_id]);
                        let do_expr_id = self.add_expr(do_expr, start_index);
                        FunctionBody::Expr(do_expr_id)
                    }
                }
            } else {
                unreachable!()
            };
            self.expect(TokenKind::EndOfItem)?;
            let id = self.program.functions.get_id();
            let function = Function {
                id: id,
                name: name.clone(),
                args: temp_args,
                body: body,
                location_id: location_id,
            };
            self.program.functions.add_item(id, function);
            Ok(FunctionOrFunctionType::Function(name, id))
        }
    }

    fn parse_export_import_data_member(parser: &mut Parser) -> Result<EIMemberInfo, ParseError> {
        let start_index = parser.get_index();
        let member = if parser.current(TokenKind::DoubleDot) {
            parser.expect(TokenKind::DoubleDot)?;
            EIMember::All
        } else {
            let name = parser.type_identifier("variant")?;
            EIMember::Specific(name)
        };
        let end_index = parser.get_index();
        let location_id = parser.get_location_id(start_index, end_index);
        let info = EIMemberInfo {
            member: member,
            location_id: location_id,
        };
        Ok(info)
    }

    fn parse_export_import_item(parser: &mut Parser) -> Result<EIItemInfo, ParseError> {
        let start_index = parser.get_index();
        let name = parser.any_identifier("item")?;
        let item = if parser.current(TokenKind::LParen) {
            let members = parser.parse_list0_in_parens(Parser::parse_export_import_data_member)?;
            let group = EIGroup {
                name: name,
                members: members,
            };
            EIItem::Group(group)
        } else {
            EIItem::Named(name)
        };
        let end_index = parser.get_index();
        let location_id = parser.get_location_id(start_index, end_index);
        let info = EIItemInfo {
            item: item,
            location_id: location_id,
        };
        Ok(info)
    }

    fn parse_hidden_item(parser: &mut Parser) -> Result<HiddenItem, ParseError> {
        let start_index = parser.get_index();
        let name = parser.any_identifier("hidden item")?;
        let end_index = parser.get_index();
        let location_id = parser.get_location_id(start_index, end_index);
        Ok(HiddenItem {
            name: name,
            location_id: location_id,
        })
    }

    fn parse_import(&mut self, id: ImportId) -> Result<Import, ParseError> {
        let start_index = self.get_index();
        self.expect(TokenKind::KeywordImport)?;
        let name = self.parse_module_name()?;
        let import_kind = if self.current(TokenKind::KeywordHiding) {
            self.expect(TokenKind::KeywordHiding)?;
            let items = self.parse_list1_in_parens(Parser::parse_hidden_item)?;
            ImportKind::Hiding(items)
        } else {
            let import_list = self.parse_export_import_list()?;
            let mut alternative_name = None;
            if let Some(as_token) = self.peek() {
                if let Token::KeywordAs = as_token.token {
                    self.advance()?;
                    let name = self.type_identifier("alternative name")?;
                    alternative_name = Some(name);
                }
            }
            ImportKind::ImportList {
                items: import_list,
                alternative_name: alternative_name,
            }
        };
        let end_index = self.get_index();
        let location_id = self.get_location_id(start_index, end_index);
        self.expect(TokenKind::EndOfItem)?;
        let import = Import {
            id: id.clone(),
            module_path: name,
            kind: import_kind,
            location_id: Some(location_id),
            implicit: false,
        };
        Ok(import)
    }

    fn parse_record_field(&mut self) -> Result<RecordField, ParseError> {
        let start_index = self.get_index();
        let name = self.var_identifier("record field name")?;
        self.expect(TokenKind::KeywordDoubleColon)?;
        let type_signature_id = self.parse_function_type(false, false)?;
        let end_index = self.get_index();
        let location_id = self.get_location_id(start_index, end_index);
        let item = RecordField {
            name: name,
            id: self.program.record_fields.get_id(),
            type_signature_id: type_signature_id,
            location_id: location_id,
        };
        Ok(item)
    }

    fn parse_record(
        &mut self,
        name: String,
        type_args: Vec<(String, LocationId)>,
        start_index: usize,
    ) -> Result<Record, ParseError> {
        let mut fields = Vec::new();
        loop {
            if self.current(TokenKind::RCurly) {
                self.expect(TokenKind::RCurly)?;
                break;
            }
            let record_field = self.parse_record_field()?;
            let id = record_field.id;
            self.program.record_fields.add_item(id, record_field);
            fields.push(id);
            let mut found = false;
            if self.current(TokenKind::Comma) {
                found = true;
                self.expect(TokenKind::Comma)?;
            }
            if self.current(TokenKind::RCurly) {
                self.expect(TokenKind::RCurly)?;
                break;
            }
            if !found {
                return report_unexpected_token(self, format!("comma or }}"));
            }
        }
        let end_index = self.get_index();
        let location_id = self.get_location_id(start_index, end_index);
        let derived_classes = self.parse_deriving()?;
        let record = Record {
            name: name,
            id: self.program.records.get_id(),
            type_args: type_args,
            fields: fields,
            location_id: location_id,
            external: false,
            derived_classes: derived_classes,
        };
        Ok(record)
    }

    fn parse_variant(&mut self) -> Result<VariantId, ParseError> {
        let variant_start_index = self.get_index();
        let name = self.type_identifier("variant")?;
        self.restore(variant_start_index);
        let type_signature_id = self.parse_type_part(true, false)?;
        let end_index = self.get_index();
        let location_id = self.get_location_id(variant_start_index, end_index);
        let id = self.program.variants.get_id();
        let variant = Variant {
            id: id,
            name: name,
            type_signature_id: type_signature_id,
            location_id: location_id,
        };
        self.program.variants.add_item(id, variant);
        Ok(id)
    }

    fn parse_deriving(&mut self) -> Result<Vec<DerivedClass>, ParseError> {
        if self.current(TokenKind::KeywordDeriving) {
            self.expect(TokenKind::KeywordDeriving)?;
            let paren_needed = if self.current(TokenKind::LParen) {
                self.expect(TokenKind::LParen)?;
                true
            } else {
                false
            };
            let mut derived_classes = Vec::new();
            loop {
                if paren_needed {
                    if self.current(TokenKind::RParen) {
                        break;
                    }
                }
                let start_index = self.get_index();
                let name = self.parse_qualified_type_name()?;
                let end_index = self.get_index();
                let location_id = self.get_location_id(start_index, end_index);
                let derived_class = DerivedClass {
                    name: name,
                    location_id: location_id,
                };
                derived_classes.push(derived_class);
                if paren_needed {
                    if self.current(TokenKind::RParen) {
                        break;
                    }
                }
                if self.current(TokenKind::EndOfItem) {
                    break;
                } else {
                    self.expect(TokenKind::Comma)?;
                }
            }
            if paren_needed {
                self.expect(TokenKind::RParen)?;
            }
            return Ok(derived_classes);
        } else {
            return Ok(Vec::new());
        }
    }

    fn parse_data(&mut self) -> Result<Data, ParseError> {
        let start_index = self.get_index();
        self.expect(TokenKind::KeywordData)?;
        let name = self.type_identifier("type")?;
        let args = self.parse_args()?;
        if self.current(TokenKind::Equal) {
            self.expect(TokenKind::Equal)?;
            if self.current(TokenKind::LCurly) {
                self.expect(TokenKind::LCurly)?;
                let record = self.parse_record(name, args, start_index)?;
                Ok(Data::Record(record))
            } else if self.current(TokenKind::KeywordExtern) {
                self.expect(TokenKind::KeywordExtern)?;
                let end_index = self.get_index();
                let location_id = self.get_location_id(start_index, end_index);
                let record = Record {
                    name: name,
                    id: self.program.records.get_id(),
                    type_args: args,
                    fields: Vec::new(),
                    location_id: location_id,
                    external: true,
                    derived_classes: Vec::new(),
                };
                Ok(Data::Record(record))
            } else {
                let mut variants = Vec::new();
                loop {
                    let variant = self.parse_variant()?;
                    variants.push(variant);
                    if self.current(TokenKind::Pipe) {
                        self.expect(TokenKind::Pipe)?;
                    } else {
                        break;
                    }
                }
                let end_index = self.get_index();
                let location_id = self.get_location_id(start_index, end_index);
                let derived_classes = self.parse_deriving()?;
                let adt = Adt {
                    name: name,
                    id: self.program.adts.get_id(),
                    type_args: args,
                    variants: variants,
                    location_id: location_id,
                    derived_classes: derived_classes,
                };
                Ok(Data::Adt(adt))
            }
        } else {
            let end_index = self.get_index();
            let location_id = self.get_location_id(start_index, end_index);
            let derived_classes = self.parse_deriving()?;
            let adt = Adt {
                name: name,
                id: self.program.adts.get_id(),
                type_args: args,
                variants: Vec::new(),
                location_id: location_id,
                derived_classes: derived_classes,
            };
            Ok(Data::Adt(adt))
        }
    }

    fn parse_export_import_list(&mut self) -> Result<EIList, ParseError> {
        let export_list = if self.current(TokenKind::LParen) {
            let items = self.parse_list0_in_parens(Parser::parse_export_import_item)?;
            EIList::Explicit(items)
        } else {
            EIList::ImplicitAll
        };
        Ok(export_list)
    }

    fn parse_module_name(&mut self) -> Result<String, ParseError> {
        let mut name = String::new();
        loop {
            let n = self.type_identifier("module name")?;
            name += &n;
            if self.current(TokenKind::Dot) {
                name.push('.');
                self.expect(TokenKind::Dot)?;
            } else {
                break;
            }
        }
        Ok(name)
    }

    pub fn parse_qualified_type_name(&mut self) -> Result<String, ParseError> {
        let mut name = String::new();
        loop {
            let n = self.type_identifier("type name")?;
            name += &n;
            if self.current(TokenKind::Dot) {
                name.push('.');
                self.expect(TokenKind::Dot)?;
            } else {
                break;
            }
        }
        Ok(name)
    }

    fn any_identifier(&mut self, name: &str) -> Result<String, ParseError> {
        match self.current_kind() {
            TokenKind::TypeIdentifier => {
                return self.type_identifier(name);
            }
            _ => {
                return self.var_identifier(name);
            }
        }
    }

    pub fn parse_qualified_name(&mut self) -> Result<String, ParseError> {
        let mut name = String::new();
        loop {
            match self.current_kind() {
                TokenKind::TypeIdentifier => {
                    let n = self.type_identifier("type name")?;
                    name += &n;
                }
                TokenKind::VarIdentifier => {
                    let n = self.var_identifier("var name")?;
                    name += &n;
                    break;
                }
                _ => {
                    break;
                }
            }
            if self.current(TokenKind::Dot) {
                name.push('.');
                self.expect(TokenKind::Dot)?;
            } else {
                break;
            }
        }
        Ok(name)
    }

    fn parse_class(&mut self, module: &mut Module) -> Result<Class, ParseError> {
        self.expect(TokenKind::KeywordClass)?;
        let constraints = if self.current_kind() == TokenKind::LParen {
            let cs = self.parse_list1_in_parens(parse_class_constraint)?;
            self.expect(TokenKind::KeywordConstraint)?;
            cs
        } else {
            Vec::new()
        };
        let start_index = self.get_index();
        let name = self.type_identifier("class name")?;
        let end_index = self.get_index();
        let class_location_id = self.get_location_id(start_index, end_index);
        let arg = self.parse_function_type(false, false)?;
        let mut member_functions = BTreeMap::new();
        let mut member_function_types = BTreeMap::new();
        if self.current_kind() == TokenKind::KeywordWhere {
            self.expect(TokenKind::KeywordWhere)?;
            while self.current_kind() != TokenKind::EndOfBlock {
                let function_or_type = self.parse_function_or_function_type()?;
                match function_or_type {
                    FunctionOrFunctionType::Function(name, function_id) => {
                        let fs = member_functions.entry(name).or_insert_with(|| Vec::new());
                        fs.push(function_id);
                    }
                    FunctionOrFunctionType::FunctionType(name, function_type_id) => {
                        let fs = member_function_types
                            .entry(name)
                            .or_insert_with(|| Vec::new());
                        fs.push(function_type_id);
                    }
                }
            }
            self.expect(TokenKind::EndOfBlock)?;
        }
        self.expect(TokenKind::EndOfItem)?;
        let id = self.program.classes.get_id();
        module.classes.push(id);
        let class = Class {
            id: id,
            name: name,
            arg: arg,
            constraints: constraints,
            member_functions: member_functions,
            member_function_types: member_function_types,
            location_id: class_location_id,
        };
        Ok(class)
    }

    fn parse_instance(&mut self, module: &mut Module) -> Result<Instance, ParseError> {
        self.expect(TokenKind::KeywordInstance)?;
        let name = if self.current_kind() == TokenKind::KeywordAs {
            self.expect(TokenKind::KeywordAs)?;
            let name = self.type_identifier("instance name")?;
            Some(name)
        } else {
            None
        };
        let constraints = if self.current_kind() == TokenKind::LParen {
            let cs = self.parse_list1_in_parens(parse_class_constraint)?;
            self.expect(TokenKind::KeywordConstraint)?;
            cs
        } else {
            Vec::new()
        };
        let class_name = self.type_identifier("class name")?;
        let start_index = self.get_index();
        let type_signature_id = self.parse_function_type(false, false)?;
        let end_index = self.get_index();
        let instance_location_id = self.get_location_id(start_index, end_index);
        let mut member_functions = BTreeMap::new();
        let mut member_function_types = BTreeMap::new();
        if self.current_kind() == TokenKind::KeywordWhere {
            self.expect(TokenKind::KeywordWhere)?;
            while self.current_kind() != TokenKind::EndOfBlock {
                let function_or_type = self.parse_function_or_function_type()?;
                match function_or_type {
                    FunctionOrFunctionType::Function(name, function_id) => {
                        let fs = member_functions.entry(name).or_insert_with(|| Vec::new());
                        fs.push(function_id);
                    }
                    FunctionOrFunctionType::FunctionType(name, function_type_id) => {
                        let fs = member_function_types
                            .entry(name)
                            .or_insert_with(|| Vec::new());
                        fs.push(function_type_id);
                    }
                }
            }
            self.expect(TokenKind::EndOfBlock)?;
        }
        self.expect(TokenKind::EndOfItem)?;
        let id = self.program.instances.get_id();
        module.instances.push(id);
        let instance = Instance {
            id: id,
            name: name,
            class_name: class_name,
            type_signature_id: type_signature_id,
            constraints: constraints,
            member_functions: member_functions,
            member_function_types: member_function_types,
            location_id: instance_location_id,
        };
        Ok(instance)
    }

    fn parse_protocol(&mut self, module: &mut Module) -> Result<Protocol, ParseError> {
        self.expect(TokenKind::KeywordProtocol)?;
        let start_index = self.get_index();
        let name = self.type_identifier("protocol name")?;
        let end_index = self.get_index();
        let location_id = self.get_location_id(start_index, end_index);
        self.expect(TokenKind::Equal)?;
        let type_signature_id = self.parse_function_type(false, false)?;
        self.expect(TokenKind::EndOfItem)?;
        let id = self.program.protocols.get_id();
        module.protocols.push(id);
        let protocol = Protocol {
            id: id,
            name: name,
            type_signature: type_signature_id,
            location_id: location_id,
        };
        Ok(protocol)
    }

    fn parse_actor(&mut self, module: &mut Module) -> Result<Actor, ParseError> {
        self.expect(TokenKind::KeywordActor)?;
        let start_index = self.get_index();
        let name = self.type_identifier("actor name")?;
        let end_index = self.get_index();
        let location_id = self.get_location_id(start_index, end_index);
        let type_signature_id = self.parse_function_type(false, false)?;
        self.expect(TokenKind::KeywordWhere)?;
        let mut handlers = Vec::new();
        loop {
            let start_index = self.get_index();
            let protocol_name = self.type_identifier("protocol name")?;
            let end_index = self.get_index();
            let protocol_location_id = self.get_location_id(start_index, end_index);
            self.expect(TokenKind::Op(BuiltinOperator::Arrow))?;
            let start_index = self.get_index();
            let handler_func_name = self.parse_qualified_name()?;
            let end_index = self.get_index();
            let handler_func_location_id = self.get_location_id(start_index, end_index);
            let handler = ProtocolHandler {
                protocol: protocol_name,
                protocol_location_id: protocol_location_id,
                handler_func: handler_func_name,
                handler_func_location_id: handler_func_location_id,
            };
            handlers.push(handler);
            self.expect(TokenKind::EndOfItem)?;
            if self.current_kind() == TokenKind::EndOfBlock {
                break;
            }
        }
        self.expect(TokenKind::EndOfBlock)?;
        self.expect(TokenKind::EndOfItem)?;
        let id = self.program.actors.get_id();
        module.actors.push(id);
        let actor = Actor {
            id: id,
            name: name,
            type_signature: type_signature_id,
            handlers: handlers,
            location_id: location_id,
        };
        Ok(actor)
    }

    fn parse_module(&mut self, id: ModuleId) -> Result<Module, ParseError> {
        self.expect(TokenKind::KeywordModule)?;
        let start_index = self.get_index();
        let name = self.parse_module_name()?;
        let end_index = self.get_index();
        let location_id = self.get_location_id(start_index, end_index);
        let export_list = self.parse_export_import_list()?;
        let mut module = Module::new(name, id, location_id, export_list);
        self.expect(TokenKind::KeywordWhere)?;
        loop {
            if let Some(token) = self.peek() {
                match token.token.kind() {
                    TokenKind::KeywordImport => {
                        let import_id = self.program.imports.get_id();
                        let import = self.parse_import(import_id)?;
                        self.program.imports.add_item(import_id, import);
                        module.imports.push(import_id);
                    }
                    TokenKind::KeywordData => {
                        let data = self.parse_data()?;
                        self.expect(TokenKind::EndOfItem)?;
                        match data {
                            Data::Record(record) => {
                                module.records.push(record.id);
                                self.program.records.add_item(record.id, record);
                            }
                            Data::Adt(adt) => {
                                module.adts.push(adt.id);
                                self.program.adts.add_item(adt.id, adt);
                            }
                        }
                    }
                    TokenKind::KeywordClass => {
                        let class = self.parse_class(&mut module)?;
                        self.program.classes.add_item(class.id, class);
                    }
                    TokenKind::KeywordInstance => {
                        let instance = self.parse_instance(&mut module)?;
                        self.program.instances.add_item(instance.id, instance);
                    }
                    TokenKind::KeywordProtocol => {
                        let protocol = self.parse_protocol(&mut module)?;
                        self.program.protocols.add_item(protocol.id, protocol);
                    }
                    TokenKind::KeywordActor => {
                        let actor = self.parse_actor(&mut module)?;
                        self.program.actors.add_item(actor.id, actor);
                    }
                    TokenKind::EndOfBlock => {
                        break;
                    }
                    _ => match self.parse_function_or_function_type()? {
                        FunctionOrFunctionType::Function(name, function_id) => {
                            let fs = module.functions.entry(name).or_insert_with(|| Vec::new());
                            fs.push(function_id);
                        }
                        FunctionOrFunctionType::FunctionType(name, function_type_id) => {
                            let fs = module
                                .function_types
                                .entry(name)
                                .or_insert_with(|| Vec::new());
                            fs.push(function_type_id);
                        }
                    },
                }
            } else {
                break;
            }
        }
        self.expect(TokenKind::EndOfBlock)?;
        self.expect(TokenKind::EndOfModule)?;
        Ok(module)
    }

    pub fn parse(&mut self) -> Result<(), ParseError> {
        while !self.is_done() {
            let m_id = self.program.modules.get_id();
            let module = self.parse_module(m_id)?;
            self.program.modules.add_item(m_id, module);
        }

        let implicit_modules = get_implicit_module_list();
        for implicit_module in implicit_modules {
            let mut modules_without_implicit_module = Vec::new();
            for (module_id, module) in &self.program.modules.items {
                let mut explicitly_imported = false;
                if module.name == implicit_module {
                    continue;
                }
                for import_id in &module.imports {
                    let import = self.program.imports.get(import_id);
                    if import.module_path == implicit_module {
                        explicitly_imported = true;
                        break;
                    }
                }
                if !explicitly_imported {
                    modules_without_implicit_module.push(*module_id);
                }
            }
            for module_id in modules_without_implicit_module {
                let import_id = self.program.imports.get_id();
                let import = Import {
                    id: import_id,
                    module_path: implicit_module.to_string(),
                    kind: ImportKind::ImportList {
                        items: EIList::ImplicitAll,
                        alternative_name: None,
                    },
                    location_id: None,
                    implicit: true,
                };
                self.program.imports.add_item(import_id, import);
                let module = self.program.modules.get_mut(&module_id);
                module.imports.push(import_id);
            }
        }
        Ok(())
    }
}
