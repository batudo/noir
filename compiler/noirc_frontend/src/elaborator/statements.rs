use noirc_errors::{Location, Span};

use crate::{
    ast::{AssignStatement, ConstrainStatement, LValue},
    hir::{
        resolution::errors::ResolverError,
        type_check::{Source, TypeCheckError},
    },
    hir_def::{
        expr::HirIdent,
        stmt::{
            HirAssignStatement, HirConstrainStatement, HirForStatement, HirLValue, HirLetStatement,
        },
    },
    macros_api::{
        ForLoopStatement, ForRange, HirStatement, LetStatement, Path, Statement, StatementKind,
    },
    node_interner::{DefinitionId, DefinitionKind, GlobalId, StmtId},
    Type,
};

use super::{lints, Elaborator};

impl<'context> Elaborator<'context> {
    fn elaborate_statement_value(&mut self, statement: Statement) -> (HirStatement, Type) {
        match statement.kind {
            StatementKind::Let(let_stmt) => self.elaborate_local_let(let_stmt),
            StatementKind::Constrain(constrain) => self.elaborate_constrain(constrain),
            StatementKind::Assign(assign) => self.elaborate_assign(assign),
            StatementKind::For(for_stmt) => self.elaborate_for(for_stmt),
            StatementKind::Break => self.elaborate_jump(true, statement.span),
            StatementKind::Continue => self.elaborate_jump(false, statement.span),
            StatementKind::Comptime(statement) => self.elaborate_comptime_statement(*statement),
            StatementKind::Expression(expr) => {
                let (expr, typ) = self.elaborate_expression(expr);
                (HirStatement::Expression(expr), typ)
            }
            StatementKind::Semi(expr) => {
                let (expr, _typ) = self.elaborate_expression(expr);
                (HirStatement::Semi(expr), Type::Unit)
            }
            StatementKind::Interned(id) => {
                let kind = self.interner.get_statement_kind(id);
                let statement = Statement { kind: kind.clone(), span: statement.span };
                self.elaborate_statement_value(statement)
            }
            StatementKind::Error => (HirStatement::Error, Type::Error),
        }
    }

    pub(super) fn elaborate_statement(&mut self, statement: Statement) -> (StmtId, Type) {
        let span = statement.span;
        let (hir_statement, typ) = self.elaborate_statement_value(statement);
        let id = self.interner.push_stmt(hir_statement);
        self.interner.push_stmt_location(id, span, self.file);
        (id, typ)
    }

    pub(super) fn elaborate_local_let(&mut self, let_stmt: LetStatement) -> (HirStatement, Type) {
        self.elaborate_let(let_stmt, None)
    }

