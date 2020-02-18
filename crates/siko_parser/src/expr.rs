use super::util::parse_parens;
use super::util::report_unexpected_token;
use super::util::ParenParseResult;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::Token;
use crate::token::TokenKind;
use siko_constants::BuiltinOperator;
use siko_syntax::expr::Case;
use siko_syntax::expr::Expr;
use siko_syntax::expr::ExprId;
use siko_syntax::expr::RecordConstructionItem;
use siko_syntax::pattern::Pattern;
use siko_syntax::pattern::PatternId;
use siko_syntax::pattern::RangeKind;
use siko_syntax::pattern::RecordFieldPattern;

fn parse_paren_expr(parser: &mut Parser) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    let res = parse_parens(parser, |p| p.parse_expr(), " expression")?;
    match res {
        ParenParseResult::Single(e) => {
            return Ok(e);
        }
        ParenParseResult::Tuple(exprs) => {
            let expr = Expr::Tuple(exprs);
            let id = parser.add_expr(expr, start_index);
            return Ok(id);
        }
    }
}

fn parse_list_expr(parser: &mut Parser) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    parser.expect(TokenKind::LBracket)?;
    let mut items = Vec::new();
    loop {
        if parser.current(TokenKind::RBracket) {
            break;
        }
        let expr = parser.parse_expr()?;
        items.push(expr);
        if parser.current(TokenKind::Comma) {
            parser.expect(TokenKind::Comma)?;
            continue;
        }
    }
    parser.expect(TokenKind::RBracket)?;
    let expr = Expr::List(items);
    let id = parser.add_expr(expr, start_index);
    return Ok(id);
}

fn parse_lambda(parser: &mut Parser) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    parser.expect(TokenKind::Lambda)?;
    let args = parser.parse_lambda_args()?;
    parser.expect(TokenKind::Op(BuiltinOperator::Arrow))?;
    let body_expr_id = parser.parse_expr()?;
    let mut temp_arg_exprs = Vec::new();
    let mut temp_args = Vec::new();
    for arg in args.iter() {
        let location = parser.get_program().patterns.get(arg).location_id;
        let temp_arg_name = parser.get_temp_var_name();
        temp_args.push((temp_arg_name.clone(), location));
        let path_expr = Expr::Path(temp_arg_name);
        let path_expr_id = parser.add_expr(path_expr, start_index);
        temp_arg_exprs.push(path_expr_id);
    }
    let tuple_expr = Expr::Tuple(temp_arg_exprs);
    let tuple_expr_id = parser.add_expr(tuple_expr, start_index);
    let tuple_pattern = Pattern::Tuple(args.clone());
    let tuple_pattern_id = parser.add_pattern(tuple_pattern, start_index);
    let bind_expr = Expr::Bind(tuple_pattern_id, tuple_expr_id);
    let bind_expr_id = parser.add_expr(bind_expr, start_index);
    let do_expr = Expr::Do(vec![bind_expr_id, body_expr_id]);
    let do_expr_id = parser.add_expr(do_expr, start_index);
    let lambda_expr = Expr::Lambda(temp_args, do_expr_id);
    let lambda_expr_id = parser.add_expr(lambda_expr, start_index);
    Ok(lambda_expr_id)
}

fn parse_do(parser: &mut Parser) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    parser.expect(TokenKind::KeywordDo)?;
    let mut exprs = Vec::new();
    loop {
        let bind_start_index = parser.get_index();
        let mut pattern_id = None;
        if parser.irrefutable_pattern_follows() {
            let mut inner_pattern_id = parse_pattern(parser)?;
            if parser.current(TokenKind::KeywordDoubleColon) {
                parser.expect(TokenKind::KeywordDoubleColon)?;
                let type_signature_id = parser.parse_function_type(false, true)?;
                let pattern = Pattern::Typed(inner_pattern_id, type_signature_id);
                inner_pattern_id = parser.add_pattern(pattern, bind_start_index);
            }
            parser.expect(TokenKind::Op(BuiltinOperator::Bind))?;
            pattern_id = Some(inner_pattern_id);
        }
        let expr = parser.parse_expr()?;
        parser.expect(TokenKind::EndOfItem)?;
        if let Some(pattern_id) = pattern_id {
            let expr = Expr::Bind(pattern_id, expr);
            let id = parser.add_expr(expr, bind_start_index);
            exprs.push(id);
        } else {
            exprs.push(expr);
        }
        if parser.current(TokenKind::EndOfBlock) {
            break;
        }
    }
    let expr = Expr::Do(exprs);
    let id = parser.add_expr(expr, start_index);
    parser.expect(TokenKind::EndOfBlock)?;
    Ok(id)
}

