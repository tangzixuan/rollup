use napi::bindgen_prelude::*;
use swc_common::Span;
use swc_ecma_ast::{ArrowExpr, BindingIdent, BlockStmt, BlockStmtOrExpr, Callee, CallExpr, Decl, ExportDecl, ExportNamedSpecifier, ExportSpecifier, Expr, ExprOrSpread, ExprStmt, Ident, ImportDecl, ImportNamedSpecifier, ImportSpecifier, Lit, MemberExpr, MemberProp, Module, ModuleDecl, ModuleExportName, ModuleItem, NamedExport, Number, Pat, PrivateName, Program, Stmt, Str, VarDecl, VarDeclarator, VarDeclKind};

// These need to reflect the order in the JavaScript decoder
const TYPE_MODULE: [u8; 4] = 0u32.to_ne_bytes();
const TYPE_EXPRESSION_STATEMENT: [u8; 4] = 1u32.to_ne_bytes();
const TYPE_NUMBER: [u8; 4] = 2u32.to_ne_bytes();
const TYPE_EXPORT_DECLARATION: [u8; 4] = 3u32.to_ne_bytes();
const TYPE_NAMED_EXPORT: [u8; 4] = 4u32.to_ne_bytes();
const TYPE_VARIABLE_DECLARATION: [u8; 4] = 5u32.to_ne_bytes();
const TYPE_VARIABLE_DECLARATOR: [u8; 4] = 6u32.to_ne_bytes();
const TYPE_IDENTIFIER: [u8; 4] = 7u32.to_ne_bytes();
const TYPE_STRING: [u8; 4] = 8u32.to_ne_bytes();
const TYPE_EXPORT_NAMED_SPECIFIER: [u8; 4] = 9u32.to_ne_bytes();
const TYPE_IMPORT_DECLARATION: [u8; 4] = 10u32.to_ne_bytes();
const TYPE_IMPORT_NAMED_SPECIFIER: [u8; 4] = 11u32.to_ne_bytes();
const TYPE_CALL_EXPRESSION: [u8; 4] = 12u32.to_ne_bytes();
const TYPE_ARROW_EXPRESSION: [u8; 4] = 13u32.to_ne_bytes();
const TYPE_BLOCK_STATEMENT: [u8; 4] = 14u32.to_ne_bytes();
const TYPE_SPREAD: [u8; 4] = 15u32.to_ne_bytes();
const TYPE_MEMBER_EXPRESSION: [u8; 4] = 16u32.to_ne_bytes();
const TYPE_PRIVATE_NAME: [u8; 4] = 17u32.to_ne_bytes();

// other constants
const DECLARATION_KIND_VAR: [u8; 4] = 0u32.to_ne_bytes();
const DECLARATION_KIND_LET: [u8; 4] = 1u32.to_ne_bytes();
const DECLARATION_KIND_CONST: [u8; 4] = 2u32.to_ne_bytes();

pub struct AstConverter {
    buffer: Vec<u8>,
    code_length: u32,
}

impl AstConverter {
    pub fn new(code_length: u32) -> Self {
        Self {
            buffer: Vec::new(),
            code_length,
        }
    }

    pub fn convert_ast_to_buffer(mut self, node: &Program) -> Buffer {
        self.convert_program(node);
        self.buffer.into()
    }

    // === enums
    fn convert_program(&mut self, node: &Program) {
        match node {
            Program::Module(module) => self.convert_module_program(module),
            _ => {
                dbg!(node);
                unimplemented!("Cannot convert AST");
            }
        }
    }

    fn convert_module_item(&mut self, module_item: &ModuleItem) {
        match module_item {
            ModuleItem::Stmt(statement) => self.convert_statement(statement),
            ModuleItem::ModuleDecl(module_declaration) => {
                self.convert_module_declaration(module_declaration);
            }
        }
    }