    /// Elaborate a local or global let statement.
    /// If this is a global let, the DefinitionId of the global is specified so that
    /// elaborate_pattern can create a Global definition kind with the correct ID
    /// instead of a local one with a fresh ID.
    pub(super) fn elaborate_let(
        &mut self,
        let_stmt: LetStatement,
        global_id: Option<GlobalId>,
    ) -> (HirStatement, Type) {
        let expr_span = let_stmt.expression.span;
        let (expression, expr_type) = self.elaborate_expression(let_stmt.expression);
        let annotated_type = self.resolve_inferred_type(let_stmt.r#type);

        let definition = match global_id {
            None => DefinitionKind::Local(Some(expression)),
            Some(id) => DefinitionKind::Global(id),
        };

        // Now check if LHS is the same type as the RHS
        // Importantly, we do not coerce any types implicitly
        self.unify_with_coercions(&expr_type, &annotated_type, expression, expr_span, || {
            TypeCheckError::TypeMismatch {
                expected_typ: annotated_type.to_string(),
                expr_typ: expr_type.to_string(),
                expr_span,
            }
        });

        if annotated_type.is_integer() {
            let errors = lints::overflowing_int(self.interner, &expression, &annotated_type);
            for error in errors {
                self.push_err(error);
            }
        }

        let r#type = annotated_type;
        let pattern = self.elaborate_pattern_and_store_ids(
            let_stmt.pattern,
            r#type.clone(),
            definition,
            &mut Vec::new(),
            global_id,
        );

        let attributes = let_stmt.attributes;
        let comptime = let_stmt.comptime;
        let let_ = HirLetStatement { pattern, r#type, expression, attributes, comptime };
        (HirStatement::Let(let_), Type::Unit)
    }

    pub(super) fn elaborate_constrain(&mut self, stmt: ConstrainStatement) -> (HirStatement, Type) {
        let expr_span = stmt.0.span;
        let (expr_id, expr_type) = self.elaborate_expression(stmt.0);

        // Must type check the assertion message expression so that we instantiate bindings
        let msg = stmt.1.map(|assert_msg_expr| self.elaborate_expression(assert_msg_expr).0);

        self.unify(&expr_type, &Type::Bool, || TypeCheckError::TypeMismatch {
            expr_typ: expr_type.to_string(),
            expected_typ: Type::Bool.to_string(),
            expr_span,
        });

        (HirStatement::Constrain(HirConstrainStatement(expr_id, self.file, msg)), Type::Unit)
    }

    pub(super) fn elaborate_assign(&mut self, assign: AssignStatement) -> (HirStatement, Type) {
        let span = assign.expression.span;
        let (expression, expr_type) = self.elaborate_expression(assign.expression);
        let (lvalue, lvalue_type, mutable) = self.elaborate_lvalue(assign.lvalue, span);

        if !mutable {
            let (name, span) = self.get_lvalue_name_and_span(&lvalue);
            self.push_err(TypeCheckError::VariableMustBeMutable { name, span });
        }

        self.unify_with_coercions(&expr_type, &lvalue_type, expression, span, || {
            TypeCheckError::TypeMismatchWithSource {
                actual: expr_type.clone(),
                expected: lvalue_type.clone(),
                span,
                source: Source::Assignment,
            }
        });

        let stmt = HirAssignStatement { lvalue, expression };
        (HirStatement::Assign(stmt), Type::Unit)
    }

    pub(super) fn elaborate_for(&mut self, for_loop: ForLoopStatement) -> (HirStatement, Type) {
        let (start, end) = match for_loop.range {
            ForRange::Range(start, end) => (start, end),
            ForRange::Array(_) => {
                let for_stmt =
                    for_loop.range.into_for(for_loop.identifier, for_loop.block, for_loop.span);

                return self.elaborate_statement_value(for_stmt);
            }
        };

        let start_span = start.span;
        let end_span = end.span;

        let (start_range, start_range_type) = self.elaborate_expression(start);
        let (end_range, end_range_type) = self.elaborate_expression(end);
        let (identifier, block) = (for_loop.identifier, for_loop.block);

        self.nested_loops += 1;
        self.push_scope();

        // TODO: For loop variables are currently mutable by default since we haven't
        //       yet implemented syntax for them to be optionally mutable.
        let kind = DefinitionKind::Local(None);
        let identifier = self.add_variable_decl(identifier, false, true, kind);

        // Check that start range and end range have the same types
        let range_span = start_span.merge(end_span);
        self.unify(&start_range_type, &end_range_type, || TypeCheckError::TypeMismatch {
            expected_typ: start_range_type.to_string(),
            expr_typ: end_range_type.to_string(),
            expr_span: range_span,
        });

        let expected_type = self.polymorphic_integer();

        self.unify(&start_range_type, &expected_type, || TypeCheckError::TypeCannotBeUsed {
            typ: start_range_type.clone(),
            place: "for loop",
            span: range_span,
        });

        self.interner.push_definition_type(identifier.id, start_range_type);

        let (block, _block_type) = self.elaborate_expression(block);

        self.pop_scope();
        self.nested_loops -= 1;

        let statement =
            HirStatement::For(HirForStatement { start_range, end_range, block, identifier });

        (statement, Type::Unit)
    }

    fn elaborate_jump(&mut self, is_break: bool, span: noirc_errors::Span) -> (HirStatement, Type) {
        let in_constrained_function = self.in_constrained_function();

        if in_constrained_function {
            self.push_err(ResolverError::JumpInConstrainedFn { is_break, span });
        }
        if self.nested_loops == 0 {
            self.push_err(ResolverError::JumpOutsideLoop { is_break, span });
        }

        let expr = if is_break { HirStatement::Break } else { HirStatement::Continue };
        (expr, self.interner.next_type_variable())
    }

    fn get_lvalue_name_and_span(&self, lvalue: &HirLValue) -> (String, Span) {
        match lvalue {
            HirLValue::Ident(name, _) => {
                let span = name.location.span;

                if let Some(definition) = self.interner.try_definition(name.id) {
                    (definition.name.clone(), span)
                } else {
                    ("(undeclared variable)".into(), span)
                }
            }
            HirLValue::MemberAccess { object, .. } => self.get_lvalue_name_and_span(object),
            HirLValue::Index { array, .. } => self.get_lvalue_name_and_span(array),
            HirLValue::Dereference { lvalue, .. } => self.get_lvalue_name_and_span(lvalue),
        }
    }

    fn elaborate_lvalue(&mut self, lvalue: LValue, assign_span: Span) -> (HirLValue, Type, bool) {
        match lvalue {
            LValue::Ident(ident) => {
                let mut mutable = true;
                let span = ident.span();
                let path = Path::from_single(ident.0.contents, span);
                let (ident, scope_index) = self.get_ident_from_path(path);
                self.resolve_local_variable(ident.clone(), scope_index);

                let typ = if ident.id == DefinitionId::dummy_id() {
                    Type::Error
                } else {
                    if let Some(definition) = self.interner.try_definition(ident.id) {
                        mutable = definition.mutable;
                    }

                    let typ = self.interner.definition_type(ident.id).instantiate(self.interner).0;
                    typ.follow_bindings()
                };

                let reference_location = Location::new(span, self.file);
                self.interner.add_local_reference(ident.id, reference_location);

                (HirLValue::Ident(ident.clone(), typ.clone()), typ, mutable)
            }
            LValue::MemberAccess { object, field_name, span } => {
                let (object, lhs_type, mut mutable) = self.elaborate_lvalue(*object, assign_span);
                let mut object = Box::new(object);
                let field_name = field_name.clone();

                let object_ref = &mut object;
                let mutable_ref = &mut mutable;
                let location = Location::new(span, self.file);

                let dereference_lhs = move |_: &mut Self, _, element_type| {
                    // We must create a temporary value first to move out of object_ref before
                    // we eventually reassign to it.
                    let id = DefinitionId::dummy_id();
                    let ident = HirIdent::non_trait_method(id, location);
                    let tmp_value = HirLValue::Ident(ident, Type::Error);

                    let lvalue = std::mem::replace(object_ref, Box::new(tmp_value));
                    *object_ref =
                        Box::new(HirLValue::Dereference { lvalue, element_type, location });
                    *mutable_ref = true;
                };

                let name = &field_name.0.contents;
                let (object_type, field_index) = self
                    .check_field_access(&lhs_type, name, field_name.span(), Some(dereference_lhs))
                    .unwrap_or((Type::Error, 0));

                let field_index = Some(field_index);
                let typ = object_type.clone();
                let lvalue =
                    HirLValue::MemberAccess { object, field_name, field_index, typ, location };
                (lvalue, object_type, mutable)
            }
            LValue::Index { array, index, span } => {
                let expr_span = index.span;
                let (index, index_type) = self.elaborate_expression(index);
                let location = Location::new(span, self.file);

                let expected = self.polymorphic_integer_or_field();
                self.unify(&index_type, &expected, || TypeCheckError::TypeMismatch {
                    expected_typ: "an integer".to_owned(),
                    expr_typ: index_type.to_string(),
                    expr_span,
                });

                let (mut lvalue, mut lvalue_type, mut mutable) =
                    self.elaborate_lvalue(*array, assign_span);

                // Before we check that the lvalue is an array, try to dereference it as many times
                // as needed to unwrap any &mut wrappers.
                while let Type::MutableReference(element) = lvalue_type.follow_bindings() {
                    let element_type = element.as_ref().clone();
                    lvalue =
                        HirLValue::Dereference { lvalue: Box::new(lvalue), element_type, location };
                    lvalue_type = *element;
                    // We know this value to be mutable now since we found an `&mut`
                    mutable = true;
                }

                let typ = match lvalue_type.follow_bindings() {
                    Type::Array(_, elem_type) => *elem_type,
                    Type::Slice(elem_type) => *elem_type,
                    Type::Error => Type::Error,
                    Type::String(_) => {
                        let (_lvalue_name, lvalue_span) = self.get_lvalue_name_and_span(&lvalue);
                        self.push_err(TypeCheckError::StringIndexAssign { span: lvalue_span });
                        Type::Error
                    }
                    other => {
                        // TODO: Need a better span here
                        self.push_err(TypeCheckError::TypeMismatch {
                            expected_typ: "array".to_string(),
                            expr_typ: other.to_string(),
                            expr_span: assign_span,
                        });
                        Type::Error
                    }
                };

                let array = Box::new(lvalue);
                let array_type = typ.clone();
                (HirLValue::Index { array, index, typ, location }, array_type, mutable)
            }
            LValue::Dereference(lvalue, span) => {
                let (lvalue, reference_type, _) = self.elaborate_lvalue(*lvalue, assign_span);
                let lvalue = Box::new(lvalue);
                let location = Location::new(span, self.file);

                let element_type = Type::type_variable(self.interner.next_type_variable_id());
                let expected_type = Type::MutableReference(Box::new(element_type.clone()));

                self.unify(&reference_type, &expected_type, || TypeCheckError::TypeMismatch {
                    expected_typ: expected_type.to_string(),
                    expr_typ: reference_type.to_string(),
                    expr_span: assign_span,
                });

                // Dereferences are always mutable since we already type checked against a &mut T
                let typ = element_type.clone();
                let lvalue = HirLValue::Dereference { lvalue, element_type, location };
                (lvalue, typ, true)
            }
            LValue::Interned(id, span) => {
                let lvalue = self.interner.get_lvalue(id, span).clone();
                self.elaborate_lvalue(lvalue, assign_span)
            }
        }
    }

    /// Type checks a field access, adding dereference operators as necessary
    pub(super) fn check_field_access(
        &mut self,
        lhs_type: &Type,
        field_name: &str,
        span: Span,
        dereference_lhs: Option<impl FnMut(&mut Self, Type, Type)>,
    ) -> Option<(Type, usize)> {
        let lhs_type = lhs_type.follow_bindings();

        match &lhs_type {
            Type::Struct(s, args) => {
                let s = s.borrow();
                if let Some((field, index)) = s.get_field(field_name, args) {
                    let reference_location = Location::new(span, self.file);
                    self.interner.add_struct_member_reference(s.id, index, reference_location);

                    return Some((field, index));
                }
            }
            Type::Tuple(elements) => {
                if let Ok(index) = field_name.parse::<usize>() {
                    let length = elements.len();
                    if index < length {
                        return Some((elements[index].clone(), index));
                    } else {
                        self.push_err(TypeCheckError::TupleIndexOutOfBounds {
                            index,
                            lhs_type,
                            length,
                            span,
                        });
                        return None;
                    }
                }
            }
            // If the lhs is a mutable reference we automatically transform
            // lhs.field into (*lhs).field
            Type::MutableReference(element) => {
                if let Some(mut dereference_lhs) = dereference_lhs {
                    dereference_lhs(self, lhs_type.clone(), element.as_ref().clone());
                    return self.check_field_access(
                        element,
                        field_name,
                        span,
                        Some(dereference_lhs),
                    );
                } else {
                    let (element, index) =
                        self.check_field_access(element, field_name, span, dereference_lhs)?;
                    return Some((Type::MutableReference(Box::new(element)), index));
                }
            }
            _ => (),
        }

        // If we get here the type has no field named 'access.rhs'.
        // Now we specialize the error message based on whether we know the object type in question yet.
        if let Type::TypeVariable(..) = &lhs_type {
            self.push_err(TypeCheckError::TypeAnnotationsNeededForFieldAccess { span });
        } else if lhs_type != Type::Error {
            self.push_err(TypeCheckError::AccessUnknownMember {
                lhs_type,
                field_name: field_name.to_string(),
                span,
            });
        }

        None
    }

    fn elaborate_comptime_statement(&mut self, statement: Statement) -> (HirStatement, Type) {
        // We have to push a new FunctionContext so that we can resolve any constraints
        // in this comptime block early before the function as a whole finishes elaborating.
        // Otherwise the interpreter below may find expressions for which the underlying trait
        // call is not yet solved for.
        self.function_context.push(Default::default());
        let span = statement.span;
        let (hir_statement, _typ) = self.elaborate_statement(statement);
        self.check_and_pop_function_context();
        let mut interpreter = self.setup_interpreter();
        let value = interpreter.evaluate_statement(hir_statement);
        let (expr, typ) = self.inline_comptime_value(value, span);

        let location = self.interner.id_location(hir_statement);
        self.debug_comptime(location, |interner| expr.to_display_ast(interner).kind);

        (HirStatement::Expression(expr), typ)
    }
}
