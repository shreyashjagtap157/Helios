#![allow(dead_code)]
//! Ownership and Borrow Checking for Omni
//!
//! Implements Omni's ownership model (similar to Rust but with `own`, `shared`, `&` references):
//! - Ownership tracking (Own, Shared, Borrowed, MutBorrowed, Moved)
//! - Move semantics with use-after-move detection
//! - Borrow conflict detection (double mut, mut+shared)
//! - Scope-based lifetime tracking
//! - Dangling reference and return-of-local-reference detection

use std::collections::HashMap;
use crate::parser::ast;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// The ownership kind of a binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipKind {
    /// Uniquely owned value (`own T`).
    Own,
    /// Reference-counted / shared ownership (`shared T`).
    Shared,
    /// Immutable borrow (`&T`).
    Borrowed,
    /// Mutable borrow (`&mut T`).
    MutBorrowed,
    /// Value has been moved - any further use is illegal.
    Moved,
}

/// Location information attached to borrow events (statement index serves as
/// a lightweight stand-in for real source spans).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub stmt_index: usize,
}

impl Location {
    pub fn new(idx: usize) -> Self {
        Self { stmt_index: idx }
    }
}

/// An active immutable borrow on a variable.
#[derive(Debug, Clone)]
pub struct ImmutableBorrow {
    pub scope_id: usize,
    pub location: Location,
}

/// An active mutable borrow on a variable.
#[derive(Debug, Clone)]
pub struct MutableBorrow {
    pub scope_id: usize,
    pub location: Location,
}

/// Per-variable tracking of ownership and borrow state.
#[derive(Debug, Clone)]
pub struct BorrowState {
    /// Current ownership kind.
    pub ownership: OwnershipKind,
    /// Whether the binding was declared `mut`.
    pub mutable: bool,
    /// Scope in which the variable was declared.
    pub declared_scope: usize,
    /// Location where the variable was declared.
    pub declared_at: Location,
    /// If moved, where it was moved.
    pub moved_at: Option<Location>,
    /// Active immutable borrows.
    pub immutable_borrows: Vec<ImmutableBorrow>,
    /// Active mutable borrow (at most one).
    pub mutable_borrow: Option<MutableBorrow>,
}

impl BorrowState {
    pub fn new_owned(mutable: bool, scope_id: usize, loc: Location) -> Self {
        Self {
            ownership: OwnershipKind::Own,
            mutable,
            declared_scope: scope_id,
            declared_at: loc,
            moved_at: None,
            immutable_borrows: Vec::new(),
            mutable_borrow: None,
        }
    }

    pub fn has_any_borrow(&self) -> bool {
        !self.immutable_borrows.is_empty() || self.mutable_borrow.is_some()
    }
}

// ---------------------------------------------------------------------------
// Scope
// ---------------------------------------------------------------------------

/// A lexical scope that tracks which variables and borrows were introduced.
#[derive(Debug, Clone)]
pub struct Scope {
    pub id: usize,
    /// Variables declared in this scope.
    pub variables: Vec<String>,
    /// Borrows created in this scope: `(var_name, is_mutable)`.
    pub borrows: Vec<(String, bool)>,
}

