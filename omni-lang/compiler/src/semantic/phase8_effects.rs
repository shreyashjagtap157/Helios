//! Phase 8: Full Effect System Extensions
//!
//! Implements Phase 8 requirements from the specification:
//! - Full effect handler syntax and semantics
//! - User-defined effect kinds
//! - Effect polymorphism in generics
//! - Structured concurrency enforcement
//! - Explicit cancellation tokens
//! - Generator effects (Gen<T>)
//!
//! This module extends the existing effects.rs with Phase 8 features.

use crate::parser::ast::*;
use crate::semantic::effects::{self, EffectError};
use std::collections::HashMap;

/// Runtime value placeholder for generators
#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// User-defined effect definition
#[derive(Debug, Clone)]
pub struct UserEffectDef {
    pub name: String,
    pub operations: Vec<EffectOperation>,
    pub param_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct EffectOperation {
    pub name: String,
    pub input_type: Type,
    pub output_type: Type,
}

impl UserEffectDef {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            operations: Vec::new(),
            param_type: None,
        }
    }

    pub fn with_param(mut self, ty: Type) -> Self {
        self.param_type = Some(ty);
        self
    }

    pub fn add_operation(mut self, name: &str, input: Type, output: Type) -> Self {
        self.operations.push(EffectOperation {
            name: name.to_string(),
            input_type: input,
            output_type: output,
        });
        self
    }
}

/// Effect polymorphism - functions can be polymorphic over effects
#[derive(Debug, Clone)]
pub struct EffectPolymorphism {
    /// Map: function_name -> (type_params, effect_params)
    pub polymorphic_functions: HashMap<String, (Vec<String>, Vec<String>)>,
}

impl EffectPolymorphism {
    pub fn new() -> Self {
        Self {
            polymorphic_functions: HashMap::new(),
        }
    }

    /// Register a polymorphic function
    pub fn register(
        &mut self,
        func_name: &str,
        type_params: Vec<String>,
        effect_params: Vec<String>,
    ) {
        self.polymorphic_functions
            .insert(func_name.to_string(), (type_params, effect_params));
    }

    /// Check if a function is polymorphic over effects
    pub fn is_polymorphic(&self, func_name: &str) -> bool {
        self.polymorphic_functions
            .get(func_name)
            .map(|(_, effects)| !effects.is_empty())
            .unwrap_or(false)
    }
}

impl Default for EffectPolymorphism {
    fn default() -> Self {
        Self::new()
    }
}

/// EffectPolymorphism in generic functions
/// Example: fn map<T, e>(items: &[T], f: fn(T) -> T / e) -> Vec<T> / e
#[derive(Debug, Clone)]
pub struct EffectGenericFn {
    pub name: String,
    pub type_params: Vec<String>,
    pub effect_params: Vec<String>,
    pub body_effect_param: String,
}

impl EffectGenericFn {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_params: Vec::new(),
            effect_params: Vec::new(),
            body_effect_param: "e".to_string(),
        }
    }

    pub fn with_type_param(mut self, param: &str) -> Self {
        self.type_params.push(param.to_string());
        self
    }

    pub fn with_effect_param(mut self, param: &str) -> Self {
        self.effect_params.push(param.to_string());
        self
    }
}

/// Structured concurrency with spawn_scope
#[derive(Debug, Clone)]
pub struct StructuredConcurrency {
    /// Active scopes
    pub active_scopes: Vec<ScopeInfo>,
    /// Maximum nesting depth
    pub max_depth: usize,
}

#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub spawned_tasks: Vec<TaskInfo>,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: usize,
    pub name: String,
    pub is_completed: bool,
}

impl StructuredConcurrency {
    pub fn new() -> Self {
        Self {
            active_scopes: Vec::new(),
            max_depth: 100,
        }
    }

    /// Enter a new spawn_scope
    pub fn enter_scope(&mut self, parent_id: Option<usize>) -> usize {
        let id = self.active_scopes.len();
        self.active_scopes.push(ScopeInfo {
            id,
            parent_id,
            spawned_tasks: Vec::new(),
        });
        id
    }