fn parse_tuple_pattern(parser: &mut Parser) -> Result<PatternId, ParseError> {
    let start_index = parser.get_index();
    let res = parse_parens(parser, |p| parse_pattern(p), "<pattern>")?;
    match res {
        ParenParseResult::Single(t) => {
            return Ok(t);
        }
        ParenParseResult::Tuple(ts) => {
            let pattern = Pattern::Tuple(ts);
            let id = parser.add_pattern(pattern, start_index);
            return Ok(id);
        }
    }
}

fn parse_record_field_pattern(parser: &mut Parser) -> Result<RecordFieldPattern, ParseError> {
    let start_index = parser.get_index();
    let field_name = parser.var_identifier("field name")?;
    let end_index = parser.get_index();
    parser.expect(TokenKind::Equal)?;
    let location_id = parser.get_location_id(start_index, end_index);
    let value = parse_sub_pattern(parser, true)?;
    if let Some(value) = value {
        let item = RecordFieldPattern {
            name: field_name,
            value: value,
            location_id: location_id,
        };
        Ok(item)
    } else {
        unimplemented!()
    }
}

fn parse_sub_pattern(parser: &mut Parser, inner: bool) -> Result<Option<PatternId>, ParseError> {
    let id = match parser.current_kind() {
        TokenKind::LParen => {
            let id = parse_tuple_pattern(parser)?;
            id
        }
        TokenKind::IntegerLiteral => {
            let start_index = parser.get_index();
            let literal = parser.advance()?;
            if let Token::IntegerLiteral(i) = literal.token {
                let pattern = Pattern::IntegerLiteral(i);
                let id = parser.add_pattern(pattern, start_index);
                id
            } else {
                unreachable!()
            }
        }
        TokenKind::CharLiteral => {
            let start_index = parser.get_index();
            let literal = parser.advance()?;
            if let Token::CharLiteral(c) = literal.token {
                if parser.current(TokenKind::DoubleDot) {
                    parser.expect(TokenKind::DoubleDot)?;
                    if parser.current_kind() == TokenKind::CharLiteral {
                        let literal = parser.advance()?;
                        if let Token::CharLiteral(c2) = literal.token {
                            let pattern = Pattern::CharRange(c, c2, RangeKind::Exclusive);
                            let id = parser.add_pattern(pattern, start_index);
                            id
                        } else {
                            unreachable!()
                        }
                    } else {
                        return report_unexpected_token(parser, format!("char literal"));
                    }
                } else if parser.current(TokenKind::InclusiveRange) {
                    parser.expect(TokenKind::InclusiveRange)?;
                    if parser.current_kind() == TokenKind::CharLiteral {
                        let literal = parser.advance()?;
                        if let Token::CharLiteral(c2) = literal.token {
                            let pattern = Pattern::CharRange(c, c2, RangeKind::Inclusive);
                            let id = parser.add_pattern(pattern, start_index);
                            id
                        } else {
                            unreachable!()
                        }
                    } else {
                        return report_unexpected_token(parser, format!("char literal"));
                    }
                } else {
                    let pattern = Pattern::CharLiteral(c);
                    let id = parser.add_pattern(pattern, start_index);
                    id
                }
            } else {
                unreachable!()
            }
        }
        TokenKind::StringLiteral => {
            let start_index = parser.get_index();
            let literal = parser.advance()?;
            if let Token::StringLiteral(s) = literal.token {
                let pattern = Pattern::StringLiteral(s);
                let id = parser.add_pattern(pattern, start_index);
                id
            } else {
                unreachable!()
            }
        }
        TokenKind::VarIdentifier => {
            let start_index = parser.get_index();
            let name = parser.var_identifier("pattern binding")?;
            let pattern = Pattern::Binding(name);
            let id = parser.add_pattern(pattern, start_index);
            id
        }
        TokenKind::TypeIdentifier => {
            if inner {
                let start_index = parser.get_index();
                let name = parser.parse_qualified_type_name()?;
                let id = parser.add_pattern(Pattern::Constructor(name, Vec::new()), start_index);
                id
            } else {
                let start_index = parser.get_index();
                let name = parser.parse_qualified_type_name()?;
                if parser.current(TokenKind::LCurly) {
                    let items = parser.parse_list0_in_curly_parens(parse_record_field_pattern)?;
                    let pattern = Pattern::Record(name, items);
                    let id = parser.add_pattern(pattern, start_index);
                    id
                } else {
                    let mut args = Vec::new();
                    loop {
                        let arg = match parse_sub_pattern(parser, true)? {
                            Some(arg) => arg,
                            None => {
                                break;
                            }
                        };
                        args.push(arg);
                    }
                    let pattern = Pattern::Constructor(name, args);
                    let id = parser.add_pattern(pattern, start_index);
                    id
                }
            }
        }
        TokenKind::Wildcard => {
            let start_index = parser.get_index();
            parser.expect(TokenKind::Wildcard)?;
            let pattern = Pattern::Wildcard;
            let id = parser.add_pattern(pattern, start_index);
            id
        }
        _ => {
            return Ok(None);
        }
    };
    Ok(Some(id))
}