impl Scope {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            variables: Vec::new(),
            borrows: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Rich borrow-checking error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BorrowError {
    /// Value used after being moved.
    UseAfterMove {
        variable: String,
        moved_at: Location,
        used_at: Location,
    },
    /// Two simultaneous mutable borrows.
    DoubleMutBorrow {
        variable: String,
        first_borrow: Location,
        second_borrow: Location,
    },
    /// Mutable borrow attempted while immutable borrows exist.
    MutBorrowWhileShared {
        variable: String,
        shared_at: Location,
        mut_at: Location,
    },
    /// Value moved while active borrows exist.
    MovedWhileBorrowed {
        variable: String,
        borrow_at: Location,
        move_at: Location,
    },
    /// Reference outlives its owner (scope).
    DanglingReference {
        variable: String,
        ref_location: Location,
    },
    /// Assignment to immutable variable.
    MutationOfImmutable {
        variable: String,
        assign_at: Location,
    },
    /// Returning a reference to a local variable.
    ReturnLocalReference {
        variable: String,
        return_at: Location,
    },
    /// Use of undeclared variable (forwarded from borrow checker context).
    UndeclaredVariable {
        variable: String,
        used_at: Location,
    },
    /// Shared borrow attempted while mutable borrow exists.
    SharedBorrowWhileMut {
        variable: String,
        mut_at: Location,
        shared_at: Location,
    },
    /// Move inside a loop body (the variable is used again in the next iteration).
    MoveInLoop {
        variable: String,
        move_at: Location,
    },
}

impl std::fmt::Display for BorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BorrowError::UseAfterMove { variable, moved_at, used_at } => {
                write!(f, "use of moved value `{}`: moved at stmt {}, used at stmt {}",
                       variable, moved_at.stmt_index, used_at.stmt_index)
            }
            BorrowError::DoubleMutBorrow { variable, first_borrow, second_borrow } => {
                write!(f, "cannot borrow `{}` as mutable more than once: first at stmt {}, second at stmt {}",
                       variable, first_borrow.stmt_index, second_borrow.stmt_index)
            }
            BorrowError::MutBorrowWhileShared { variable, shared_at, mut_at } => {
                write!(f, "cannot borrow `{}` as mutable while shared borrow exists (shared at stmt {}, mut at stmt {})",
                       variable, shared_at.stmt_index, mut_at.stmt_index)
            }
            BorrowError::MovedWhileBorrowed { variable, borrow_at, move_at } => {
                write!(f, "cannot move `{}` while borrowed (borrow at stmt {}, move at stmt {})",
                       variable, borrow_at.stmt_index, move_at.stmt_index)
            }
            BorrowError::DanglingReference { variable, ref_location } => {
                write!(f, "dangling reference to `{}` at stmt {}", variable, ref_location.stmt_index)
            }
            BorrowError::MutationOfImmutable { variable, assign_at } => {
                write!(f, "cannot assign to immutable variable `{}` at stmt {}", variable, assign_at.stmt_index)
            }
            BorrowError::ReturnLocalReference { variable, return_at } => {
                write!(f, "cannot return reference to local variable `{}` at stmt {}", variable, return_at.stmt_index)
            }
            BorrowError::UndeclaredVariable { variable, used_at } => {
                write!(f, "undeclared variable `{}` at stmt {}", variable, used_at.stmt_index)
            }
            BorrowError::SharedBorrowWhileMut { variable, mut_at, shared_at } => {
                write!(f, "cannot borrow `{}` as shared while mutable borrow exists (mut at stmt {}, shared at stmt {})",
                       variable, mut_at.stmt_index, shared_at.stmt_index)
            }
            BorrowError::MoveInLoop { variable, move_at } => {
                write!(f, "value `{}` moved inside loop at stmt {} (would be used again in next iteration)",
                       variable, move_at.stmt_index)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// BorrowChecker
// ---------------------------------------------------------------------------

/// The main borrow checker that walks the AST and validates ownership rules.
pub struct BorrowChecker {
    /// Per-variable borrow state (keyed by variable name).
    variables: HashMap<String, BorrowState>,
    /// Stack of lexical scopes.
    scopes: Vec<Scope>,
    /// Monotonically increasing scope id.
    next_scope_id: usize,
    /// Current statement index (used as a lightweight location).
    current_stmt: usize,
    /// Accumulated errors.
    errors: Vec<BorrowError>,
    /// Whether we are currently inside a loop body.
    in_loop: bool,
    /// Variables moved inside the current loop iteration (for loop-move detection).
    loop_moves: Vec<String>,
    /// Whether the current function has a reference return type.
    returns_reference: bool,
}

impl BorrowChecker {
    // ------------------------------------------------------------------
    // Construction
    // ------------------------------------------------------------------

    pub fn new() -> Self {
        let root_scope = Scope::new(0);
        Self {
            variables: HashMap::new(),
            scopes: vec![root_scope],
            next_scope_id: 1,
            current_stmt: 0,
            errors: Vec::new(),
            in_loop: false,
            loop_moves: Vec::new(),
            returns_reference: false,
        }
    }

    fn current_scope_id(&self) -> usize {
        self.scopes.last().map(|s| s.id).unwrap_or(0)
    }

    fn loc(&self) -> Location {
        Location::new(self.current_stmt)
    }

    // ------------------------------------------------------------------
    // Scope management
    // ------------------------------------------------------------------

    pub fn enter_scope(&mut self) {
        let id = self.next_scope_id;
        self.next_scope_id += 1;
        self.scopes.push(Scope::new(id));
    }

    pub fn exit_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            // Release all borrows that were created in this scope.
            for (var_name, is_mut) in &scope.borrows {
                self.release_borrow_internal(var_name, *is_mut);
            }
            // Remove variables declared in this scope.
            for var in &scope.variables {
                self.variables.remove(var);
            }
        }
    }

    // ------------------------------------------------------------------
    // Backward-compatible API (used by Analyzer in mod.rs)
    // ------------------------------------------------------------------

    /// Alias for `enter_scope` (old API compatibility).
    pub fn push_scope(&mut self) {
        self.enter_scope();
    }

    /// Alias for `exit_scope` (old API compatibility).
    pub fn pop_scope(&mut self) {
        self.exit_scope();
    }

    /// Declare a variable (old API: returns Result).
    pub fn bind_variable(&mut self, name: String) -> Result<(), String> {
        self.declare_variable(&name, true);
        Ok(())
    }

    /// Create a shared (immutable) borrow (old API).
    pub fn borrow_shared(&mut self, name: &str) -> Result<(), String> {
        let before = self.errors.len();
        self.add_borrow(name, false);
        if self.errors.len() > before {
            let err = self.errors.pop().unwrap();
            Err(format!("{}", err))
        } else if !self.variables.contains_key(name) {
            Err(format!("Variable {} not found", name))
        } else {
            Ok(())
        }
    }

    /// Create a mutable borrow (old API).
    pub fn borrow_mut_compat(&mut self, name: &str) -> Result<(), String> {
        let before = self.errors.len();
        self.add_borrow(name, true);
        if self.errors.len() > before {
            let err = self.errors.pop().unwrap();
            Err(format!("{}", err))
        } else if !self.variables.contains_key(name) {
            Err(format!("Variable {} not found", name))
        } else {
            Ok(())
        }
    }

    /// Move a variable (old API).
    pub fn move_var(&mut self, name: &str) -> Result<(), String> {
        let before = self.errors.len();
        self.mark_moved(name);
        if self.errors.len() > before {
            let err = self.errors.pop().unwrap();
            Err(format!("{}", err))
        } else if !self.variables.contains_key(name) {
            Err(format!("Variable {} not found", name))
        } else {
            Ok(())
        }
    }

    /// Check if a variable can be read (old API).
    pub fn can_read(&self, name: &str) -> Result<(), String> {
        match self.variables.get(name) {
            None => Err(format!("Variable {} not found", name)),
            Some(state) if state.ownership == OwnershipKind::Moved => {
                Err(format!("Variable {} used after move", name))
            }
            _ => Ok(()),
        }
    }