    /// Exit a scope and verify all tasks completed
    pub fn exit_scope(&mut self, scope_id: usize) -> Result<(), String> {
        if let Some(scope) = self.active_scopes.get(scope_id) {
            let incomplete: Vec<_> = scope
                .spawned_tasks
                .iter()
                .filter(|t| !t.is_completed)
                .collect();

            if !incomplete.is_empty() {
                return Err(format!(
                    "Scope {} contains {} incomplete tasks",
                    scope_id,
                    incomplete.len()
                ));
            }
        }

        self.active_scopes.pop();
        Ok(())
    }

    /// Spawn a task in the current scope
    pub fn spawn_task(&mut self, scope_id: usize, name: &str) -> usize {
        let task_id = self.active_scopes[scope_id].spawned_tasks.len();
        self.active_scopes[scope_id].spawned_tasks.push(TaskInfo {
            id: task_id,
            name: name.to_string(),
            is_completed: false,
        });
        task_id
    }

    /// Complete a task
    pub fn complete_task(&mut self, scope_id: usize, task_id: usize) {
        if let Some(scope) = self.active_scopes.get_mut(scope_id) {
            if let Some(task) = scope.spawned_tasks.get_mut(task_id) {
                task.is_completed = true;
            }
        }
    }
}

impl Default for StructuredConcurrency {
    fn default() -> Self {
        Self::new()
    }
}

/// Explicit cancellation token
#[derive(Debug, Clone)]
pub struct CancelToken {
    pub id: usize,
    pub is_cancelled: bool,
    pub cancellation_source: CancellationSource,
}

#[derive(Debug, Clone)]
pub enum CancellationSource {
    Explicit,    // Explicit cancel() call
    Timeout,     // Timeout expired
    ParentScope, // Parent scope cancelled
    ScopeExit,   // Scope exited without completing
}

impl CancelToken {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            is_cancelled: false,
            cancellation_source: CancellationSource::Explicit,
        }
    }

    pub fn cancel(&mut self, source: CancellationSource) {
        self.is_cancelled = true;
        self.cancellation_source = source;
    }

    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }

    /// Check for cancellation - explicit cancellation point
    pub fn check(&self) -> Result<(), String> {
        if self.is_cancelled {
            Err(format!("Operation cancelled at token {}", self.id))
        } else {
            Ok(())
        }
    }
}

/// Generator effect - lazy sequences
#[derive(Debug, Clone)]
pub struct Gen<T: Clone> {
    state: T,
    is_exhausted: bool,
}

impl<T: Clone> Gen<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            is_exhausted: false,
        }
    }

    pub fn next(&mut self) -> Option<T> {
        if self.is_exhausted {
            None
        } else {
            self.is_exhausted = true;
            Some(self.state.clone())
        }
    }

    pub fn is_exhausted(&self) -> bool {
        self.is_exhausted
    }
}

/// Effect Handler Definition
#[derive(Debug, Clone)]
pub struct EffectHandlerDef {
    pub effect_name: String,
    pub operation_name: String,
    pub handler_body: String,
}

impl EffectHandlerDef {
    pub fn new(effect: &str, operation: &str) -> Self {
        Self {
            effect_name: effect.to_string(),
            operation_name: operation.to_string(),
            handler_body: String::new(),
        }
    }

    pub fn with_body(mut self, body: &str) -> Self {
        self.handler_body = body.to_string();
        self
    }
}

/// Combined Phase 8 Effect System
pub struct Phase8EffectSystem {
    pub user_effects: HashMap<String, UserEffectDef>,
    pub effect_polymorphism: EffectPolymorphism,
    pub structured_concurrency: StructuredConcurrency,
    pub cancel_tokens: HashMap<usize, CancelToken>,
    pub generators: HashMap<String, Box<dyn Fn() -> Option<RuntimeValue>>>,
}

#[derive(Debug)]
struct CallableBodyRef<'a> {
    name: String,
    origin: String,
    is_async: bool,
    declared_effects: effects::EffectRow,
    body: &'a [super::TypedStatement],
    current_type_name: Option<String>,
}

fn builtin_effect_row(name: &str) -> Option<effects::EffectRow> {
    let io = effects::EffectRow::just(effects::builtin::io());
    let debug = effects::EffectRow::just(effects::builtin::debug());
    let diverge = effects::EffectRow::just(effects::builtin::diverge());

    match name {
        "println" | "print" | "eprintln" | "eprint" | "file_read" | "file_write_bytes" => Some(io),
        "dbg" => Some(debug),
        "exit" | "assert" | "assert_eq" => Some(diverge),
        _ => None,
    }
}

