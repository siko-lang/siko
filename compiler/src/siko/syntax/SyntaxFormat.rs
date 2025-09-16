use crate::siko::syntax::{Format::format_block_2_items, Function::ResultKind};

use super::{
    Data::{Enum, Field, Struct, Variant},
    Effect::Effect,
    Expr::{BinaryOp, Branch, ContextHandler, Expr, SimpleExpr, UnaryOp, With},
    Format::{escape_char, escape_string, format_block, format_list, format_list2, Format, Token},
    Function::{Function, FunctionExternKind, Parameter},
    Identifier::Identifier,
    Implicit::Implicit,
    Module::{Derive, Import, Module, ModuleItem},
    Pattern::{Pattern, SimplePattern},
    Statement::{Block, Statement, StatementKind},
    Trait::{AssociatedType, AssociatedTypeDeclaration, Instance, Trait},
    Type::{Constraint, ConstraintArgument, Type, TypeParameterDeclaration},
};

impl Format for Identifier {
    fn format(&self) -> Vec<Token> {
        vec![Token::Chunk(self.toString())]
    }
}

impl Format for BinaryOp {
    fn format(&self) -> Vec<Token> {
        let op_str = match self {
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::LessThan => "<",
            BinaryOp::GreaterThan => ">",
            BinaryOp::LessThanOrEqual => "<=",
            BinaryOp::GreaterThanOrEqual => ">=",
        };
        vec![Token::Chunk(format!(" {} ", op_str))]
    }
}

impl Format for UnaryOp {
    fn format(&self) -> Vec<Token> {
        let op_str = match self {
            UnaryOp::Not => "!",
            UnaryOp::Neg => "-",
            UnaryOp::Deref => "*",
        };
        vec![Token::Chunk(op_str.to_string())]
    }
}

impl Format for Expr {
    fn format(&self) -> Vec<Token> {
        self.expr.format()
    }
}