    /// Check if a variable can be written to (old API).
    pub fn can_write(&self, name: &str) -> Result<(), String> {
        match self.variables.get(name) {
            None => Err(format!("Variable {} not found", name)),
            Some(state) => {
                if state.ownership == OwnershipKind::Moved {
                    Err(format!("Variable {} used after move", name))
                } else if state.has_any_borrow() {
                    Err(format!("Cannot mutate {} while borrowed", name))
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Return (release) a borrow on a variable (old API).
    pub fn return_borrow(&mut self, name: &str) -> Result<(), String> {
        if !self.variables.contains_key(name) {
            return Err(format!("Variable {} not found", name));
        }
        self.release_borrow(name);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Variable management
    // ------------------------------------------------------------------

    /// Declare a new variable in the current scope.
    pub fn declare_variable(&mut self, name: &str, mutable: bool) {
        let scope_id = self.current_scope_id();
        let loc = self.loc();
        let state = BorrowState::new_owned(mutable, scope_id, loc);
        self.variables.insert(name.to_string(), state);
        if let Some(scope) = self.scopes.last_mut() {
            scope.variables.push(name.to_string());
        }
    }

    /// Mark a variable as moved. Returns error if already moved or borrowed.
    pub fn mark_moved(&mut self, var_name: &str) {
        let loc = self.loc();

        let state = match self.variables.get(var_name) {
            Some(s) => s.clone(),
            None => {
                // Undeclared - skip (may be a global / external).
                return;
            }
        };

        match state.ownership {
            OwnershipKind::Moved => {
                self.errors.push(BorrowError::UseAfterMove {
                    variable: var_name.to_string(),
                    moved_at: state.moved_at.unwrap_or(loc),
                    used_at: loc,
                });
            }
            OwnershipKind::Shared => {
                // Shared (RC) values are cloned on move - no error.
                return;
            }
            _ => {
                // Check for active borrows.
                if state.has_any_borrow() {
                    let borrow_loc = state.mutable_borrow.as_ref()
                        .map(|b| b.location)
                        .or_else(|| state.immutable_borrows.first().map(|b| b.location))
                        .unwrap_or(loc);
                    self.errors.push(BorrowError::MovedWhileBorrowed {
                        variable: var_name.to_string(),
                        borrow_at: borrow_loc,
                        move_at: loc,
                    });
                    return;
                }
            }
        }

        // Perform the move.
        if let Some(s) = self.variables.get_mut(var_name) {
            s.ownership = OwnershipKind::Moved;
            s.moved_at = Some(loc);
        }

        // Loop-move tracking.
        if self.in_loop {
            self.loop_moves.push(var_name.to_string());
        }
    }

    /// Record a borrow on `var_name`. If `mutable` is true, it is a `&mut` borrow.
    pub fn add_borrow(&mut self, var_name: &str, mutable: bool) {
        let loc = self.loc();
        let scope_id = self.current_scope_id();

        let state = match self.variables.get(var_name) {
            Some(s) => s.clone(),
            None => return,
        };

        // Cannot borrow a moved value.
        if state.ownership == OwnershipKind::Moved {
            self.errors.push(BorrowError::UseAfterMove {
                variable: var_name.to_string(),
                moved_at: state.moved_at.unwrap_or(loc),
                used_at: loc,
            });
            return;
        }

        if mutable {
            // Mut borrow conflicts.
            if let Some(ref mb) = state.mutable_borrow {
                self.errors.push(BorrowError::DoubleMutBorrow {
                    variable: var_name.to_string(),
                    first_borrow: mb.location,
                    second_borrow: loc,
                });
                return;
            }
            if let Some(ib) = state.immutable_borrows.first() {
                self.errors.push(BorrowError::MutBorrowWhileShared {
                    variable: var_name.to_string(),
                    shared_at: ib.location,
                    mut_at: loc,
                });
                return;
            }
            if let Some(s) = self.variables.get_mut(var_name) {
                s.mutable_borrow = Some(MutableBorrow { scope_id, location: loc });
            }
        } else {
            // Shared borrow conflicts with existing mut borrow.
            if let Some(ref mb) = state.mutable_borrow {
                self.errors.push(BorrowError::SharedBorrowWhileMut {
                    variable: var_name.to_string(),
                    mut_at: mb.location,
                    shared_at: loc,
                });
                return;
            }
            if let Some(s) = self.variables.get_mut(var_name) {
                s.immutable_borrows.push(ImmutableBorrow { scope_id, location: loc });
            }
        }

        // Track borrow in current scope for cleanup on exit.
        if let Some(scope) = self.scopes.last_mut() {
            scope.borrows.push((var_name.to_string(), mutable));
        }
    }

    /// Release a borrow (called during scope exit).
    pub fn release_borrow(&mut self, var_name: &str) {
        self.release_borrow_internal(var_name, false);
        self.release_borrow_internal(var_name, true);
    }

    fn release_borrow_internal(&mut self, var_name: &str, mutable: bool) {
        if let Some(state) = self.variables.get_mut(var_name) {
            if mutable {
                state.mutable_borrow = None;
            } else if !state.immutable_borrows.is_empty() {
                state.immutable_borrows.pop();
            }
        }
    }

    /// Check that reading `var_name` is legal.
    fn check_use(&mut self, var_name: &str) {
        let loc = self.loc();
        let state = match self.variables.get(var_name) {
            Some(s) => s,
            None => return, // external / unknown
        };
        if state.ownership == OwnershipKind::Moved {
            self.errors.push(BorrowError::UseAfterMove {
                variable: var_name.to_string(),
                moved_at: state.moved_at.unwrap_or(loc),
                used_at: loc,
            });
        }
    }

    // ------------------------------------------------------------------
    // AST walking - public entry points
    // ------------------------------------------------------------------

    /// Check an entire module, returning all borrow errors found.
    pub fn check_module(module: &ast::Module) -> Vec<BorrowError> {
        let mut checker = BorrowChecker::new();
        for item in &module.items {
            checker.check_item(item);
        }
        checker.errors
    }

    /// Check a single top-level item.
    fn check_item(&mut self, item: &ast::Item) {
        match item {
            ast::Item::Function(func) => self.check_function(func),
            ast::Item::Struct(s) => {
                for m in &s.methods {
                    self.check_function(m);
                }
            }
            ast::Item::Impl(imp) => {
                for m in &imp.methods {
                    self.check_function(m);
                }
            }
            ast::Item::Trait(t) => {
                for m in &t.methods {
                    self.check_function(m);
                }
            }
            ast::Item::Module(md) => {
                for sub in &md.items {
                    self.check_item(sub);
                }
            }
            _ => {}
        }
    }

    /// Analyse a function body.
    pub fn check_function(&mut self, func: &ast::Function) {
        // Each function gets its own clean scope and state.
        let saved_vars = self.variables.clone();
        let saved_scopes = self.scopes.clone();
        let saved_next = self.next_scope_id;
        let saved_stmt = self.current_stmt;
        let saved_in_loop = self.in_loop;
        let saved_loop_moves = self.loop_moves.clone();
        let saved_returns_ref = self.returns_reference;

        self.variables.clear();
        self.scopes = vec![Scope::new(0)];
        self.next_scope_id = 1;
        self.current_stmt = 0;
        self.in_loop = false;
        self.loop_moves.clear();
        self.returns_reference = Self::type_is_reference(func.return_type.as_ref());

        // Bind parameters.
        for param in &func.params {
            let is_borrow = Self::type_is_reference(Some(&param.ty));
            self.declare_variable(&param.name, false);
            if is_borrow {
                // Mark the parameter's ownership kind as Borrowed / MutBorrowed.
                if let Some(s) = self.variables.get_mut(&param.name) {
                    s.ownership = if Self::type_is_mut_reference(Some(&param.ty)) {
                        OwnershipKind::MutBorrowed
                    } else {
                        OwnershipKind::Borrowed
                    };
                }
            }
        }

        self.check_block(&func.body);

        // Restore previous state.
        self.variables = saved_vars;
        self.scopes = saved_scopes;
        self.next_scope_id = saved_next;
        self.current_stmt = saved_stmt;
        self.in_loop = saved_in_loop;
        self.loop_moves = saved_loop_moves;
        self.returns_reference = saved_returns_ref;
    }

    fn check_block(&mut self, block: &ast::Block) {
        self.enter_scope();
        for (i, stmt) in block.statements.iter().enumerate() {
            self.current_stmt = i;
            self.check_statement(stmt);
        }
        self.exit_scope();
    }

    // ------------------------------------------------------------------
    // Statement checking
    // ------------------------------------------------------------------

    pub fn check_statement(&mut self, stmt: &ast::Statement) {
        match stmt {
            ast::Statement::Let { name, mutable, ty, value } => {
                // Evaluate the RHS first (may move values).
                self.check_expression(value);
                // Check if the value is an identifier being moved.
                if let ast::Expression::Identifier(ref src) = value {
                    self.mark_moved(src);
                }
                let is_shared = ty.as_ref().map_or(false, |t| Self::type_is_shared(t));
                self.declare_variable(name, *mutable);
                if is_shared {
                    if let Some(s) = self.variables.get_mut(name) {
                        s.ownership = OwnershipKind::Shared;
                    }
                }
            }
            ast::Statement::Var { name, ty: _, value } => {
                if let Some(val) = value {
                    self.check_expression(val);
                    if let ast::Expression::Identifier(ref src) = val {
                        self.mark_moved(src);
                    }
                }
                self.declare_variable(name, true); // `var` is always mutable
            }
            ast::Statement::Assignment { target, op: _, value } => {
                self.check_assignment(target, value);
            }
            ast::Statement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.check_return_expression(expr);
                    self.check_expression(expr);
                }
            }
            ast::Statement::If { condition, then_block, else_block } => {
                self.check_expression(condition);
                self.check_block(then_block);
                if let Some(eb) = else_block {
                    self.check_block(eb);
                }
            }
            ast::Statement::For { var, iter, body } => {
                self.check_expression(iter);
                let was_in_loop = self.in_loop;
                let saved_loop_moves = std::mem::take(&mut self.loop_moves);
                self.in_loop = true;

                self.enter_scope();
                self.declare_variable(var, false);
                for (i, s) in body.statements.iter().enumerate() {
                    self.current_stmt = i;
                    self.check_statement(s);
                }
                // Any variable moved inside the loop body is an error if it was
                // declared outside the loop.
                self.check_loop_moves();
                self.exit_scope();

                self.in_loop = was_in_loop;
                self.loop_moves = saved_loop_moves;
            }
            ast::Statement::While { condition, body } => {
                self.check_expression(condition);
                let was_in_loop = self.in_loop;
                let saved_loop_moves = std::mem::take(&mut self.loop_moves);
                self.in_loop = true;

                self.enter_scope();
                for (i, s) in body.statements.iter().enumerate() {
                    self.current_stmt = i;
                    self.check_statement(s);
                }
                self.check_loop_moves();
                self.exit_scope();

                self.in_loop = was_in_loop;
                self.loop_moves = saved_loop_moves;
            }
            ast::Statement::Loop { body } => {
                let was_in_loop = self.in_loop;
                let saved_loop_moves = std::mem::take(&mut self.loop_moves);
                self.in_loop = true;

                self.enter_scope();
                for (i, s) in body.statements.iter().enumerate() {
                    self.current_stmt = i;
                    self.check_statement(s);
                }
                self.check_loop_moves();
                self.exit_scope();

                self.in_loop = was_in_loop;
                self.loop_moves = saved_loop_moves;
            }
            ast::Statement::Match { expr, arms } => {
                self.check_expression(expr);
                for arm in arms {
                    match &arm.body {
                        ast::MatchBody::Expr(e) => self.check_expression(e),
                        ast::MatchBody::Block(b) => self.check_block(b),
                    }
                }
            }
            ast::Statement::Expression(expr) => {
                self.check_expression(expr);
            }
            ast::Statement::Defer(inner) => {
                self.check_statement(inner);
            }
            ast::Statement::Spawn(expr) => {
                self.check_expression(expr);
            }
            ast::Statement::Yield(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.check_expression(expr);
                }
            }
            ast::Statement::Select { arms } => {
                for arm in arms {
                    self.check_expression(&arm.channel_op);
                    self.check_block(&arm.body);
                }
            }
            // Break / Continue / Pass - no ownership effects.
            _ => {}
        }
    }

    // ------------------------------------------------------------------
    // Expression checking
    // ------------------------------------------------------------------

    pub fn check_expression(&mut self, expr: &ast::Expression) {
        match expr {
            ast::Expression::Identifier(name) => {
                self.check_use(name);
            }
            ast::Expression::Borrow { mutable, expr: inner } => {
                if let ast::Expression::Identifier(ref name) = **inner {
                    self.add_borrow(name, *mutable);
                } else {
                    self.check_expression(inner);
                }
            }
            ast::Expression::Call(callee, args) => {
                self.check_expression(callee);
                for arg in args {
                    self.check_call_arg(arg);
                }
            }
            ast::Expression::MethodCall { receiver, method: _, args } => {
                self.check_expression(receiver);
                for arg in args {
                    self.check_call_arg(arg);
                }
            }
            ast::Expression::Binary(lhs, _op, rhs) => {
                self.check_expression(lhs);
                self.check_expression(rhs);
            }
            ast::Expression::Unary(_op, inner) => {
                self.check_expression(inner);
            }
            ast::Expression::Field(base, _field) => {
                self.check_expression(base);
            }
            ast::Expression::Index(base, idx) => {
                self.check_expression(base);
                self.check_expression(idx);
            }
            ast::Expression::Path(base, _seg) => {
                self.check_expression(base);
            }
            ast::Expression::Array(elems) => {
                for e in elems {
                    self.check_expression(e);
                }
            }
            ast::Expression::Tuple(elems) => {
                for e in elems {
                    self.check_expression(e);
                }
            }
            ast::Expression::StructLiteral { name: _, fields } => {
                for (_fname, fval) in fields {
                    self.check_expression(fval);
                    if let ast::Expression::Identifier(ref src) = fval {
                        self.mark_moved(src);
                    }
                }
            }
            ast::Expression::Deref(inner) => {
                self.check_expression(inner);
            }
            ast::Expression::Await(inner) => {
                self.check_expression(inner);
            }
            ast::Expression::Range { start, end, .. } => {
                if let Some(s) = start { self.check_expression(s); }
                if let Some(e) = end { self.check_expression(e); }
            }
            ast::Expression::Lambda { params: _, body } => {
                self.check_expression(body);
            }
            ast::Expression::If { condition, then_expr, else_expr } => {
                self.check_expression(condition);
                self.check_expression(then_expr);
                if let Some(e) = else_expr {
                    self.check_expression(e);
                }
            }
            ast::Expression::Match { expr: inner, arms } => {
                self.check_expression(inner);
                for arm in arms {
                    match &arm.body {
                        ast::MatchBody::Expr(e) => self.check_expression(e),
                        ast::MatchBody::Block(b) => self.check_block(b),
                    }
                }
            }
            ast::Expression::ListComprehension { expr, var: _, iter, filter } => {
                self.check_expression(iter);
                self.check_expression(expr);
                if let Some(f) = filter {
                    self.check_expression(f);
                }
            }
            ast::Expression::Generator { body } => {
                self.check_block(body);
            }
            ast::Expression::Some(inner) => {
                self.check_expression(inner);
            }
            ast::Expression::Ok(inner) => {
                self.check_expression(inner);
            }
            ast::Expression::Err(inner) => {
                self.check_expression(inner);
            }
            // Literals, None - no ownership effects.
            _ => {}
        }
    }

    // ------------------------------------------------------------------
    // Assignment checking
    // ------------------------------------------------------------------

    pub fn check_assignment(&mut self, target: &ast::Expression, value: &ast::Expression) {
        let loc = self.loc();

        // Check the value side first.
        self.check_expression(value);

        // If assigning an owned value, the source is moved.
        if let ast::Expression::Identifier(ref src) = value {
            self.mark_moved(src);
        }

        // Check the target is mutable.
        if let ast::Expression::Identifier(ref name) = target {
            if let Some(state) = self.variables.get(name) {
                if !state.mutable {
                    self.errors.push(BorrowError::MutationOfImmutable {
                        variable: name.clone(),
                        assign_at: loc,
                    });
                }
                // If the target was previously moved, assignment re-initializes it.
                if state.ownership == OwnershipKind::Moved {
                    if let Some(s) = self.variables.get_mut(name) {
                        s.ownership = OwnershipKind::Own;
                        s.moved_at = None;
                    }
                    return;
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Call argument checking (move semantics for function arguments)
    // ------------------------------------------------------------------

    fn check_call_arg(&mut self, arg: &ast::Expression) {
        match arg {
            ast::Expression::Identifier(ref name) => {
                self.check_use(name);
                // Passing by value moves the argument.
                self.mark_moved(name);
            }
            ast::Expression::Borrow { mutable, expr: inner } => {
                if let ast::Expression::Identifier(ref name) = **inner {
                    self.add_borrow(name, *mutable);
                } else {
                    self.check_expression(inner);
                }
            }
            _ => {
                self.check_expression(arg);
            }
        }
    }

    // ------------------------------------------------------------------
    // Return expression checking (dangling reference detection)
    // ------------------------------------------------------------------

    fn check_return_expression(&mut self, expr: &ast::Expression) {
        let loc = self.loc();
        // If returning a borrow of a local, flag it.
        if let ast::Expression::Borrow { mutable: _, expr: inner } = expr {
            if let ast::Expression::Identifier(ref name) = **inner {
                if let Some(state) = self.variables.get(name) {
                    // If the variable was declared in a non-root scope,
                    // or is a local, it will dangle.
                    if state.declared_scope > 0 || self.scopes.len() > 1 {
                        self.errors.push(BorrowError::ReturnLocalReference {
                            variable: name.clone(),
                            return_at: loc,
                        });
                    }
                }
            }
        }
        // Also detect returning a reference-typed identifier to a local when
        // the function signature says it returns a reference.
        if self.returns_reference {
            if let ast::Expression::Identifier(ref name) = expr {
                if let Some(state) = self.variables.get(name) {
                    if (state.ownership == OwnershipKind::Borrowed
                        || state.ownership == OwnershipKind::MutBorrowed)
                        && state.declared_scope >= 1
                    {
                        self.errors.push(BorrowError::DanglingReference {
                            variable: name.clone(),
                            ref_location: loc,
                        });
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Loop-move checking
    // ------------------------------------------------------------------

    fn check_loop_moves(&mut self) {
        let loc = self.loc();
        for var_name in std::mem::take(&mut self.loop_moves) {
            // Only flag if the variable was declared *outside* the loop scope.
            let is_loop_local = self.scopes.last()
                .map_or(false, |s| s.variables.contains(&var_name));
            if !is_loop_local {
                self.errors.push(BorrowError::MoveInLoop {
                    variable: var_name,
                    move_at: loc,
                });
            }
        }
    }

    // ------------------------------------------------------------------
    // Type helpers
    // ------------------------------------------------------------------

    fn type_is_reference(ty: Option<&ast::Type>) -> bool {
        match ty {
            Some(ast::Type::WithOwnership(_, ast::Ownership::Borrow)) => true,
            Some(ast::Type::WithOwnership(_, ast::Ownership::BorrowMut)) => true,
            _ => false,
        }
    }

    fn type_is_mut_reference(ty: Option<&ast::Type>) -> bool {
        matches!(ty, Some(ast::Type::WithOwnership(_, ast::Ownership::BorrowMut)))
    }

    fn type_is_shared(ty: &ast::Type) -> bool {
        matches!(ty, ast::Type::WithOwnership(_, ast::Ownership::Shared))
    }
}

// ===========================================================================
// Unit tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::*;

    // Helpers to build AST fragments quickly.

    fn ident(name: &str) -> Expression {
        Expression::Identifier(name.to_string())
    }

    fn int_lit(v: i64) -> Expression {
        Expression::Literal(Literal::Int(v))
    }

    fn borrow_expr(name: &str, mutable: bool) -> Expression {
        Expression::Borrow {
            mutable,
            expr: Box::new(ident(name)),
        }
    }

    fn let_stmt(name: &str, mutable: bool, value: Expression) -> Statement {
        Statement::Let {
            name: name.to_string(),
            mutable,
            ty: None,
            value,
        }
    }

    fn assign_stmt(target: &str, value: Expression) -> Statement {
        Statement::Assignment {
            target: ident(target),
            op: None,
            value,
        }
    }

    fn return_stmt(expr: Expression) -> Statement {
        Statement::Return(Some(expr))
    }

    fn expr_stmt(expr: Expression) -> Statement {
        Statement::Expression(expr)
    }

    fn call_expr(func: &str, args: Vec<Expression>) -> Expression {
        Expression::Call(Box::new(ident(func)), args)
    }

    fn make_function(name: &str, stmts: Vec<Statement>) -> Function {
        Function {
            name: name.to_string(),
            is_async: false,
            attributes: vec![],
            params: vec![],
            return_type: None,
            body: Block { statements: stmts },
        }
    }

    fn make_module(funcs: Vec<Function>) -> Module {
        Module {
            items: funcs.into_iter().map(Item::Function).collect(),
        }
    }

    // ---------------------------------------------------------------
    // Test 1: Use after move detection
    // ---------------------------------------------------------------
    #[test]
    fn test_use_after_move() {
        let func = make_function("test", vec![
            let_stmt("x", false, int_lit(42)),
            let_stmt("y", false, ident("x")),      // moves x
            expr_stmt(ident("x")),                  // USE AFTER MOVE
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "expected use-after-move error");
        assert!(matches!(errors[0], BorrowError::UseAfterMove { .. }));
    }

    // ---------------------------------------------------------------
    // Test 2: No error for normal sequential usage
    // ---------------------------------------------------------------
    #[test]
    fn test_valid_sequential_usage() {
        let func = make_function("test", vec![
            let_stmt("x", false, int_lit(1)),
            expr_stmt(ident("x")),
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(errors.is_empty(), "expected no errors, got {:?}", errors);
    }

    // ---------------------------------------------------------------
    // Test 3: Double mutable borrow
    // ---------------------------------------------------------------
    #[test]
    fn test_double_mut_borrow() {
        let func = make_function("test", vec![
            let_stmt("x", true, int_lit(1)),
            let_stmt("a", false, borrow_expr("x", true)),   // &mut x
            let_stmt("b", false, borrow_expr("x", true)),   // &mut x AGAIN -> error
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "expected double-mut-borrow error");
        assert!(matches!(errors[0], BorrowError::DoubleMutBorrow { .. }));
    }

    // ---------------------------------------------------------------
    // Test 4: Mutable borrow while shared borrows exist
    // ---------------------------------------------------------------
    #[test]
    fn test_mut_borrow_while_shared() {
        let func = make_function("test", vec![
            let_stmt("x", true, int_lit(1)),
            let_stmt("a", false, borrow_expr("x", false)),  // &x
            let_stmt("b", false, borrow_expr("x", true)),   // &mut x -> error
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "expected mut-borrow-while-shared error");
        assert!(matches!(errors[0], BorrowError::MutBorrowWhileShared { .. }));
    }

    // ---------------------------------------------------------------
    // Test 5: Shared borrow while mutable borrow exists
    // ---------------------------------------------------------------
    #[test]
    fn test_shared_borrow_while_mut() {
        let func = make_function("test", vec![
            let_stmt("x", true, int_lit(1)),
            let_stmt("a", false, borrow_expr("x", true)),   // &mut x
            let_stmt("b", false, borrow_expr("x", false)),  // &x -> error
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "expected shared-borrow-while-mut error");
        assert!(matches!(errors[0], BorrowError::SharedBorrowWhileMut { .. }));
    }

    // ---------------------------------------------------------------
    // Test 6: Multiple shared borrows are OK
    // ---------------------------------------------------------------
    #[test]
    fn test_multiple_shared_borrows_ok() {
        let func = make_function("test", vec![
            let_stmt("x", false, int_lit(1)),
            let_stmt("a", false, borrow_expr("x", false)),
            let_stmt("b", false, borrow_expr("x", false)),
            let_stmt("c", false, borrow_expr("x", false)),
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(errors.is_empty(), "multiple shared borrows should be fine, got {:?}", errors);
    }

    // ---------------------------------------------------------------
    // Test 7: Moved while borrowed
    // ---------------------------------------------------------------
    #[test]
    fn test_moved_while_borrowed() {
        let func = make_function("test", vec![
            let_stmt("x", false, int_lit(1)),
            let_stmt("r", false, borrow_expr("x", false)),  // &x
            let_stmt("y", false, ident("x")),               // move x -> error
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "expected moved-while-borrowed error");
        assert!(matches!(errors[0], BorrowError::MovedWhileBorrowed { .. }));
    }

    // ---------------------------------------------------------------
    // Test 8: Valid sequential borrows (release then re-borrow)
    // ---------------------------------------------------------------
    #[test]
    fn test_valid_sequential_borrows() {
        // Inner scope borrows x, exits, then outer scope borrows x mutably.
        let func = make_function("test", vec![
            let_stmt("x", true, int_lit(1)),
            // inner scope: borrow &x then release
            Statement::If {
                condition: Expression::Literal(Literal::Bool(true)),
                then_block: Block {
                    statements: vec![
                        let_stmt("r", false, borrow_expr("x", false)),
                    ],
                },
                else_block: None,
            },
            // After scope exit, mut borrow should be fine.
            let_stmt("m", false, borrow_expr("x", true)),
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(errors.is_empty(), "sequential borrows should work, got {:?}", errors);
    }

    // ---------------------------------------------------------------
    // Test 9: Move in loop detection
    // ---------------------------------------------------------------
    #[test]
    fn test_move_in_loop() {
        let func = make_function("test", vec![
            let_stmt("data", false, int_lit(100)),
            Statement::For {
                var: "i".to_string(),
                iter: Expression::Range {
                    start: Some(Box::new(int_lit(0))),
                    end: Some(Box::new(int_lit(10))),
                    inclusive: false,
                },
                body: Block {
                    statements: vec![
                        // Move `data` inside loop - error because next iteration
                        // would try to use it again.
                        let_stmt("tmp", false, ident("data")),
                    ],
                },
            },
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "expected move-in-loop error");
        assert!(matches!(errors.last().unwrap(), BorrowError::MoveInLoop { .. }));
    }

    // ---------------------------------------------------------------
    // Test 10: Return local reference detection
    // ---------------------------------------------------------------
    #[test]
    fn test_return_local_reference() {
        let func = make_function("test", vec![
            let_stmt("local", false, int_lit(42)),
            return_stmt(borrow_expr("local", false)),  // returning &local -> error
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "expected return-local-reference error");
        assert!(matches!(errors[0], BorrowError::ReturnLocalReference { .. }));
    }

    // ---------------------------------------------------------------
    // Test 11: Mutation of immutable variable
    // ---------------------------------------------------------------
    #[test]
    fn test_mutation_of_immutable() {
        let func = make_function("test", vec![
            let_stmt("x", false, int_lit(1)),          // immutable
            assign_stmt("x", int_lit(2)),              // assign -> error
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "expected mutation-of-immutable error");
        assert!(matches!(errors[0], BorrowError::MutationOfImmutable { .. }));
    }

    // ---------------------------------------------------------------
    // Test 12: Mutable variable assignment is OK
    // ---------------------------------------------------------------
    #[test]
    fn test_mutable_assignment_ok() {
        let func = make_function("test", vec![
            let_stmt("x", true, int_lit(1)),
            assign_stmt("x", int_lit(2)),
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(errors.is_empty(), "mutable assignment should be fine, got {:?}", errors);
    }

    // ---------------------------------------------------------------
    // Test 13: Function call moves argument
    // ---------------------------------------------------------------
    #[test]
    fn test_function_call_moves_arg() {
        let func = make_function("test", vec![
            let_stmt("data", false, int_lit(1)),
            expr_stmt(call_expr("consume", vec![ident("data")])),
            expr_stmt(ident("data")),  // use after move
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "expected use-after-move from call");
        assert!(matches!(errors[0], BorrowError::UseAfterMove { .. }));
    }

    // ---------------------------------------------------------------
    // Test 14: Borrow passed to function is OK (no move)
    // ---------------------------------------------------------------
    #[test]
    fn test_borrow_arg_no_move() {
        let func = make_function("test", vec![
            let_stmt("data", false, int_lit(1)),
            expr_stmt(call_expr("read", vec![borrow_expr("data", false)])),
            expr_stmt(ident("data")),  // should still be usable
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        // Only borrow errors, no use-after-move.
        let move_errors: Vec<_> = errors.iter()
            .filter(|e| matches!(e, BorrowError::UseAfterMove { .. }))
            .collect();
        assert!(move_errors.is_empty(), "borrow arg should not move, got {:?}", move_errors);
    }

    // ---------------------------------------------------------------
    // Test 15: Scope-based borrow release
    // ---------------------------------------------------------------
    #[test]
    fn test_scope_borrow_release() {
        // After inner scope exits, borrows from that scope are released.
        let func = make_function("test", vec![
            let_stmt("x", true, int_lit(1)),
            Statement::If {
                condition: Expression::Literal(Literal::Bool(true)),
                then_block: Block {
                    statements: vec![
                        let_stmt("r", false, borrow_expr("x", true)), // &mut x in inner scope
                    ],
                },
                else_block: None,
            },
            // After scope exit, x should be unborrowed.
            assign_stmt("x", int_lit(99)),
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(errors.is_empty(), "borrow should be released after scope exit, got {:?}", errors);
    }

    // ---------------------------------------------------------------
    // Test 16: Re-assign after move re-initializes
    // ---------------------------------------------------------------
    #[test]
    fn test_reinit_after_move() {
        let func = make_function("test", vec![
            let_stmt("x", true, int_lit(1)),
            let_stmt("y", false, ident("x")),    // move x
            assign_stmt("x", int_lit(2)),        // re-init x
            expr_stmt(ident("x")),               // OK now
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        let use_after: Vec<_> = errors.iter()
            .filter(|e| matches!(e, BorrowError::UseAfterMove { .. }))
            .collect();
        assert!(use_after.is_empty(), "re-init should clear moved state, got {:?}", use_after);
    }

    // ---------------------------------------------------------------
    // Test 17: While-loop with use after move
    // ---------------------------------------------------------------
    #[test]
    fn test_while_loop_move() {
        let func = make_function("test", vec![
            let_stmt("data", false, int_lit(1)),
            Statement::While {
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block {
                    statements: vec![
                        let_stmt("tmp", false, ident("data")),
                    ],
                },
            },
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "expected move-in-loop error in while");
    }

    // ---------------------------------------------------------------
    // Test 18: Struct literal moves fields
    // ---------------------------------------------------------------
    #[test]
    fn test_struct_literal_moves_fields() {
        let func = make_function("test", vec![
            let_stmt("name", false, Expression::Literal(Literal::String("hello".into()))),
            expr_stmt(Expression::StructLiteral {
                name: "Person".into(),
                fields: vec![
                    ("name".into(), ident("name")),
                ],
            }),
            expr_stmt(ident("name")),  // use after move
        ]);
        let module = make_module(vec![func]);
        let errors = BorrowChecker::check_module(&module);
        assert!(!errors.is_empty(), "struct literal should move field values");
        assert!(matches!(errors[0], BorrowError::UseAfterMove { .. }));
    }

    // ---------------------------------------------------------------
    // Test 19: Module-level check across multiple functions
    // ---------------------------------------------------------------
    #[test]
    fn test_module_multiple_functions() {
        let f1 = make_function("ok_func", vec![
            let_stmt("a", false, int_lit(1)),
            expr_stmt(ident("a")),
        ]);
        let f2 = make_function("bad_func", vec![
            let_stmt("b", false, int_lit(2)),
            let_stmt("c", false, ident("b")),
            expr_stmt(ident("b")),  // use after move
        ]);
        let module = make_module(vec![f1, f2]);
        let errors = BorrowChecker::check_module(&module);
        assert_eq!(errors.len(), 1, "only bad_func should have an error");
    }

    // ---------------------------------------------------------------
    // Test 20: BorrowError Display formatting
    // ---------------------------------------------------------------
    #[test]
    fn test_error_display() {
        let err = BorrowError::UseAfterMove {
            variable: "x".into(),
            moved_at: Location::new(1),
            used_at: Location::new(3),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("moved value"));
        assert!(msg.contains("x"));
    }
}