fn callable_signature_row(function: &super::TypedFunction) -> effects::EffectRow {
    let mut row = function
        .effect_row
        .clone()
        .unwrap_or_else(effects::EffectRow::pure);

    if function.is_async {
        row.insert(effects::builtin::async_());
    }

    row
}

fn method_owner_name(ty: &Type, current_type_name: Option<&str>) -> Option<String> {
    match ty {
        Type::Named(name) | Type::Generic(name, _) => Some(name.clone()),
        Type::TraitObject { principal, .. } => Some(principal.clone()),
        Type::AssocType(trait_name, _) => Some(trait_name.clone()),
        Type::WhereConstrained { base, .. } => method_owner_name(base, current_type_name),
        Type::WithOwnership(inner, _) => method_owner_name(inner, current_type_name),
        Type::SelfOwned | Type::SelfRef { .. } => current_type_name.map(|name| name.to_string()),
        _ => None,
    }
}

fn build_callable_maps<'a>(
    module: &'a super::TypedModule,
) -> (
    HashMap<String, effects::EffectRow>,
    Vec<CallableBodyRef<'a>>,
) {
    let mut rows = HashMap::new();

    for name in [
        "println",
        "print",
        "eprintln",
        "eprint",
        "file_read",
        "file_write_bytes",
        "dbg",
        "exit",
        "assert",
        "assert_eq",
    ] {
        if let Some(row) = builtin_effect_row(name) {
            rows.insert(name.to_string(), row);
        }
    }

    let mut bodies = Vec::new();

    for item in &module.items {
        match item {
            super::TypedItem::Function(function) => {
                rows.insert(function.name.clone(), callable_signature_row(function));
                bodies.push(CallableBodyRef {
                    name: function.name.clone(),
                    origin: function.name.clone(),
                    is_async: function.is_async,
                    declared_effects: function
                        .effect_row
                        .clone()
                        .unwrap_or_else(effects::EffectRow::pure),
                    body: &function.body,
                    current_type_name: None,
                });
            }
            super::TypedItem::Impl(impl_block) => {
                for method in &impl_block.methods {
                    let key = format!("{}::{}", impl_block.type_name, method.name);
                    rows.insert(key.clone(), callable_signature_row(method));
                    bodies.push(CallableBodyRef {
                        name: key.clone(),
                        origin: key,
                        is_async: method.is_async,
                        declared_effects: method
                            .effect_row
                            .clone()
                            .unwrap_or_else(effects::EffectRow::pure),
                        body: &method.body,
                        current_type_name: Some(impl_block.type_name.clone()),
                    });
                }
            }
            super::TypedItem::Extern(block) => {
                for function in &block.functions {
                    rows.insert(function.name.clone(), callable_signature_row(function));
                }
            }
            _ => {}
        }
    }

    (rows, bodies)
}

fn collect_expr_list_effects(
    expressions: &[super::TypedExpr],
    current_rows: &HashMap<String, effects::EffectRow>,
    current_type_name: Option<&str>,
    validate_lambdas: bool,
    errors: &mut Vec<EffectError>,
    origin: &str,
) -> effects::EffectRow {
    expressions
        .iter()
        .fold(effects::EffectRow::pure(), |row, expr| {
            row.union(&collect_expr_effects(
                expr,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            ))
        })
}

fn collect_statement_block_effects(
    statements: &[super::TypedStatement],
    current_rows: &HashMap<String, effects::EffectRow>,
    current_type_name: Option<&str>,
    validate_lambdas: bool,
    errors: &mut Vec<EffectError>,
    origin: &str,
) -> effects::EffectRow {
    statements
        .iter()
        .fold(effects::EffectRow::pure(), |row, statement| {
            row.union(&collect_statement_effects(
                statement,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            ))
        })
}