pub fn parse_pattern(parser: &mut Parser) -> Result<PatternId, ParseError> {
    let id = parse_sub_pattern(parser, false)?;
    match id {
        Some(id) => Ok(id),
        None => report_unexpected_token(parser, format!("<pattern>")),
    }
}

fn parse_case(parser: &mut Parser) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    parser.expect(TokenKind::KeywordCase)?;
    let body = parser.parse_expr()?;
    parser.expect(TokenKind::KeywordOf)?;
    let mut cases = Vec::new();
    loop {
        let start_index = parser.get_index();
        let mut sub_patterns = Vec::new();
        loop {
            let pattern_id = parse_pattern(parser)?;
            sub_patterns.push(pattern_id);
            if parser.current(TokenKind::Pipe) {
                parser.expect(TokenKind::Pipe)?;
            } else {
                break;
            }
        }
        let mut pattern_id = if sub_patterns.len() == 1 {
            sub_patterns[0]
        } else {
            let pattern = Pattern::Or(sub_patterns);
            parser.add_pattern(pattern, start_index)
        };
        if parser.current(TokenKind::KeywordIf) {
            parser.expect(TokenKind::KeywordIf)?;
            let guard_expr = parser.parse_expr()?;
            let pattern = Pattern::Guarded(pattern_id, guard_expr);
            pattern_id = parser.add_pattern(pattern, start_index);
        }
        parser.expect(TokenKind::Op(BuiltinOperator::Arrow))?;
        let case_body = parser.parse_expr()?;
        parser.expect(TokenKind::EndOfItem)?;
        let case = Case {
            pattern_id: pattern_id,
            body: case_body,
        };
        cases.push(case);
        if parser.current(TokenKind::EndOfBlock) {
            break;
        }
    }
    let expr = Expr::CaseOf(body, cases);
    let id = parser.add_expr(expr, start_index);
    parser.expect(TokenKind::EndOfBlock)?;
    Ok(id)
}

fn parse_if(parser: &mut Parser) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    parser.expect(TokenKind::KeywordIf)?;
    let cond = parser.parse_expr()?;
    parser.expect(TokenKind::KeywordThen)?;
    let true_branch = parser.parse_expr()?;
    parser.expect(TokenKind::KeywordElse)?;
    let false_branch = parser.parse_expr()?;
    let expr = Expr::If(cond, true_branch, false_branch);
    let id = parser.add_expr(expr, start_index);
    Ok(id)
}

fn parse_record_construction_item(
    parser: &mut Parser,
) -> Result<RecordConstructionItem, ParseError> {
    let start_index = parser.get_index();
    let field_name = parser.var_identifier("field name")?;
    let end_index = parser.get_index();
    parser.expect(TokenKind::Equal)?;
    let location_id = parser.get_location_id(start_index, end_index);
    let body = parser.parse_expr()?;
    let item = RecordConstructionItem {
        field_name: field_name,
        body: body,
        location_id: location_id,
    };
    Ok(item)
}

fn record_construction(
    parser: &mut Parser,
    name: String,
    start_index: usize,
    initialization: bool,
) -> Result<ExprId, ParseError> {
    let items = parser.parse_list0_in_curly_parens(parse_record_construction_item)?;
    let expr = if initialization {
        Expr::RecordInitialization(name, items)
    } else {
        Expr::RecordUpdate(name, items)
    };
    let id = parser.add_expr(expr, start_index);
    Ok(id)
}