impl Format for SimpleExpr {
    fn format(&self) -> Vec<Token> {
        match self {
            SimpleExpr::Value(name) => name.format(),
            SimpleExpr::SelfValue => vec![Token::Chunk("self".to_string())],
            SimpleExpr::Name(name) => name.format(),
            SimpleExpr::FieldAccess(receiver, field) => {
                let mut result = receiver.format();
                result.push(Token::Chunk(".".to_string()));
                result.extend(field.format());
                result
            }
            SimpleExpr::TupleIndex(receiver, index) => {
                let mut result = receiver.format();
                result.push(Token::Chunk(format!(".{}", index)));
                result
            }
            SimpleExpr::Call(func, args) => {
                if args.len() > 6 {
                    let args_output = format_list2(args, &[Token::Chunk(", ".to_string()), Token::Break]);
                    let mut result = func.format();
                    result.push(Token::Chunk("(".to_string()));
                    result.push(Token::PushOffset);
                    result.extend(args_output);
                    result.push(Token::PopOffset);
                    result.push(Token::Chunk(")".to_string()));
                    result
                } else {
                    let args_output = format_list(args, Token::Chunk(", ".to_string()));
                    let mut result = func.format();
                    if !args.is_empty() {
                        result.push(Token::Chunk("(".to_string()));
                        result.push(Token::PushOffset);
                        result.extend(args_output);
                        result.push(Token::PopOffset);
                        result.push(Token::Chunk(")".to_string()));
                    }
                    result
                }
            }
            SimpleExpr::MethodCall(receiver, method, args) => {
                let mut result = receiver.format();
                result.push(Token::Chunk(".".to_string()));
                result.extend(method.format());
                if !args.is_empty() {
                    result.push(Token::Chunk("(".to_string()));
                    result.extend(format_list(args, Token::Chunk(", ".to_string())));
                    result.push(Token::Chunk(")".to_string()));
                } else {
                    result.push(Token::Chunk("()".to_string()));
                }
                result
            }
            SimpleExpr::Loop(pattern, init, body) => {
                let mut result = vec![Token::Chunk("loop ".to_string())];
                result.extend(pattern.format());
                result.push(Token::Chunk(" = ".to_string()));
                result.extend(init.format());
                result.push(Token::Chunk(" ".to_string()));
                result.extend(body.format());
                result
            }
            SimpleExpr::BinaryOp(op, left, right) => {
                let mut result = left.format();
                result.extend(op.format());
                result.extend(right.format());
                result
            }
            SimpleExpr::UnaryOp(op, expr) => {
                let mut result = op.format();
                result.extend(expr.format());
                result
            }
            SimpleExpr::Match(expr, branches) => {
                let mut result = vec![Token::Chunk("match ".to_string())];
                result.extend(expr.format());
                result.extend(format_block(branches));
                result
            }
            SimpleExpr::Block(block) => block.format(),
            SimpleExpr::Tuple(exprs) => {
                let mut result = vec![Token::Chunk("(".to_string())];
                result.extend(format_list(exprs, Token::Chunk(", ".to_string())));
                result.push(Token::Chunk(")".to_string()));
                result
            }
            SimpleExpr::StringLiteral(s) => {
                vec![Token::Chunk(format!("\"{}\"", escape_string(s)))]
            }
            SimpleExpr::IntegerLiteral(i) => vec![Token::Chunk(i.clone())],
            SimpleExpr::CharLiteral(c) => vec![Token::Chunk(format!(
                "'{}'",
                escape_char(c.chars().next().unwrap_or('\0'))
            ))],
            SimpleExpr::Return(None) => vec![Token::Chunk("return".to_string())],
            SimpleExpr::Return(Some(expr)) => {
                let mut result = vec![Token::Chunk("return ".to_string())];
                result.extend(expr.format());
                result
            }
            SimpleExpr::Break(None) => vec![Token::Chunk("break".to_string())],
            SimpleExpr::Break(Some(expr)) => {
                let mut result = vec![Token::Chunk("break ".to_string())];
                result.extend(expr.format());
                result
            }
            SimpleExpr::Continue(None) => vec![Token::Chunk("continue".to_string())],
            SimpleExpr::Continue(Some(expr)) => {
                let mut result = vec![Token::Chunk("continue ".to_string())];
                result.extend(expr.format());
                result
            }
            SimpleExpr::Ref(expr) => {
                let mut result = vec![Token::Chunk("&".to_string())];
                result.extend(expr.format());
                result
            }
            SimpleExpr::List(exprs) => {
                if exprs.len() > 3 {
                    let mut result = vec![Token::Chunk("[".to_string()), Token::PushOffset];
                    result.extend(format_list2(exprs, &[Token::Chunk(",".to_string()), Token::Break]));
                    result.push(Token::Chunk("]".to_string()));
                    result.push(Token::PopOffset);
                    result
                } else {
                    let mut result = vec![Token::Chunk("[".to_string())];
                    result.extend(format_list(exprs, Token::Chunk(", ".to_string())));
                    result.push(Token::Chunk("]".to_string()));
                    result
                }
            }
            SimpleExpr::With(with) => with.format(),
            SimpleExpr::Lambda(params, body) => {
                let mut result = vec![Token::Chunk("\\".to_string())];
                result.extend(format_list(params, Token::Chunk(", ".to_string())));
                result.push(Token::Chunk(" -> ".to_string()));
                result.extend(body.format());
                result
            }
            SimpleExpr::Yield(expr) => {
                let mut result = vec![Token::Chunk("yield ".to_string())];
                result.extend(expr.format());
                result
            }
            SimpleExpr::CreateGenerator(expr) => {
                let mut result = vec![Token::Chunk("gen ".to_string())];
                result.extend(expr.format());
                result
            }
        }
    }
}

impl Format for With {
    fn format(&self) -> Vec<Token> {
        let mut result = vec![Token::Chunk("with { ".to_string())];
        result.extend(format_list(&self.handlers, Token::Chunk(", ".to_string())));
        result.push(Token::Chunk(" } ".to_string()));
        result.extend(self.body.format());
        result
    }
}