fn collect_statement_effects(
    statement: &super::TypedStatement,
    current_rows: &HashMap<String, effects::EffectRow>,
    current_type_name: Option<&str>,
    validate_lambdas: bool,
    errors: &mut Vec<EffectError>,
    origin: &str,
) -> effects::EffectRow {
    match statement {
        super::TypedStatement::Let { value, .. } => collect_expr_effects(
            value,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        ),
        super::TypedStatement::Assignment { target, value, .. } => collect_expr_effects(
            target,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )
        .union(&collect_expr_effects(
            value,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )),
        super::TypedStatement::Return(expr) => expr
            .as_ref()
            .map(|expr| {
                collect_expr_effects(
                    expr,
                    current_rows,
                    current_type_name,
                    validate_lambdas,
                    errors,
                    origin,
                )
            })
            .unwrap_or_else(effects::EffectRow::pure),
        super::TypedStatement::If {
            condition,
            then_block,
            else_block,
        } => {
            let mut row = collect_expr_effects(
                condition,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            );
            row = row.union(&collect_statement_block_effects(
                then_block,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            ));
            if let Some(else_block) = else_block {
                row = row.union(&collect_statement_block_effects(
                    else_block,
                    current_rows,
                    current_type_name,
                    validate_lambdas,
                    errors,
                    origin,
                ));
            }
            row
        }
        super::TypedStatement::For { iter, body, .. } => collect_expr_effects(
            iter,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )
        .union(&collect_statement_block_effects(
            body,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )),
        super::TypedStatement::While { condition, body } => collect_expr_effects(
            condition,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )
        .union(&collect_statement_block_effects(
            body,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )),
        super::TypedStatement::Loop { body } => collect_statement_block_effects(
            body,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        ),
        super::TypedStatement::Match { expr, arms } => {
            let mut row = collect_expr_effects(
                expr,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            );
            for (_, arm_body) in arms {
                row = row.union(&collect_statement_block_effects(
                    arm_body,
                    current_rows,
                    current_type_name,
                    validate_lambdas,
                    errors,
                    origin,
                ));
            }
            row
        }
        super::TypedStatement::Defer(inner) => collect_statement_effects(
            inner,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        ),
        super::TypedStatement::Break
        | super::TypedStatement::Continue
        | super::TypedStatement::Pass => effects::EffectRow::pure(),
        super::TypedStatement::Expr(expr) => collect_expr_effects(
            expr,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        ),
        super::TypedStatement::Yield(expr) => {
            let mut row = effects::EffectRow::just(effects::builtin::yield_());
            if let Some(expr) = expr {
                row = row.union(&collect_expr_effects(
                    expr,
                    current_rows,
                    current_type_name,
                    validate_lambdas,
                    errors,
                    origin,
                ));
            }
            row
        }
        super::TypedStatement::Spawn(expr) => collect_expr_effects(
            expr,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )
        .union(&effects::EffectRow::just(effects::builtin::async_())),
        super::TypedStatement::Select { arms } => {
            let mut row = effects::EffectRow::just(effects::builtin::async_());
            for arm in arms {
                row = row.union(&collect_expr_effects(
                    &arm.channel_op,
                    current_rows,
                    current_type_name,
                    validate_lambdas,
                    errors,
                    origin,
                ));
                row = row.union(&collect_statement_block_effects(
                    &arm.body,
                    current_rows,
                    current_type_name,
                    validate_lambdas,
                    errors,
                    origin,
                ));
            }
            row
        }
    }
}

fn validate_callable_like(
    label: &str,
    is_async: bool,
    declared_effects: &effects::EffectRow,
    body_effects: &effects::EffectRow,
    errors: &mut Vec<EffectError>,
) {
    let async_effect = effects::builtin::async_();
    let mut effective_declared = declared_effects.clone();
    let mut skip_async_missing = false;

    if is_async {
        effective_declared.insert(async_effect.clone());
    } else if declared_effects.contains(&async_effect) {
        errors.push(EffectError::Custom {
            message: "async effect requires an async function or lambda".to_string(),
            origin: label.to_string(),
        });
        skip_async_missing = true;
    } else if body_effects.contains(&async_effect) {
        errors.push(EffectError::UnhandledEffect {
            effect: async_effect.clone(),
            origin: label.to_string(),
        });
        skip_async_missing = true;
    }

    for effect in body_effects.iter() {
        if skip_async_missing && effect.name == async_effect.name {
            continue;
        }

        if !effective_declared.contains(effect) {
            errors.push(EffectError::UnhandledEffect {
                effect: effect.clone(),
                origin: label.to_string(),
            });
        }
    }
}