    fn convert_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Expr(expression_statement) => self.convert_expression_statement(expression_statement),
            Stmt::Decl(declaration) => self.convert_declaration(declaration),
            _ => {
                dbg!(statement);
                todo!("Cannot convert Statement");
            }
        }
    }

    fn convert_expression(&mut self, expression: &Expr) {
        match expression {
            Expr::Lit(literal) => self.convert_literal(literal),
            Expr::Call(call_expression) => self.convert_call_expression(call_expression),
            Expr::Ident(identifier) => self.convert_identifier(identifier),
            Expr::Arrow(arrow_expression) => self.convert_arrow_expression(arrow_expression),
            Expr::Member(member_expression) => self.convert_member_expression(member_expression),
            _ => {
                dbg!(expression);
                todo!("Cannot convert Expression");
            }
        }
    }

    fn convert_literal(&mut self, literal: &Lit) {
        match literal {
            Lit::Num(number_literal) => self.convert_literal_number(number_literal),
            Lit::Str(string_literal) => self.convert_literal_string(string_literal),
            _ => {
                dbg!(literal);
                todo!("Cannot convert Literal");
            }
        }
    }

    fn convert_module_declaration(&mut self, module_declaration: &ModuleDecl) {
        match module_declaration {
            ModuleDecl::ExportDecl(export_declaration) => self.convert_export_declaration(export_declaration),
            ModuleDecl::ExportNamed(export_named) => self.convert_export_named_declaration(export_named),
            ModuleDecl::Import(import_declaration) => self.convert_import_declaration(import_declaration),
            _ => {
                dbg!(module_declaration);
                todo!("Cannot convert ModuleDeclaration");
            }
        }
    }

    fn convert_declaration(&mut self, declaration: &Decl) {
        match declaration {
            Decl::Var(variable_declaration) => self.convert_variable_declaration(variable_declaration),
            _ => {
                dbg!(declaration);
                todo!("Cannot convert Declaration");
            }
        }
    }

    fn convert_pattern(&mut self, pattern: &Pat) {
        match pattern {
            Pat::Ident(binding_identifier) => self.convert_binding_identifier(binding_identifier),
            _ => {
                dbg!(pattern);
                todo!("Cannot convert Pattern");
            }
        }
    }

    fn convert_binding_identifier(&mut self, binding_identifier: &BindingIdent) {
        self.convert_identifier(&binding_identifier.id);
    }

    fn convert_export_specifier(&mut self, export_specifier: &ExportSpecifier) {
        match export_specifier {
            ExportSpecifier::Named(export_named_specifier) => self.convert_export_named_specifier(export_named_specifier),
            _ => {
                dbg!(export_specifier);
                todo!("Cannot convert ExportSpecifier");
            }
        }
    }

    fn convert_module_export_name(&mut self, module_export_name: &ModuleExportName) {
        match module_export_name {
            ModuleExportName::Ident(identifier) => self.convert_identifier(identifier),
            _ => {
                dbg!(module_export_name);
                todo!("Cannot convert ModuleExportName");
            }
        }
    }

    fn convert_import_specifier(&mut self, import_specifier: &ImportSpecifier) {
        match import_specifier {
            ImportSpecifier::Named(import_named_specifier) => self.convert_import_named_specifier(import_named_specifier),
            _ => {
                dbg!(import_specifier);
                todo!("Cannot convert ImportSpecifier");
            }
        }
    }

    fn convert_callee(&mut self, callee: &Callee) {
        match callee {
            Callee::Expr(expr) => self.convert_expression(expr),
            _ => {
                dbg!(callee);
                todo!("Cannot convert Callee");
            }
        }
    }

    // === nodes
    fn convert_module_program(&mut self, module: &Module) {
        self.add_type_and_positions(&TYPE_MODULE, &module.span);
        // acorn uses the file length instead of the end of the last statement
        let reference_position = self.buffer.len() - 4;
        self.buffer[reference_position..reference_position + 4].copy_from_slice(&self.code_length.to_ne_bytes());
        // body
        self.convert_item_list(&module.body, |ast_converter, module_item| ast_converter.convert_module_item(module_item));
    }

    fn convert_expression_statement(&mut self, expression_statement: &ExprStmt) {
        self.add_type_and_positions(&TYPE_EXPRESSION_STATEMENT, &expression_statement.span);
        // expression
        self.convert_expression(&expression_statement.expr);
    }

    fn convert_export_declaration(&mut self, export_declaration: &ExportDecl) {
        self.add_type_and_positions(&TYPE_EXPORT_DECLARATION, &export_declaration.span);
        // declaration
        self.convert_declaration(&export_declaration.decl);
    }

    fn convert_export_named_declaration(&mut self, export_named_declaration: &NamedExport) {
        self.add_type_and_positions(&TYPE_NAMED_EXPORT, &export_named_declaration.span);
        // reserve for src
        let reference_position = self.reserve_reference_positions(1);
        // specifiers
        self.convert_item_list(&export_named_declaration.specifiers, |ast_converter, specifier| ast_converter.convert_export_specifier(specifier));
        // src
        export_named_declaration.src.as_ref().map(|src| {
            self.update_reference_position(reference_position);
            self.convert_literal_string(&*src);
        });
    }

    fn convert_literal_number(&mut self, literal: &Number) {
        self.add_type_and_positions(&TYPE_NUMBER, &literal.span);
        // reserve for raw
        let reference_position = self.reserve_reference_positions(1);
        // value, needs to be little endian as we are reading via a DataView
        self.buffer.extend_from_slice(&literal.value.to_le_bytes());
        // raw
        literal.raw.as_ref().map(|raw| {
            self.update_reference_position(reference_position);
            self.convert_string(&*raw);
        });
    }

    fn convert_literal_string(&mut self, literal: &Str) {
        self.add_type_and_positions(&TYPE_STRING, &literal.span);
        // reserve for raw
        let reference_position = self.reserve_reference_positions(1);
        // value
        self.convert_string(&literal.value);
        // raw
        literal.raw.as_ref().map(|raw| {
            self.update_reference_position(reference_position);
            self.convert_string(&*raw);
        });
    }

    fn convert_variable_declaration(&mut self, variable_declaration: &VarDecl) {
        self.add_type_and_positions(&TYPE_VARIABLE_DECLARATION, &variable_declaration.span);
        self.buffer.extend_from_slice(match variable_declaration.kind {
            VarDeclKind::Var => &DECLARATION_KIND_VAR,
            VarDeclKind::Let => &DECLARATION_KIND_LET,
            VarDeclKind::Const => &DECLARATION_KIND_CONST,
        });
        self.convert_item_list(&variable_declaration.decls, |ast_converter, variable_declarator| ast_converter.convert_variable_declarator(variable_declarator));
    }

    fn convert_variable_declarator(&mut self, variable_declarator: &VarDeclarator) {
        self.add_type_and_positions(&TYPE_VARIABLE_DECLARATOR, &variable_declarator.span);
        // reserve for init
        let reference_position = self.reserve_reference_positions(1);
        // id
        self.convert_pattern(&variable_declarator.name);
        // init
        variable_declarator.init.as_ref().map(|init| {
            self.update_reference_position(reference_position);
            self.convert_expression(&init);
        });
    }

    fn convert_identifier(&mut self, identifier: &Ident) {
        self.add_type_and_positions(&TYPE_IDENTIFIER, &identifier.span);
        // name
        self.convert_string(&identifier.sym);
    }

    fn convert_export_named_specifier(&mut self, export_named_specifier: &ExportNamedSpecifier) {
        self.add_type_and_positions(&TYPE_EXPORT_NAMED_SPECIFIER, &export_named_specifier.span);
        // reserve for exported
        let reference_position = self.reserve_reference_positions(1);
        // local
        self.convert_module_export_name(&export_named_specifier.orig);
        // exported
        export_named_specifier.exported.as_ref().map(|exported| {
            self.update_reference_position(reference_position);
            self.convert_module_export_name(&exported);
        });
    }

    fn convert_import_declaration(&mut self, import_declaration: &ImportDecl) {
        self.add_type_and_positions(&TYPE_IMPORT_DECLARATION, &import_declaration.span);
        // reserve for src
        let reference_position = self.reserve_reference_positions(1);
        // specifiers
        self.convert_item_list(&import_declaration.specifiers, |ast_converter, import_specifier| ast_converter.convert_import_specifier(import_specifier));
        // src
        self.update_reference_position(reference_position);
        self.convert_literal_string(&*import_declaration.src);
    }

    fn convert_call_expression(&mut self, call_expression: &CallExpr) {
        self.add_type_and_positions(&TYPE_CALL_EXPRESSION, &call_expression.span);
        // reserve for callee
        let reference_position = self.reserve_reference_positions(1);
        // arguments
        self.convert_item_list(&call_expression.args, |ast_converter, argument| ast_converter.convert_expression_or_spread(argument));
        // callee
        self.update_reference_position(reference_position);
        self.convert_callee(&call_expression.callee);
    }

    fn convert_import_named_specifier(&mut self, import_named_specifier: &ImportNamedSpecifier) {
        self.add_type_and_positions(&TYPE_IMPORT_NAMED_SPECIFIER, &import_named_specifier.span);
        // reserve for imported
        let reference_position = self.reserve_reference_positions(1);
        // local
        self.convert_identifier(&import_named_specifier.local);
        // imported
        import_named_specifier.imported.as_ref().map(|imported| {
            self.update_reference_position(reference_position);
            self.convert_module_export_name(&imported);
        });
    }

    fn convert_arrow_expression(&mut self, arrow_expression: &ArrowExpr) {
        self.add_type_and_positions(&TYPE_ARROW_EXPRESSION, &arrow_expression.span);
        // async
        self.convert_boolean(arrow_expression.is_async);
        // generator
        self.convert_boolean(arrow_expression.is_generator);
        // reserve for params
        let reference_position = self.reserve_reference_positions(1);
        match &*arrow_expression.body {
            BlockStmtOrExpr::BlockStmt(block_statement) => {
                // expression
                self.convert_boolean(false);
                // body
                self.convert_block_statement(block_statement)
            },
            BlockStmtOrExpr::Expr(expression) => {
                // expression
                self.convert_boolean(true);
                // body
                self.convert_expression(expression)
            },
        }
        // params
        self.update_reference_position(reference_position);
        self.convert_item_list(&arrow_expression.params, |ast_converter, param| ast_converter.convert_pattern(param));
    }

    fn convert_block_statement(&mut self, block_statement: &BlockStmt) {
        self.add_type_and_positions(&TYPE_BLOCK_STATEMENT, &block_statement.span);
        // body
        self.convert_item_list(&block_statement.stmts, |ast_converter, statement| ast_converter.convert_statement(statement));
    }


    fn convert_expression_or_spread(&mut self, expression_or_spread: &ExprOrSpread) {
        match expression_or_spread.spread {
            Some(spread_span) => {
                self.add_type_and_positions(&TYPE_SPREAD, &spread_span);
                // we need to set the end position to that of the expression
                let reference_position = self.buffer.len();
                self.convert_expression(&expression_or_spread.expr);
                let expression_end: [u8; 4] = self.buffer[reference_position + 4..reference_position + 8].try_into().unwrap();
                self.buffer[reference_position - 4..reference_position].copy_from_slice(&expression_end);
            }
            None => self.convert_expression(&expression_or_spread.expr),
        }
    }

    fn convert_member_expression(&mut self, member_expression: &MemberExpr) {
        self.add_type_and_positions(&TYPE_MEMBER_EXPRESSION, &member_expression.span);
        // reserve object
        let reference_position = self.reserve_reference_positions(1);
        match &member_expression.prop {
            MemberProp::Ident(ident) => {
                // computed
                self.convert_boolean(false);
                // property
                self.convert_identifier(&ident)
            }
            MemberProp::Computed(computed) => {
                // computed
                self.convert_boolean(true);
                // property
                self.convert_expression(&computed.expr)
            }
            MemberProp::PrivateName(private_name) => {
                // computed
                self.convert_boolean(false);
                // property
                self.convert_private_name(&private_name)
            }
        }
        // object
        self.update_reference_position(reference_position);
        self.convert_expression(&member_expression.obj);
    }

    fn convert_private_name(&mut self, private_name: &&PrivateName) {
        self.add_type_and_positions(&TYPE_PRIVATE_NAME, &private_name.span);
        // id
        self.convert_identifier(&private_name.id);
    }

    // === helpers
    fn add_type_and_positions(&mut self, node_type: &[u8; 4], span: &Span) {
        // type
        self.buffer.extend_from_slice(node_type);
        // start
        self.buffer.extend_from_slice(&(span.lo.0 - 1).to_ne_bytes());
        // end
        self.buffer.extend_from_slice(&(span.hi.0 - 1).to_ne_bytes());
    }

    fn convert_item_list<T, F>(&mut self, item_list: &Vec<T>, convert_item: F)
        where F: Fn(&mut AstConverter, &T)
    {
        // store number of items in first position
        self.buffer.extend_from_slice(&(item_list.len() as u32).to_ne_bytes());
        let mut reference_position = self.buffer.len();
        // make room for the reference positions of the items
        self.buffer.resize(self.buffer.len() + item_list.len() * 4, 0);
        for item in item_list {
            let insert_position = (self.buffer.len() as u32) >> 2;
            self.buffer[reference_position..reference_position + 4].copy_from_slice(&insert_position.to_ne_bytes());
            reference_position += 4;
            convert_item(self, item);
        }
    }

    // TODO Lukas deduplicate strings and see if we can easily compare atoms
    fn convert_string(&mut self, string: &str) {
        let length = string.len();
        let additional_length = ((length + 3) & !3) - length;
        self.buffer.extend_from_slice(&(length as u32).to_ne_bytes());
        self.buffer.extend_from_slice(string.as_bytes());
        self.buffer.resize(self.buffer.len() + additional_length, 0);
    }

    fn convert_boolean(&mut self, boolean: bool) {
        self.buffer.extend_from_slice(&(if boolean { 1u32 } else { 0u32 }).to_ne_bytes());
    }

    fn reserve_reference_positions(&mut self, item_count: usize) -> usize {
        let reference_position = self.buffer.len();
        self.buffer.resize(reference_position + (item_count << 2), 0);
        reference_position
    }

    fn update_reference_position(&mut self, reference_position: usize) -> usize {
        let insert_position = (self.buffer.len() as u32) >> 2;
        self.buffer[reference_position..reference_position + 4].copy_from_slice(&insert_position.to_ne_bytes());
        reference_position + 4
    }
}