impl Format for ContextHandler {
    fn format(&self) -> Vec<Token> {
        let mut result = self.name.format();
        result.push(Token::Chunk(" = ".to_string()));
        result.extend(self.handler.format());
        result
    }
}

impl Format for Branch {
    fn format(&self) -> Vec<Token> {
        let mut result = self.pattern.format();
        result.push(Token::Chunk(" -> ".to_string()));
        result.extend(self.body.format());
        result
    }
}

impl Format for Pattern {
    fn format(&self) -> Vec<Token> {
        self.pattern.format()
    }
}

impl Format for SimplePattern {
    fn format(&self) -> Vec<Token> {
        match self {
            SimplePattern::Named(name, patterns) => {
                if patterns.is_empty() {
                    name.format()
                } else {
                    let mut result = name.format();
                    result.push(Token::Chunk("(".to_string()));
                    result.extend(format_list(patterns, Token::Chunk(", ".to_string())));
                    result.push(Token::Chunk(")".to_string()));
                    result
                }
            }
            SimplePattern::Bind(name, mutable) => {
                if *mutable {
                    let mut result = vec![Token::Chunk("mut ".to_string())];
                    result.extend(name.format());
                    result
                } else {
                    name.format()
                }
            }
            SimplePattern::Tuple(patterns) => {
                let mut result = vec![Token::Chunk("(".to_string())];
                result.extend(format_list(patterns, Token::Chunk(", ".to_string())));
                if patterns.len() == 1 {
                    result.push(Token::Chunk(",".to_string()));
                }
                result.push(Token::Chunk(")".to_string()));
                result
            }
            SimplePattern::StringLiteral(s) => {
                vec![Token::Chunk(format!("\"{}\"", escape_string(s)))]
            }
            SimplePattern::IntegerLiteral(i) => vec![Token::Chunk(i.clone())],
            SimplePattern::Wildcard => vec![Token::Chunk("_".to_string())],
            SimplePattern::Guarded(pattern, expr) => {
                let mut result = pattern.format();
                result.push(Token::Chunk(" if ".to_string()));
                result.extend(expr.format());
                result
            }
        }
    }
}

impl Format for Block {
    fn format(&self) -> Vec<Token> {
        format_block(&self.statements)
    }
}

impl Format for Statement {
    fn format(&self) -> Vec<Token> {
        let mut result = self.kind.format();
        if self.hasSemicolon {
            result.push(Token::Chunk(";".to_string()));
        }
        result
    }
}

impl Format for StatementKind {
    fn format(&self) -> Vec<Token> {
        match self {
            StatementKind::Expr(expr) => expr.format(),
            StatementKind::Assign(lhs, rhs) => {
                let mut result = lhs.format();
                result.push(Token::Chunk(" = ".to_string()));
                result.extend(rhs.format());
                result
            }
            StatementKind::Let(pattern, expr, ty) => {
                let mut result = vec![Token::Chunk("let ".to_string())];
                result.extend(pattern.format());
                if let Some(ty) = ty {
                    result.push(Token::Chunk(": ".to_string()));
                    result.extend(ty.format());
                }
                result.push(Token::Chunk(" = ".to_string()));
                result.extend(expr.format());
                result
            }
        }
    }
}