fn parse_arg(parser: &mut Parser) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    let token_info = parser.peek().expect("Ran out of tokens");
    let id = match token_info.token {
        Token::TypeIdentifier(..) => {
            let path = parser.parse_qualified_name()?;
            if parser.current(TokenKind::LCurly) {
                return record_construction(parser, path, start_index, true);
            } else {
                let expr = Expr::Path(path);
                let id = parser.add_expr(expr, start_index);
                id
            }
        }
        Token::VarIdentifier(..) => {
            let path = parser.var_identifier("identifier")?;
            if parser.current(TokenKind::LCurly) {
                return record_construction(parser, path, start_index, false);
            } else {
                let expr = Expr::Path(path);
                let id = parser.add_expr(expr, start_index);
                id
            }
        }
        Token::IntegerLiteral(n) => {
            parser.advance()?;
            let expr = Expr::IntegerLiteral(n);
            let id = parser.add_expr(expr, start_index);
            id
        }
        Token::FloatLiteral(f) => {
            parser.advance()?;
            let expr = Expr::FloatLiteral(f);
            let id = parser.add_expr(expr, start_index);
            id
        }
        Token::CharLiteral(c) => {
            parser.advance()?;
            let expr = Expr::CharLiteral(c);
            let id = parser.add_expr(expr, start_index);
            id
        }
        Token::StringLiteral(s) => {
            parser.advance()?;
            if parser.current(TokenKind::Formatter) {
                parser.expect(TokenKind::Formatter)?;
                let items = if parser.current(TokenKind::LParen) {
                    parser.parse_list1_in_parens(|p| parse_ops(p))?
                } else {
                    let item = parser.parse_expr()?;
                    vec![item]
                };
                let expr = Expr::Formatter(s, items);
                let id = parser.add_expr(expr, start_index);
                id
            } else {
                let expr = Expr::StringLiteral(s);
                let id = parser.add_expr(expr, start_index);
                id
            }
        }
        Token::LParen => {
            return parse_paren_expr(parser);
        }
        Token::LBracket => {
            return parse_list_expr(parser);
        }
        Token::KeywordIf => {
            return parse_if(parser);
        }
        Token::KeywordDo => {
            return parse_do(parser);
        }
        Token::Lambda => {
            return parse_lambda(parser);
        }
        Token::KeywordCase => {
            return parse_case(parser);
        }
        Token::KeywordReturn => {
            parser.expect(TokenKind::KeywordReturn)?;
            let item = parser.parse_expr()?;
            let expr = Expr::Return(item);
            let id = parser.add_expr(expr, start_index);
            id
        }
        _ => {
            return report_unexpected_token(parser, format!("expression"));
        }
    };
    Ok(id)
}

fn parse_primary(parser: &mut Parser) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    let f = parse_unary(parser, false)?;
    let mut args = Vec::new();
    loop {
        match parser.current_kind() {
            TokenKind::Op(BuiltinOperator::Not)
            | TokenKind::TypeIdentifier
            | TokenKind::VarIdentifier
            | TokenKind::IntegerLiteral
            | TokenKind::FloatLiteral
            | TokenKind::StringLiteral
            | TokenKind::CharLiteral
            | TokenKind::LParen
            | TokenKind::KeywordIf
            | TokenKind::KeywordDo
            | TokenKind::LBracket
            | TokenKind::Lambda => {}
            _ => break,
        }
        let arg = parse_unary(parser, true)?;
        args.push(arg);
    }
    if args.is_empty() {
        Ok(f)
    } else {
        let expr = Expr::FunctionCall(f, args);
        let id = parser.add_expr(expr, start_index);
        Ok(id)
    }
}