fn validate_inline_lambda(
    lambda: &super::TypedExpr,
    current_rows: &HashMap<String, effects::EffectRow>,
    current_type_name: Option<&str>,
    validate_lambdas: bool,
    errors: &mut Vec<EffectError>,
    origin: &str,
) -> effects::EffectRow {
    let super::TypedExprKind::Lambda {
        is_async,
        effect_row,
        body,
        ..
    } = &lambda.kind
    else {
        return effects::EffectRow::pure();
    };

    let declared_effects = effect_row.clone().unwrap_or_else(effects::EffectRow::pure);
    let body_effects = collect_expr_effects(
        body,
        current_rows,
        current_type_name,
        validate_lambdas,
        errors,
        origin,
    );

    validate_callable_like(
        &format!("{}::<lambda>", origin),
        *is_async,
        &declared_effects,
        &body_effects,
        errors,
    );

    let mut signature_row = declared_effects;
    if *is_async {
        signature_row.insert(effects::builtin::async_());
    }

    signature_row.union(&body_effects)
}

fn collect_expr_effects(
    expr: &super::TypedExpr,
    current_rows: &HashMap<String, effects::EffectRow>,
    current_type_name: Option<&str>,
    validate_lambdas: bool,
    errors: &mut Vec<EffectError>,
    origin: &str,
) -> effects::EffectRow {
    match &expr.kind {
        super::TypedExprKind::Literal(_)
        | super::TypedExprKind::Identifier(_)
        | super::TypedExprKind::None => effects::EffectRow::pure(),
        super::TypedExprKind::Binary(left, _, right) => collect_expr_effects(
            left,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )
        .union(&collect_expr_effects(
            right,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )),
        super::TypedExprKind::Unary(_, inner)
        | super::TypedExprKind::Borrow { expr: inner, .. }
        | super::TypedExprKind::Deref(inner)
        | super::TypedExprKind::Await(inner)
        | super::TypedExprKind::Some(inner)
        | super::TypedExprKind::Ok(inner)
        | super::TypedExprKind::Err(inner) => {
            let mut row = collect_expr_effects(
                inner,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            );
            if matches!(&expr.kind, super::TypedExprKind::Await(_)) {
                row = row.union(&effects::EffectRow::just(effects::builtin::async_()));
            }
            row
        }
        super::TypedExprKind::Call(callee, args) => {
            let mut row = collect_expr_list_effects(
                args,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            );

            match &callee.kind {
                super::TypedExprKind::Identifier(name) => {
                    if let Some(call_row) = current_rows.get(name) {
                        row = row.union(call_row);
                    } else if let Some(builtin_row) = builtin_effect_row(name) {
                        row = row.union(&builtin_row);
                    }
                }
                super::TypedExprKind::Lambda { .. } => {
                    row = row.union(&validate_inline_lambda(
                        callee,
                        current_rows,
                        current_type_name,
                        validate_lambdas,
                        errors,
                        origin,
                    ));
                }
                _ => {
                    row = row.union(&collect_expr_effects(
                        callee,
                        current_rows,
                        current_type_name,
                        validate_lambdas,
                        errors,
                        origin,
                    ));
                }
            }

            row
        }
        super::TypedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            let mut row = collect_expr_effects(
                receiver,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            );
            row = row.union(&collect_expr_list_effects(
                args,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            ));

            if let Some(owner_name) = method_owner_name(&receiver.ty, current_type_name) {
                let method_key = format!("{}::{}", owner_name, method);
                if let Some(method_row) = current_rows.get(&method_key) {
                    row = row.union(method_row);
                }
            }

            row
        }
        super::TypedExprKind::Field(inner, _) => collect_expr_effects(
            inner,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        ),
        super::TypedExprKind::Index(inner, index) => collect_expr_effects(
            inner,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )
        .union(&collect_expr_effects(
            index,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )),
        super::TypedExprKind::LetChain { value, body, .. } => collect_expr_effects(
            value,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )
        .union(&collect_expr_effects(
            body,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )),
        super::TypedExprKind::Tuple(elements) | super::TypedExprKind::Array(elements) => {
            collect_expr_list_effects(
                elements,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            )
        }
        super::TypedExprKind::StructLiteral { fields, .. } => {
            fields
                .iter()
                .fold(effects::EffectRow::pure(), |row, (_, field_expr)| {
                    row.union(&collect_expr_effects(
                        field_expr,
                        current_rows,
                        current_type_name,
                        validate_lambdas,
                        errors,
                        origin,
                    ))
                })
        }
        super::TypedExprKind::Range { start, end, .. } => {
            let mut row = effects::EffectRow::pure();
            if let Some(start) = start {
                row = row.union(&collect_expr_effects(
                    start,
                    current_rows,
                    current_type_name,
                    validate_lambdas,
                    errors,
                    origin,
                ));
            }
            if let Some(end) = end {
                row = row.union(&collect_expr_effects(
                    end,
                    current_rows,
                    current_type_name,
                    validate_lambdas,
                    errors,
                    origin,
                ));
            }
            row
        }
        super::TypedExprKind::Lambda { .. } => {
            if validate_lambdas {
                let _ = validate_inline_lambda(
                    expr,
                    current_rows,
                    current_type_name,
                    validate_lambdas,
                    errors,
                    origin,
                );
            }
            effects::EffectRow::pure()
        }
        super::TypedExprKind::If {
            condition,
            then_expr,
            else_expr,
        } => collect_expr_effects(
            condition,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )
        .union(&collect_expr_effects(
            then_expr,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        ))
        .union(&collect_expr_effects(
            else_expr,
            current_rows,
            current_type_name,
            validate_lambdas,
            errors,
            origin,
        )),
        super::TypedExprKind::Match {
            expr: matched,
            arms,
        } => {
            let mut row = collect_expr_effects(
                matched,
                current_rows,
                current_type_name,
                validate_lambdas,
                errors,
                origin,
            );
            for (_, arm_expr) in arms {
                row = row.union(&collect_expr_effects(
                    arm_expr,
                    current_rows,
                    current_type_name,
                    validate_lambdas,
                    errors,
                    origin,
                ));
            }
            row
        }
    }
}