impl Format for Type {
    fn format(&self) -> Vec<Token> {
        match self {
            Type::Named(name, args) => {
                if args.is_empty() {
                    name.format()
                } else {
                    let mut result = name.format();
                    result.push(Token::Chunk("[".to_string()));
                    result.extend(format_list(args, Token::Chunk(", ".to_string())));
                    result.push(Token::Chunk("]".to_string()));
                    result
                }
            }
            Type::Tuple(elements) => {
                let mut result = vec![Token::Chunk("(".to_string())];
                match elements.len() {
                    1 => {
                        result.extend(elements[0].format());
                        result.push(Token::Chunk(",)".to_string()));
                    }
                    n if n < 4 => {
                        result.extend(format_list(elements, Token::Chunk(", ".to_string())));
                        result.push(Token::Chunk(")".to_string()));
                    }
                    _ => {
                        result.push(Token::PushOffset);
                        result.extend(format_list2(elements, &[Token::Chunk(",".to_string()), Token::Break]));
                        result.push(Token::PopOffset);
                        result.push(Token::Chunk(")".to_string()));
                    }
                }
                result
            }
            Type::Function(params, ret) => {
                if params.len() > 3 {
                    let mut result = vec![Token::PushOffset];
                    result.extend(format_list2(params, &[Token::Chunk(" ->".to_string()), Token::Break]));
                    result.push(Token::Chunk(" -> ".to_string()));
                    result.extend(ret.format());
                    result.push(Token::PopOffset);
                    result
                } else {
                    let mut result = vec![Token::Chunk("fn(".to_string())];
                    result.extend(format_list(params, Token::Chunk(", ".to_string())));
                    result.push(Token::Chunk(") -> ".to_string()));
                    result.extend(ret.format());
                    result
                }
            }
            Type::Reference(ty) => {
                let mut result = vec![Token::Chunk("&".to_string())];
                result.extend(ty.format());
                result
            }
            Type::Ptr(ty) => {
                let mut result = vec![Token::Chunk("*".to_string())];
                result.extend(ty.format());
                result
            }
            Type::SelfType => vec![Token::Chunk("Self".to_string())],
            Type::Never => vec![Token::Chunk("!".to_string())],
            Type::NumericConstant(value) => vec![Token::Chunk(format!("{}", value))],
            Type::Void => vec![Token::Chunk("void".to_string())],
            Type::VoidPtr => vec![Token::Chunk("void*".to_string())],
        }
    }
}

impl Format for TypeParameterDeclaration {
    fn format(&self) -> Vec<Token> {
        let mut result = vec![Token::Chunk("[".to_string())];
        result.extend(format_list(&self.params, Token::Chunk(", ".to_string())));
        if !self.constraints.is_empty() {
            result.push(Token::Chunk(": ".to_string()));
            result.extend(format_list(&self.constraints, Token::Chunk(", ".to_string())));
        }
        result.push(Token::Chunk("]".to_string()));
        result
    }
}

impl Format for Constraint {
    fn format(&self) -> Vec<Token> {
        let mut result = self.name.format();
        if !self.args.is_empty() {
            result.push(Token::Chunk("[".to_string()));
            result.extend(format_list(&self.args, Token::Chunk(", ".to_string())));
            result.push(Token::Chunk("]".to_string()));
        }
        result
    }
}

impl Format for ConstraintArgument {
    fn format(&self) -> Vec<Token> {
        match self {
            ConstraintArgument::Type(ty) => ty.format(),
            ConstraintArgument::AssociatedType(name, ty) => {
                let mut result = name.format();
                result.push(Token::Chunk(" = ".to_string()));
                result.extend(ty.format());
                result
            }
        }
    }
}

impl Format for Parameter {
    fn format(&self) -> Vec<Token> {
        match self {
            Parameter::Named(name, ty, mutable) => {
                let mut result = Vec::new();
                if *mutable {
                    result.push(Token::Chunk("mut ".to_string()));
                }
                result.extend(name.format());
                result.push(Token::Chunk(": ".to_string()));
                result.extend(ty.format());
                result
            }
            Parameter::SelfParam => vec![Token::Chunk("self".to_string())],
            Parameter::MutSelfParam => vec![Token::Chunk("mut self".to_string())],
            Parameter::RefSelfParam => vec![Token::Chunk("&self".to_string())],
        }
    }
}