fn parse_unary(parser: &mut Parser, is_arg: bool) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    let ops: &[BuiltinOperator] = if is_arg {
        &[BuiltinOperator::Not]
    } else {
        &[BuiltinOperator::Not, BuiltinOperator::Sub]
    };
    if let Some((op, _)) = parser.consume_op(ops) {
        let function_id_expr = Expr::Builtin(op);
        let function_id_expr_id = parser.add_expr(function_id_expr, start_index);
        let right = parse_unary(parser, is_arg)?;
        let op = if op == BuiltinOperator::Sub {
            BuiltinOperator::Minus
        } else {
            op
        };
        if op == BuiltinOperator::Minus {
            let right_expr_info = parser.get_program().exprs.get_mut(&right);
            // FIXME: fix location of these literals
            if let Expr::IntegerLiteral(n) = right_expr_info.item {
                right_expr_info.item = Expr::IntegerLiteral(-n);
                return Ok(right);
            }
            if let Expr::FloatLiteral(n) = right_expr_info.item {
                right_expr_info.item = Expr::FloatLiteral(-n);
                return Ok(right);
            }
        }
        let expr = Expr::FunctionCall(function_id_expr_id, vec![right]);
        let id = parser.add_expr(expr, start_index);
        Ok(id)
    } else {
        return parse_field_access(parser);
    }
}

fn parse_binary_op(
    parser: &mut Parser,
    ops: &[BuiltinOperator],
    next: fn(&mut Parser) -> Result<ExprId, ParseError>,
) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    let mut left = next(parser)?;
    loop {
        if let Some((op, _)) = parser.consume_op(ops) {
            let function_id_expr = Expr::Builtin(op);
            let function_id_expr_id = parser.add_expr(function_id_expr, start_index);
            let right = next(parser)?;
            let expr = Expr::FunctionCall(function_id_expr_id, vec![left, right]);
            let id = parser.add_expr(expr, start_index);
            left = id;
            continue;
        } else {
            break;
        }
    }
    Ok(left)
}

pub fn parse_ops(parser: &mut Parser) -> Result<ExprId, ParseError> {
    return parse_andor(parser);
}

fn parse_andor(parser: &mut Parser) -> Result<ExprId, ParseError> {
    return parse_binary_op(
        parser,
        &[BuiltinOperator::And, BuiltinOperator::Or],
        parse_equal,
    );
}

fn parse_equal(parser: &mut Parser) -> Result<ExprId, ParseError> {
    return parse_binary_op(
        parser,
        &[BuiltinOperator::Equals, BuiltinOperator::NotEquals],
        parse_ord_ops,
    );
}

fn parse_ord_ops(parser: &mut Parser) -> Result<ExprId, ParseError> {
    return parse_binary_op(
        parser,
        &[
            BuiltinOperator::LessThan,
            BuiltinOperator::LessOrEqualThan,
            BuiltinOperator::GreaterThan,
            BuiltinOperator::GreaterOrEqualThan,
        ],
        parse_addsub,
    );
}

fn parse_addsub(parser: &mut Parser) -> Result<ExprId, ParseError> {
    return parse_binary_op(
        parser,
        &[BuiltinOperator::Add, BuiltinOperator::Sub],
        parse_muldiv,
    );
}

fn parse_muldiv(parser: &mut Parser) -> Result<ExprId, ParseError> {
    return parse_binary_op(
        parser,
        &[BuiltinOperator::Mul, BuiltinOperator::Div],
        parse_pipe_forward,
    );
}

fn parse_pipe_forward(parser: &mut Parser) -> Result<ExprId, ParseError> {
    return parse_binary_op(parser, &[BuiltinOperator::PipeForward], parse_primary);
}

fn parse_field_access(parser: &mut Parser) -> Result<ExprId, ParseError> {
    let start_index = parser.get_index();
    let mut left = parse_arg(parser)?;
    loop {
        if let Some(dot_token) = parser.peek() {
            if dot_token.token.kind() == TokenKind::Dot {
                parser.expect(TokenKind::Dot)?;
                if let Some(next) = parser.peek() {
                    match next.token {
                        Token::VarIdentifier(_) => {
                            let field_name = parser.var_identifier("field name")?;
                            let expr = match field_name.parse::<usize>() {
                                Ok(n) => Expr::TupleFieldAccess(n, left),
                                Err(_) => Expr::FieldAccess(field_name, left),
                            };
                            let id = parser.add_expr(expr, start_index);
                            left = id;
                            continue;
                        }
                        Token::IntegerLiteral(id) => {
                            parser.advance()?;
                            let expr = Expr::TupleFieldAccess(id as usize, left);
                            let id = parser.add_expr(expr, start_index);
                            left = id;
                            continue;
                        }
                        _ => {
                            return report_unexpected_token(
                                parser,
                                format!("field name or tuple member"),
                            );
                        }
                    }
                } else {
                    return report_unexpected_token(parser, format!("expression"));
                }
            } else {
                break;
            }
        }
    }
    Ok(left)
}