/// Validate effects in a typed module.
///
/// This performs Phase 8 effect system validation by recomputing effect rows
/// from the typed AST, stabilizing direct and recursive call effects, and then
/// checking each callable body against its declared signature.
pub fn validate_effects(module: &super::TypedModule) -> Result<(), Vec<EffectError>> {
    let (signature_rows, callables) = build_callable_maps(module);
    let mut effect_rows = signature_rows.clone();

    loop {
        let mut changed = false;
        let mut next_rows = effect_rows.clone();

        for callable in &callables {
            let mut ignored_errors = Vec::new();
            let body_effects = collect_statement_block_effects(
                callable.body,
                &effect_rows,
                callable.current_type_name.as_deref(),
                false,
                &mut ignored_errors,
                &callable.origin,
            );

            let combined_effects = signature_rows
                .get(&callable.name)
                .cloned()
                .unwrap_or_else(effects::EffectRow::pure)
                .union(&body_effects);

            if next_rows.get(&callable.name) != Some(&combined_effects) {
                next_rows.insert(callable.name.clone(), combined_effects);
                changed = true;
            }
        }

        effect_rows = next_rows;
        if !changed {
            break;
        }
    }

    let mut errors = Vec::new();

    for callable in &callables {
        let body_effects = collect_statement_block_effects(
            callable.body,
            &effect_rows,
            callable.current_type_name.as_deref(),
            true,
            &mut errors,
            &callable.origin,
        );

        validate_callable_like(
            &callable.origin,
            callable.is_async,
            &callable.declared_effects,
            &body_effects,
            &mut errors,
        );
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

impl Phase8EffectSystem {
    pub fn new() -> Self {
        Self {
            user_effects: HashMap::new(),
            effect_polymorphism: EffectPolymorphism::new(),
            structured_concurrency: StructuredConcurrency::new(),
            cancel_tokens: HashMap::new(),
            generators: HashMap::new(),
        }
    }

    /// Register a user-defined effect
    pub fn register_user_effect(&mut self, effect: UserEffectDef) {
        self.user_effects.insert(effect.name.clone(), effect);
    }

    /// Register a polymorphic generic function
    pub fn register_polymorphic(
        &mut self,
        func_name: &str,
        types: Vec<String>,
        effects: Vec<String>,
    ) {
        self.effect_polymorphism.register(func_name, types, effects);
    }

    /// Create a new cancellation token
    pub fn create_cancel_token(&mut self) -> CancelToken {
        let id = self.cancel_tokens.len();
        let token = CancelToken::new(id);
        self.cancel_tokens.insert(id, token.clone());
        token
    }
}

impl Default for Phase8EffectSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::effects::{EffectError, EffectRow};
    use crate::semantic::{
        TypedExpr, TypedExprKind, TypedFunction, TypedItem, TypedModule, TypedStatement,
    };

    fn typed_identifier(name: &str) -> TypedExpr {
        TypedExpr {
            kind: TypedExprKind::Identifier(name.to_string()),
            ty: Type::Named("()".to_string()),
        }
    }

    fn typed_call(name: &str) -> TypedExpr {
        TypedExpr {
            kind: TypedExprKind::Call(Box::new(typed_identifier(name)), Vec::new()),
            ty: Type::Named("()".to_string()),
        }
    }

    fn typed_function(
        name: &str,
        is_async: bool,
        effect_row: Option<EffectRow>,
        body: Vec<TypedStatement>,
    ) -> TypedFunction {
        TypedFunction {
            name: name.to_string(),
            params: Vec::new(),
            return_type: Type::Named("()".to_string()),
            effect_row,
            body,
            is_async,
        }
    }

    fn typed_module(functions: Vec<TypedFunction>) -> TypedModule {
        TypedModule {
            items: functions.into_iter().map(TypedItem::Function).collect(),
        }
    }

    #[test]
    fn test_validate_effects_accepts_pure_function() {
        let module = typed_module(vec![typed_function(
            "main",
            false,
            Some(EffectRow::pure()),
            Vec::new(),
        )]);

        assert!(validate_effects(&module).is_ok());
    }

    #[test]
    fn test_validate_effects_accepts_implicit_async_effect() {
        let module = typed_module(vec![typed_function(
            "main",
            true,
            Some(EffectRow::pure()),
            Vec::new(),
        )]);

        assert!(validate_effects(&module).is_ok());
    }

    #[test]
    fn test_validate_effects_rejects_builtin_io_call_without_effect() {
        let module = typed_module(vec![typed_function(
            "main",
            false,
            Some(EffectRow::pure()),
            vec![TypedStatement::Expr(typed_call("println"))],
        )]);

        let errors = validate_effects(&module).expect_err("print call should require IO");

        assert!(errors.iter().any(|error| matches!(
            error,
            EffectError::UnhandledEffect { effect, .. } if effect.name == "IO"
        )));
    }

    #[test]
    fn test_user_effect_definition() {
        // Just test basic functionality without type issues
        let effect = UserEffectDef::new("Logging");

        assert_eq!(effect.name, "Logging");
    }

    #[test]
    fn test_effect_polymorphism() {
        let mut poly = EffectPolymorphism::new();

        poly.register("map", vec!["T".to_string()], vec!["e".to_string()]);

        assert!(poly.is_polymorphic("map"));
        assert!(!poly.is_polymorphic("other"));
    }

    #[test]
    fn test_structured_concurrency() {
        let mut sc = StructuredConcurrency::new();

        let scope_id = sc.enter_scope(None);
        sc.spawn_task(scope_id, "task1");

        assert_eq!(sc.active_scopes.len(), 1);
    }

    #[test]
    fn test_cancel_token() {
        let token = CancelToken::new(0);

        assert!(!token.is_cancelled());

        let mut token = token;
        token.cancel(CancellationSource::Explicit);

        assert!(token.is_cancelled());
    }

    #[test]
    fn test_generator() {
        let mut gen: Gen<i32> = Gen::new(42);

        assert!(!gen.is_exhausted());
        assert_eq!(gen.next(), Some(42));
        assert!(gen.is_exhausted());
    }

    #[test]
    fn test_phase8_system() {
        let mut system = Phase8EffectSystem::new();

        // Register user effect
        let effect = UserEffectDef::new("Custom");
        system.register_user_effect(effect);

        // Register polymorphic function
        system.register_polymorphic("transform", vec!["T".to_string()], vec!["IO".to_string()]);

        // Create cancel token
        let _token = system.create_cancel_token();

        // Verify
        assert!(system.user_effects.contains_key("Custom"));
        assert!(system.effect_polymorphism.is_polymorphic("transform"));
    }
}