impl Format for Function {
    fn format(&self) -> Vec<Token> {
        let mut result = Vec::new();

        if self.public {
            result.push(Token::Chunk("pub ".to_string()));
        }

        result.push(Token::Chunk("fn ".to_string()));
        result.extend(self.name.format());

        if let Some(type_params) = &self.typeParams {
            result.extend(type_params.format());
        }

        result.push(Token::Chunk("(".to_string()));
        result.extend(format_list(&self.params, Token::Chunk(", ".to_string())));
        result.push(Token::Chunk(")".to_string()));
        match &self.result {
            ResultKind::SingleReturn(ty) => {
                result.push(Token::Chunk(" -> ".to_string()));
                result.extend(ty.format());
            }
            ResultKind::Generator(yieldTy, retTy) => {
                result.push(Token::Chunk(":".to_string()));
                result.extend(yieldTy.format());
                result.push(Token::Chunk(" -> ".to_string()));
                result.extend(retTy.format());
            }
        }

        match (&self.body, &self.externKind) {
            (Some(body), None) => {
                let body_formatted = body.format();
                let starts_with_block = body_formatted
                    .first()
                    .map_or(false, |token| matches!(token, Token::Chunk(s) if s.starts_with(" {")));

                if starts_with_block {
                    result.extend(body_formatted);
                } else {
                    result.push(Token::Chunk(" = ".to_string()));
                    result.extend(body_formatted);
                }
            }
            (None, Some(extern_kind)) => match extern_kind {
                FunctionExternKind::Builtin => result.push(Token::Chunk(" = extern".to_string())),
                FunctionExternKind::C(Some(header)) => {
                    result.push(Token::Chunk(format!(" = extern \"C\" (\"{}\")", header)))
                }
                FunctionExternKind::C(None) => result.push(Token::Chunk(" = extern \"C\"".to_string())),
            },
            (None, None) => {
                // Function signature only
            }
            (Some(_), Some(_)) => {
                // This shouldn't happen - function can't have both body and be extern
                result.push(Token::Chunk(" = extern".to_string()));
            }
        }

        result
    }
}

impl Format for Struct {
    fn format(&self) -> Vec<Token> {
        let mut result = Vec::new();

        if self.public {
            result.push(Token::Chunk("pub ".to_string()));
        }

        result.push(Token::Chunk("struct ".to_string()));
        result.extend(self.name.format());

        if let Some(type_params) = &self.typeParams {
            result.extend(type_params.format());
        }

        result.extend(format_block_2_items(&self.fields, &self.methods));

        result
    }
}

impl Format for Field {
    fn format(&self) -> Vec<Token> {
        let mut result = Vec::new();

        if self.public {
            result.push(Token::Chunk("pub ".to_string()));
        }

        result.extend(self.name.format());
        result.push(Token::Chunk(": ".to_string()));
        result.extend(self.ty.format());
        result.push(Token::Chunk(",".to_string()));
        result
    }
}

impl Format for Enum {
    fn format(&self) -> Vec<Token> {
        let mut result = Vec::new();

        if self.public {
            result.push(Token::Chunk("pub ".to_string()));
        }

        result.push(Token::Chunk("enum ".to_string()));
        result.extend(self.name.format());

        if let Some(type_params) = &self.typeParams {
            result.extend(type_params.format());
        }

        result.extend(format_block_2_items(&self.variants, &self.methods));

        result
    }
}

impl Format for Variant {
    fn format(&self) -> Vec<Token> {
        let mut result = Vec::new();
        result.extend(self.name.format());
        if !self.items.is_empty() {
            result.push(Token::Chunk("(".to_string()));
            result.extend(format_list(&self.items, Token::Chunk(", ".to_string())));
            result.push(Token::Chunk(")".to_string()));
        }
        result.push(Token::Chunk(",".to_string()));
        result
    }
}

// Trait and Instance formatting
impl Format for Trait {
    fn format(&self) -> Vec<Token> {
        let mut result = Vec::new();

        if self.public {
            result.push(Token::Chunk("pub ".to_string()));
        }

        result.push(Token::Chunk("trait".to_string()));
        if let Some(type_params) = &self.typeParams {
            result.extend(type_params.format());
        }

        result.push(Token::Chunk(" ".to_string()));
        result.extend(self.name.format());
        if !self.params.is_empty() {
            result.push(Token::Chunk("[".to_string()));
            result.extend(format_list(&self.params, Token::Chunk(", ".to_string())));
            result.push(Token::Chunk("]".to_string()));
        }

        result.extend(format_block_2_items(&self.associatedTypes, &self.methods));
        result
    }
}

impl Format for AssociatedTypeDeclaration {
    fn format(&self) -> Vec<Token> {
        let mut result = vec![Token::Chunk("type ".to_string())];
        result.extend(self.name.format());

        if !self.constraints.is_empty() {
            result.push(Token::Chunk(": ".to_string()));
            result.extend(format_list(&self.constraints, Token::Chunk(" + ".to_string())));
        }

        result
    }
}

impl Format for Instance {
    fn format(&self) -> Vec<Token> {
        let mut result = Vec::new();

        if self.public {
            result.push(Token::Chunk("pub ".to_string()));
        }

        result.push(Token::Chunk("instance".to_string()));
        if let Some(type_params) = &self.typeParams {
            result.extend(type_params.format());
        }

        result.push(Token::Chunk(" ".to_string()));
        // Optional instance name
        if let Some(name) = &self.name {
            result.extend(name.format());
            result.push(Token::Chunk(" ".to_string()));
        }

        result.extend(self.traitName.format());

        if !self.types.is_empty() {
            result.push(Token::Chunk("[".to_string()));
            result.extend(format_list(&self.types, Token::Chunk(", ".to_string())));
            result.push(Token::Chunk("]".to_string()));
        }

        result.extend(format_block_2_items(&self.associatedTypes, &self.methods));

        result
    }
}

impl Format for AssociatedType {
    fn format(&self) -> Vec<Token> {
        let mut result = vec![Token::Chunk("type ".to_string())];
        result.extend(self.name.format());
        result.push(Token::Chunk(" = ".to_string()));
        result.extend(self.ty.format());
        result
    }
}

// Effect formatting
impl Format for Effect {
    fn format(&self) -> Vec<Token> {
        let mut result = Vec::new();

        if self.public {
            result.push(Token::Chunk("pub ".to_string()));
        }

        result.push(Token::Chunk("effect ".to_string()));
        result.extend(self.name.format());

        if !self.methods.is_empty() {
            result.extend(format_block(&self.methods));
        }

        result
    }
}

// Implicit formatting
impl Format for Implicit {
    fn format(&self) -> Vec<Token> {
        let mut result = Vec::new();

        if self.public {
            result.push(Token::Chunk("pub ".to_string()));
        }

        result.push(Token::Chunk("implicit ".to_string()));

        if self.mutable {
            result.push(Token::Chunk("mut ".to_string()));
        }

        result.extend(self.name.format());
        result.push(Token::Chunk(": ".to_string()));
        result.extend(self.ty.format());

        result
    }
}

// Module and Import formatting
impl Format for Import {
    fn format(&self) -> Vec<Token> {
        let mut result = vec![Token::Chunk("import ".to_string())];
        result.extend(self.moduleName.format());

        if let Some(alias) = &self.alias {
            result.push(Token::Chunk(" as ".to_string()));
            result.extend(alias.format());
        }

        if self.implicitImport {
            result.push(Token::Chunk(" implicit".to_string()));
        }

        result
    }
}

impl Format for Derive {
    fn format(&self) -> Vec<Token> {
        self.name.format()
    }
}

impl Format for ModuleItem {
    fn format(&self) -> Vec<Token> {
        match self {
            ModuleItem::Struct(s) => s.format(),
            ModuleItem::Enum(e) => e.format(),
            ModuleItem::Function(f) => f.format(),
            ModuleItem::Import(i) => i.format(),
            ModuleItem::Effect(e) => e.format(),
            ModuleItem::Implicit(i) => i.format(),
            ModuleItem::Trait(t) => t.format(),
            ModuleItem::Instance(i) => i.format(),
        }
    }
}

impl Format for Module {
    fn format(&self) -> Vec<Token> {
        let mut result = vec![Token::Chunk("module ".to_string())];
        result.extend(self.name.format());
        result.extend(format_block(&self.items));
        result
    }
}
