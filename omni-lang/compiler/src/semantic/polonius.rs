//! Polonius-based Borrow Checker for Omni v2.0
//!
//! This module implements Polonius fact generation and runs the `polonius-engine`
//! to compute precise borrow diagnostics. It generates facts for variables,
//! paths, borrows, moves and control-flow (CFG edges) so Polonius can reason
//! precisely about complex control flow (if/else, loops, matches).

use crate::parser::ast;
use crate::semantic::borrow_check::{BorrowError, Location};
use polonius_engine::{AllFacts, Algorithm, Output, Atom, FactTypes};
use std::collections::HashMap;
use std::env;

// ---------------------------------------------------------------------------
// Lightweight atom type used for Polonius facts (wraps usize)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Id(usize);

impl From<usize> for Id {
    fn from(v: usize) -> Self {
        Id(v)
    }
}

impl From<Id> for usize {
    fn from(id: Id) -> Self {
        id.0
    }
}

impl Atom for Id {
    fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
struct FactsTypes;

impl FactTypes for FactsTypes {
    type Origin = Id;
    type Loan = Id;
    type Point = Id;
    type Variable = Id;
    type Path = Id;
}

// ---------------------------------------------------------------------------
// Auxiliary bookkeeping to map polonius facts back to human errors
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct LoanInfo {
    var: String,
    field: Option<String>,
    mutable: bool,
    origin_point: usize,
}

#[derive(Clone, Debug)]
struct PathInfo {
    var: String,
}

// ---------------------------------------------------------------------------
// Fact builder: traverses AST, creates points and facts
// ---------------------------------------------------------------------------

struct FactBuilder {
    facts: AllFacts<FactsTypes>,
    // Separate id counters to avoid accidental overlap across atom domains.
    next_point: usize,
    next_var: usize,
    next_path: usize,
    next_origin: usize,
    next_loan: usize,

    // variable / path maps
    var_map: HashMap<String, Id>,
    path_map: HashMap<String, Id>,

    // loan/origin bookkeeping
    loan_info: HashMap<Id, LoanInfo>,

    // path metadata
    path_info: HashMap<Id, PathInfo>,
    // stable child path map: (var, field) -> path id
    child_path_map: HashMap<(String, String), Id>,
    // record explicit moves: path -> list of (point, inside_loop)
    path_moves: HashMap<Id, Vec<(usize, bool)>>,

    // record borrow events in order for heuristics
    borrow_events: Vec<(String, bool, usize, Id)>, // (var, mutable, point, loan)

    // declaration points for variables
    var_decl_point: HashMap<String, usize>,
    // mutability map for variables (true if `mut`)
    var_mutable: HashMap<String, bool>,
    // list of loans issued (in chronological order) to support scope-based kills
    issued_loans: Vec<Id>,

    // loop depth tracking
    loop_depth: usize,
}

impl FactBuilder {
    fn new() -> Self {
        Self {
            facts: AllFacts::default(),
            next_point: 1,
            next_var: 1_000_000,
            next_path: 2_000_000,
            next_origin: 3_000_000,
            next_loan: 4_000_000,
            var_map: HashMap::new(),
            path_map: HashMap::new(),
            loan_info: HashMap::new(),
            path_info: HashMap::new(),
            child_path_map: HashMap::new(),
            path_moves: HashMap::new(),
            borrow_events: Vec::new(),
            var_decl_point: HashMap::new(),
            var_mutable: HashMap::new(),
            issued_loans: Vec::new(),
            loop_depth: 0,
        }
    }

    fn new_point(&mut self) -> Id {
        let id = Id(self.next_point);
        self.next_point += 1;
        id
    }

    fn new_var_id(&mut self) -> Id {
        let id = Id(self.next_var);
        self.next_var += 1;
        id
    }

    fn new_path_id(&mut self) -> Id {
        let id = Id(self.next_path);
        self.next_path += 1;
        id
    }

    fn ensure_var(&mut self, name: &str, decl_point: Option<usize>) -> Id {
        if let Some(id) = self.var_map.get(name) {
            return *id;
        }
        let vid = self.new_var_id();
        self.var_map.insert(name.to_string(), vid);

        // create a path representing the whole variable
        let pid = self.new_path_id();
        self.path_map.insert(name.to_string(), pid);
        self.path_info.insert(pid, PathInfo { var: name.to_string() });

        // path_is_var(path, var)
        self.facts.path_is_var.push((pid, vid));

        if let Some(p) = decl_point {
            self.var_decl_point.insert(name.to_string(), p);
        }

        vid
    }

    fn new_origin(&mut self) -> Id {
        let id = Id(self.next_origin);
        self.next_origin += 1;
        id
    }

    fn new_loan(&mut self) -> Id {
        let id = Id(self.next_loan);
        self.next_loan += 1;
        id
    }

    fn add_cfg_edge(&mut self, a: Id, b: Id) {
        self.facts.cfg_edge.push((a, b));
    }

    fn add_var_used(&mut self, var: Id, point: Id) {
        self.facts.var_used_at.push((var, point));
    }

    fn add_var_defined(&mut self, var: Id, point: Id) {
        self.facts.var_defined_at.push((var, point));
    }

    fn add_path_moved(&mut self, path: Id, point: Id) {
        self.facts.path_moved_at_base.push((path, point));
        let entry = self.path_moves.entry(path).or_default();
        entry.push((usize::from(point), self.loop_depth > 0));
    }

    fn add_path_assigned(&mut self, path: Id, point: Id) {
        self.facts.path_assigned_at_base.push((path, point));
    }

    fn add_path_accessed(&mut self, path: Id, point: Id) {
        self.facts.path_accessed_at_base.push((path, point));
    }

    fn add_loan_issued(&mut self, origin: Id, loan: Id, point: Id) {
        self.facts.loan_issued_at.push((origin, loan, point));
        self.issued_loans.push(loan);
    }

    fn add_child_path(&mut self, child: Id, parent: Id) {
        self.facts.child_path.push((child, parent));
    }

    fn get_or_create_child_path(&mut self, var: &str, field: &str, parent: Id) -> Id {
        let key = (var.to_string(), field.to_string());
        if let Some(id) = self.child_path_map.get(&key) {
            *id
        } else {
            let id = self.new_path_id();
            self.child_path_map.insert(key, id);
            self.add_child_path(id, parent);
            id
        }
    }

    fn record_loan_info(&mut self, loan: Id, var: &str, field: Option<String>, mutable: bool, origin_point: usize) {
        self.loan_info.insert(
            loan,
            LoanInfo {
                var: var.to_string(),
                field,
                mutable,
                origin_point,
            },
        );
    }

    // Build facts for a function body and return mapping data used later.
    fn build_for_function(&mut self, func: &ast::Function) {
        // treat function entry as point 0
        let entry = self.new_point();

        // Bind parameters as declared at entry.
        for param in &func.params {
            let is_borrow = Self::type_is_reference(Some(&param.ty));
            let var_id = self.ensure_var(&param.name, Some(usize::from(entry)));
            if is_borrow {
                // Mark origin placeholder: use placeholder facts so polonius knows refs are live
                let origin = self.new_origin();
                // use_of_var_derefs_origin(var, origin)
                self.facts.use_of_var_derefs_origin.push((var_id, origin));
                // placeholder(origin, loan) - create loan placeholder mapping
                let loan = self.new_loan();
                self.facts.placeholder.push((origin, loan));
            }
        }

        // Process function body statements and create a linear CFG with branch edges.
        let (maybe_entry, maybe_exit) = self.process_block(&func.body);

        // Compute a canonical exit point for the function so we can record drops.
        let exit = if maybe_entry.is_none() {
            // no statements: create a fresh exit and connect entry -> exit
            let ex = self.new_point();
            self.add_cfg_edge(entry, ex);
            ex
        } else if let (Some(e), Some(x)) = (maybe_entry, maybe_exit) {
            // connect function entry to first block entry
            self.add_cfg_edge(entry, e);
            x
        } else if let (Some(e), None) = (maybe_entry, maybe_exit) {
            // a block that had an entry but no explicit exit: create exit and connect
            let ex = self.new_point();
            if let Some(last) = maybe_exit {
                self.add_cfg_edge(last, ex);
            }
            self.add_cfg_edge(entry, e);
            ex
        } else {
            // fallback
            let ex = self.new_point();
            self.add_cfg_edge(entry, ex);
            ex
        };

        // Mark universal region placeholder to avoid incomplete placeholder errors
        // (keeps polonius happy for small inputs)
        // Not strictly necessary, but harmless.
        for _ in 0..1 {
            let origin = self.new_origin();
            self.facts.universal_region.push(origin);
        }

        // Ensure root-path assignments are recorded: when variables were declared
        // we already recorded var_defined; also record path_assigned for root paths.
        let var_entries: Vec<(String, Id)> = self
            .var_map
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        for (name, vid) in var_entries.iter() {
            if let Some(pid) = self.path_map.get(name) {
                // use the function exit as the drop point for conservative liveness
                self.facts.var_dropped_at.push((*vid, exit));
                // Also treat initial binding as an assignment at its declared point
                if let Some(decl_pt) = self.var_decl_point.get(name) {
                    let p = Id(*decl_pt);
                    self.add_path_assigned(*pid, p);
                }
            }
        }

        // Compute loan_killed_at and loan_invalidated_at heuristically by scanning
        // assigned and moved paths and matching them to recorded loans.
        // Build maps for quick lookup: path -> points assigned/moved
        let mut assigned_map: HashMap<Id, Vec<Id>> = HashMap::new();
        for (path, point) in &self.facts.path_assigned_at_base {
            assigned_map.entry(*path).or_default().push(*point);
        }
        let mut moved_map: HashMap<Id, Vec<Id>> = HashMap::new();
        for (path, point) in &self.facts.path_moved_at_base {
            moved_map.entry(*path).or_default().push(*point);
        }

        for (loan, info) in &self.loan_info {
            // find relevant paths: root var path
            let root_pid = self.path_map.get(&info.var);
            if root_pid.is_none() {
                continue;
            }
            let root_pid = *root_pid.unwrap();

            // if loan targets a specific field, resolve child path id
            let candidate_paths: Vec<Id> = if let Some(field_name) = &info.field {
                if let Some(child_pid) = self.child_path_map.get(&(info.var.clone(), field_name.clone())) {
                    vec![*child_pid]
                } else {
                    vec![]
                }
            } else {
                // whole-variable loan: consider root and all recorded child paths
                let mut v = vec![root_pid];
                for (k, pid) in &self.child_path_map {
                    if k.0 == info.var {
                        v.push(*pid);
                    }
                }
                v
            };

            // For each candidate path, if it was assigned or moved at some point,
            // mark the loan as killed/invalidated at those points.
            for pid in candidate_paths {
                if let Some(points) = assigned_map.get(&pid) {
                    for p in points {
                        self.facts.loan_killed_at.push((*loan, *p));
                        self.facts.loan_invalidated_at.push((*p, *loan));
                    }
                }
                if let Some(points) = moved_map.get(&pid) {
                    for p in points {
                        self.facts.loan_killed_at.push((*loan, *p));
                        self.facts.loan_invalidated_at.push((*p, *loan));
                    }
                }
            }
        }

        // -----------------------------------------------------------------
        // Heuristic subset_base generation:
        // Emit subset relations between placeholder / origin ids when we can
        // reasonably infer that one origin is derived from another. This
        // includes: field-origin -> root-origin, and later-origin -> earlier-origin
        // for the same variable. These heuristics help Polonius reason about
        // origin containment for borrowed projections in control-flow.
        // -----------------------------------------------------------------
        let loan_issued = self.facts.loan_issued_at.clone();
        // Build (origin -> (loan, point)) map for quick lookup
        let mut origin_map: HashMap<Id, (Id, Id)> = HashMap::new();
        for (origin, loan, point) in loan_issued.iter() {
            origin_map.insert(*origin, (*loan, *point));
        }

        // Iterate pairs of issued origins and create subset edges when:
        // - same variable and parent has no field while child has a field
        // - same variable and parent origin was issued earlier (conservative)
        let issued_pairs = loan_issued;
        for (child_origin, child_loan, child_point) in issued_pairs.iter() {
            if let Some(child_info) = self.loan_info.get(child_loan) {
                for (parent_origin, parent_loan, _parent_point) in issued_pairs.iter() {
                    if child_origin == parent_origin {
                        continue;
                    }
                    if let Some(parent_info) = self.loan_info.get(parent_loan) {
                        if child_info.var == parent_info.var {
                            // field -> root
                            if child_info.field.is_some() && parent_info.field.is_none() {
                                self.facts.subset_base.push((*child_origin, *parent_origin, *child_point));
                                continue;
                            }

                            // later origin may be a subset of earlier origin
                            if child_info.origin_point >= parent_info.origin_point {
                                self.facts.subset_base.push((*child_origin, *parent_origin, *child_point));
                                continue;
                            }
                        }
                    }
                }
            }
        }

        // For every variable that has use_of_var_derefs_origin facts, mark
        // drop_of_var_derefs_origin at function exit conservatively.
        // Clone the relation before mutating facts to avoid double-borrowing `self`.
        let use_of_var_clone = self.facts.use_of_var_derefs_origin.clone();
        for (var, origin) in use_of_var_clone.iter() {
            self.facts.drop_of_var_derefs_origin.push((*var, *origin));
        }
    }

    fn process_block(&mut self, block: &ast::Block) -> (Option<Id>, Option<Id>) {
        let mut first: Option<Id> = None;
        let mut last: Option<Id> = None;
        for stmt in &block.statements {
            let (entry, exit, terminates) = self.process_statement(stmt);
            if first.is_none() {
                first = Some(entry);
            }
            if let Some(prev) = last {
                // do not connect prev -> entry if prev was a terminating return
                self.add_cfg_edge(prev, entry);
            }
            last = Some(exit);
            if terminates {
                // after a terminator we stop connecting following statements
                break;
            }
        }
        (first, last)
    }

    fn process_statement(&mut self, stmt: &ast::Statement) -> (Id, Id, bool) {
        match stmt {
            ast::Statement::Let { name, mutable, ty: _, value } => {
                let point = self.new_point();
                // if value exists, process expression first
                if let Some(val) = value {
                    self.process_expression(val, usize::from(point));
                    if let ast::Expression::Identifier(src) = val {
                        // move from src
                        let pid = self.path_for_var(src);
                        self.add_path_moved(pid, point);
                    }
                }
                // declare variable at this point
                self.ensure_var(name, Some(usize::from(point)));
                // record mutability
                self.var_mutable.insert(name.clone(), *mutable);
                        // variable assigned/defined here
                        let vid = self.var_map.get(name).copied().unwrap();
                        self.add_var_defined(vid, point);
                        if let Some(pid) = self.path_map.get(name) {
                            self.add_path_assigned(*pid, point);
                        }
                (point, point, false)
            }
            ast::Statement::Var { name, value, .. } => {
                let point = self.new_point();
                if let Some(val) = value {
                    self.process_expression(val, usize::from(point));
                    if let ast::Expression::Identifier(src) = val {
                        let pid = self.path_for_var(src);
                        self.add_path_moved(pid, point);
                    }
                }
                self.ensure_var(name, Some(usize::from(point)));
                // Var does not carry explicit mutability in the AST; treat as mutable by default
                self.var_mutable.insert(name.clone(), true);
                let vid = self.var_map.get(name).copied().unwrap();
                self.add_var_defined(vid, point);
                if let Some(pid) = self.path_map.get(name) {
                    self.add_path_assigned(*pid, point);
                }
                (point, point, false)
            }
            ast::Statement::Assignment { target, op: _, value } => {
                let point = self.new_point();
                self.process_expression(value, usize::from(point));
                // if assigning from ident, that is a move
                if let ast::Expression::Identifier(src) = value {
                    let pid = self.path_for_var(src);
                    self.add_path_moved(pid, point);
                }
                // target may be identifier
                if let ast::Expression::Identifier(name) = target {
                    let vid = self.ensure_var(name, None);
                    self.add_var_defined(vid, point);
                    if let Some(pid) = self.path_map.get(name) {
                        self.add_path_assigned(*pid, point);
                    }
                } else if let ast::Expression::Field(base, field) = target {
                    if let ast::Expression::Identifier(varname) = base.as_ref() {
                            let pid = self.path_for_var(varname);
                            // create or reuse a stable child path id for this field
                            let child_id = self.get_or_create_child_path(varname, field.as_str(), pid);
                            self.path_info.insert(child_id, PathInfo { var: varname.clone() });
                            self.add_path_assigned(child_id, point);
                        }
                }
                (point, point, false)
            }
            ast::Statement::Return(expr_opt) => {
                let point = self.new_point();
                if let Some(expr) = expr_opt {
                    // If returning a borrow of a local, emit ReturnLocalReference directly.
                    if let ast::Expression::Borrow { mutable: _, expr: inner } = expr {
                        if let ast::Expression::Identifier(ref name) = inner.as_ref() {
                            if let Some(decl) = self.var_decl_point.get(name) {
                                // if declared in nested scope (decl_point > 0) treat as local
                                if *decl > 0 {
                                    // Record direct error (returning local borrow)
                                    // We'll append this to loan-based errors later; for now we record
                                    // a loan invalidation fact so polonius also sees it.
                                    let vid = self.ensure_var(name, None);
                                    self.facts.var_used_at.push((vid, point));
                                }
                            }
                        }
                    }
                    self.process_expression(expr, usize::from(point));
                }
                // terminating statement
                (point, point, true)
            }
            ast::Statement::If { condition, then_block, else_block } => {
                // condition point
                let cond_point = self.new_point();
                self.process_expression(condition, usize::from(cond_point));

                // then block
                let then_start_loans = self.issued_loans.len();
                let (then_entry, then_exit) = self.process_block(then_block);

                // else block
                let else_start_loans = self.issued_loans.len();
                let (else_entry, else_exit) = if let Some(eb) = else_block {
                    self.process_block(eb)
                } else {
                    (None, None)
                };

                // follow point after the if
                let follow = self.new_point();

                // edges: cond -> then_entry | cond -> else_entry/ follow
                if let Some(te) = then_entry {
                    self.add_cfg_edge(cond_point, te);
                    if let Some(te_end) = then_exit {
                        self.add_cfg_edge(te_end, follow);
                    } else {
                        self.add_cfg_edge(cond_point, follow);
                    }
                } else {
                    self.add_cfg_edge(cond_point, follow);
                }

                if let Some(ee) = else_entry {
                    self.add_cfg_edge(cond_point, ee);
                    if let Some(ee_end) = else_exit {
                        self.add_cfg_edge(ee_end, follow);
                    } else {
                        self.add_cfg_edge(cond_point, follow);
                    }
                } else {
                    // no else -> cond may go to follow
                    self.add_cfg_edge(cond_point, follow);
                }

                // After constructing the follow point, mark loans that were issued
                // inside the then/else blocks as killed/invalidated at the follow.
                let follow_pt = follow;
                // then-block loans
                let then_new = self.issued_loans.len();
                if then_new > then_start_loans {
                    for loan in &self.issued_loans[then_start_loans..then_new] {
                        self.facts.loan_killed_at.push((*loan, follow_pt));
                        self.facts.loan_invalidated_at.push((follow_pt, *loan));
                    }
                }
                // else-block loans
                let else_new = self.issued_loans.len();
                if else_new > else_start_loans {
                    for loan in &self.issued_loans[else_start_loans..else_new] {
                        self.facts.loan_killed_at.push((*loan, follow_pt));
                        self.facts.loan_invalidated_at.push((follow_pt, *loan));
                    }
                }

                (cond_point, follow, false)
            }
            ast::Statement::For { var, iter, body } => {
                let iter_point = self.new_point();
                self.process_expression(iter, usize::from(iter_point));

                // body
                self.loop_depth += 1;
                let body_start_loans = self.issued_loans.len();
                let (body_entry, body_exit) = self.process_block(body);
                self.loop_depth -= 1;

                let follow = self.new_point();

                if let Some(be) = body_entry {
                    self.add_cfg_edge(iter_point, be);
                    if let Some(bx) = body_exit {
                        // loop back
                        self.add_cfg_edge(bx, iter_point);
                    }
                    // also from iter -> follow (empty loop)
                    self.add_cfg_edge(iter_point, follow);
                } else {
                    self.add_cfg_edge(iter_point, follow);
                }

                // kill loans created inside the loop body at follow point (scope exit)
                let follow_pt = follow;
                let body_new = self.issued_loans.len();
                if body_new > body_start_loans {
                    for loan in &self.issued_loans[body_start_loans..body_new] {
                        self.facts.loan_killed_at.push((*loan, follow_pt));
                        self.facts.loan_invalidated_at.push((follow_pt, *loan));
                    }
                }

                // declare loop variable at iter_point
                self.ensure_var(var, Some(usize::from(iter_point)));

                (iter_point, follow, false)
            }
            ast::Statement::While { condition, body } => {
                let cond_point = self.new_point();
                self.process_expression(condition, usize::from(cond_point));
                self.loop_depth += 1;
                let body_start_loans = self.issued_loans.len();
                let (body_entry, body_exit) = self.process_block(body);
                self.loop_depth -= 1;
                let follow = self.new_point();
                if let Some(be) = body_entry {
                    self.add_cfg_edge(cond_point, be);
                    if let Some(bx) = body_exit {
                        self.add_cfg_edge(bx, cond_point);
                    }
                    self.add_cfg_edge(cond_point, follow);
                } else {
                    self.add_cfg_edge(cond_point, follow);
                }
                // kill loans created inside the loop body at follow point
                let follow_pt = follow;
                let body_new = self.issued_loans.len();
                if body_new > body_start_loans {
                    for loan in &self.issued_loans[body_start_loans..body_new] {
                        self.facts.loan_killed_at.push((*loan, follow_pt));
                        self.facts.loan_invalidated_at.push((follow_pt, *loan));
                    }
                }
                (cond_point, follow, false)
            }
            ast::Statement::Loop { body } => {
                let entry = self.new_point();
                self.loop_depth += 1;
                let body_start_loans = self.issued_loans.len();
                let (body_entry, body_exit) = self.process_block(body);
                self.loop_depth -= 1;
                if let Some(be) = body_entry {
                    self.add_cfg_edge(entry, be);
                    if let Some(bx) = body_exit {
                        self.add_cfg_edge(bx, entry);
                    }
                }
                // follow point (infinite loop may not reach it, but keep for CFG closure)
                let follow = self.new_point();
                self.add_cfg_edge(entry, follow);
                // kill loans created inside the loop body at follow point
                let follow_pt = follow;
                let body_new = self.issued_loans.len();
                if body_new > body_start_loans {
                    for loan in &self.issued_loans[body_start_loans..body_new] {
                        self.facts.loan_killed_at.push((*loan, follow_pt));
                        self.facts.loan_invalidated_at.push((follow_pt, *loan));
                    }
                }
                (entry, follow, false)
            }
            ast::Statement::Match { expr, arms } => {
                let cond_point = self.new_point();
                self.process_expression(expr, usize::from(cond_point));
                let follow = self.new_point();
                for arm in arms {
                    match &arm.body {
                        ast::MatchBody::Expr(e) => {
                            let (ent, ex, term) = self.process_statement(&ast::Statement::Expression(e.clone()));
                            self.add_cfg_edge(cond_point, ent);
                            if !term {
                                self.add_cfg_edge(ex, follow);
                            }
                        }
                        ast::MatchBody::Block(b) => {
                            let arm_start_loans = self.issued_loans.len();
                            let (be, bx) = self.process_block(b);
                            if let Some(be) = be {
                                self.add_cfg_edge(cond_point, be);
                                if let Some(bx) = bx {
                                    self.add_cfg_edge(bx, follow);
                                }
                            }
                            // kill loans created inside this arm at follow
                            let arm_new = self.issued_loans.len();
                            if arm_new > arm_start_loans {
                                for loan in &self.issued_loans[arm_start_loans..arm_new] {
                                    self.facts.loan_killed_at.push((*loan, follow));
                                    self.facts.loan_invalidated_at.push((follow, *loan));
                                }
                            }
                        }
                    }
                }
                (cond_point, follow, false)
            }
            ast::Statement::Expression(expr) => {
                let point = self.new_point();
                self.process_expression(expr, usize::from(point));
                (point, point, false)
            }
            _ => {
                // Default: create a point and traverse children conservatively.
                let point = self.new_point();
                // best-effort: try to walk contained expressions
                match stmt {
                    ast::Statement::Defer(inner) => {
                        self.process_statement(inner);
                    }
                    ast::Statement::Spawn(expr) => {
                        self.process_expression(expr, usize::from(point));
                    }
                    ast::Statement::Yield(opt) => {
                        if let Some(e) = opt {
                            self.process_expression(e, usize::from(point));
                        }
                    }
                    ast::Statement::Select { arms } => {
                        for arm in arms {
                            self.process_expression(&arm.channel_op, usize::from(point));
                            self.process_block(&arm.body);
                        }
                    }
                    _ => {}
                }
                (point, point, false)
            }
        }
    }

    fn process_expression(&mut self, expr: &ast::Expression, point_idx: usize) {
        let p = Id(point_idx);
        match expr {
            ast::Expression::Identifier(name) => {
                let var_id = self.ensure_var(name, None);
                self.add_var_used(var_id, p);
                // also path access
                if let Some(pid) = self.path_map.get(name) {
                    self.add_path_accessed(*pid, p);
                }
            }
            ast::Expression::Borrow { mutable, expr } => {
                match expr.as_ref() {
                    ast::Expression::Identifier(name) => {
                        self.ensure_var(name, None);
                        let origin = self.new_origin();
                        let loan = self.new_loan();
                        self.add_loan_issued(origin, loan, p);
                        self.record_loan_info(loan, name, None, *mutable, point_idx);
                        self.borrow_events.push((name.to_string(), *mutable, point_idx, loan));
                    }
                    ast::Expression::Field(base, field) => {
                        if let ast::Expression::Identifier(varname) = base.as_ref() {
                            let pid = self.path_for_var(varname);
                            // create or reuse a stable child path id for this field
                            let child = self.new_path_id();
                            self.add_child_path(child, pid);
                            self.path_info.insert(child, PathInfo { var: varname.clone() });
                            let origin = self.new_origin();
                            let loan = self.new_loan();
                            self.add_loan_issued(origin, loan, p);
                            self.record_loan_info(loan, varname, Some(field.clone()), *mutable, point_idx);
                            self.borrow_events.push((varname.clone(), *mutable, point_idx, loan));
                        }
                    }
                    _ => {
                        // Recursively process inner expression
                        self.process_expression(expr, point_idx);
                    }
                }
            }
            ast::Expression::Call(callee, args) => {
                self.process_expression(callee, point_idx);
                for arg in args {
                    // if arg is identifier -> moved
                    if let ast::Expression::Identifier(name) = arg {
                        let pid = self.path_for_var(name);
                        self.add_path_moved(pid, p);
                    }
                    self.process_expression(arg, point_idx);
                }
            }
            ast::Expression::MethodCall { receiver, args, .. } => {
                self.process_expression(receiver, point_idx);
                for arg in args {
                    if let ast::Expression::Identifier(name) = arg {
                        let pid = self.path_for_var(name);
                        self.add_path_moved(pid, p);
                    }
                    self.process_expression(arg, point_idx);
                }
            }
            ast::Expression::StructLiteral { fields, .. } => {
                    for (_fname, val) in fields {
                    if let ast::Expression::Identifier(name) = val {
                        let pid = self.path_for_var(name);
                        self.add_path_moved(pid, p);
                    }
                    self.process_expression(val, point_idx);
                }
            }
            ast::Expression::Field(base, field) => {
                // If base is an identifier, record an accessed child path for the field
                    if let ast::Expression::Identifier(varname) = base.as_ref() {
                    let pid = self.path_for_var(varname);
                    let child = self.get_or_create_child_path(varname, field, pid);
                    self.add_path_accessed(child, p);
                }
                self.process_expression(base, point_idx);
            }
            ast::Expression::Binary(lhs, _, rhs) => {
                self.process_expression(lhs, point_idx);
                self.process_expression(rhs, point_idx);
            }
            ast::Expression::Unary(_, inner) => {
                self.process_expression(inner, point_idx);
            }
            ast::Expression::Index(base, idx) => {
                self.process_expression(base, point_idx);
                self.process_expression(idx, point_idx);
            }
            ast::Expression::Match { expr, arms } => {
                self.process_expression(expr, point_idx);
                for arm in arms {
                    match &arm.body {
                        ast::MatchBody::Expr(e) => self.process_expression(e, point_idx),
                        ast::MatchBody::Block(b) => {
                            self.process_block(b);
                        }
                    }
                }
            }
            ast::Expression::Generator { body } => {
                self.process_block(body);
            }
            ast::Expression::Some(inner)
            | ast::Expression::Ok(inner)
            | ast::Expression::Err(inner)
            | ast::Expression::Await(inner)
            | ast::Expression::Deref(inner)
            | ast::Expression::Lambda { body: inner, .. }
                => self.process_expression(inner, point_idx),
            ast::Expression::Array(elems) | ast::Expression::Tuple(elems) => {
                for e in elems {
                    self.process_expression(e, point_idx);
                }
            }
            _ => {}
        }
    }

    fn path_for_var(&mut self, name: &str) -> Id {
        if let Some(pid) = self.path_map.get(name) {
            *pid
        } else {
            self.ensure_var(name, None);
            self.path_map.get(name).copied().unwrap()
        }
    }

    fn type_is_reference(ty: Option<&ast::Type>) -> bool {
        match ty {
            Some(ast::Type::WithOwnership(_, ast::Ownership::Borrow)) => true,
            Some(ast::Type::WithOwnership(_, ast::Ownership::BorrowMut)) => true,
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Run Polonius and map results back to BorrowError
// ---------------------------------------------------------------------------

// Lightweight public checker wrapper used by tests and integration points.
// Provides a minimal constructor so existing tests that expect a
// `PoloniusChecker::new()` symbol continue to compile.
pub struct PoloniusChecker;

impl PoloniusChecker {
    pub fn new() -> Self {
        PoloniusChecker
    }
}


pub fn run_polonius(module: &ast::Module) -> Vec<BorrowError> {
    let mut all_errors = Vec::new();

    for item in &module.items {
        if let ast::Item::Function(func) = item {
            let mut builder = FactBuilder::new();
            builder.build_for_function(func);

            // Debug dump of generated facts (opt-in via POLONIUS_DEBUG)
            if env::var("POLONIUS_DEBUG").is_ok() {
                eprintln!("=== POLONIUS DEBUG: function {} ===", func.name);
                eprintln!("var_map: {:?}", builder.var_map);
                eprintln!("path_map: {:?}", builder.path_map);
                eprintln!("loan_info: {:?}", builder.loan_info);
                eprintln!("borrow_events: {:?}", builder.borrow_events);
                eprintln!("path_moves: {:?}", builder.path_moves);
                eprintln!("facts counts: loan_issued_at={}, path_moved_at_base={}, path_assigned_at_base={}, path_accessed_at_base={}",
                    builder.facts.loan_issued_at.len(),
                    builder.facts.path_moved_at_base.len(),
                    builder.facts.path_assigned_at_base.len(),
                    builder.facts.path_accessed_at_base.len()
                );
                eprintln!("loan_killed_at: {:?}", builder.facts.loan_killed_at);
                eprintln!("loan_invalidated_at: {:?}", builder.facts.loan_invalidated_at);
            }

            // Run Polonius
            let output = Output::<FactsTypes>::compute(&builder.facts, Algorithm::DatafrogOpt, false);

            if env::var("POLONIUS_DEBUG").is_ok() {
                eprintln!("Polonius output.errors: {:?}", output.errors);
                eprintln!("Polonius output.move_errors: {:?}", output.move_errors);
            }

            // Map move errors -> MoveInLoop / MovedWhileBorrowed / UseAfterMove
            for (point, paths) in &output.move_errors {
                let p_idx: usize = (*point).into();
                for path in paths {
                    if let Some(info) = builder.path_info.get(path) {
                        if let Some(moves) = builder.path_moves.get(path) {
                            if let Some((m, inside_loop)) = moves.iter().find(|(pp, _)| *pp == p_idx) {
                                if *inside_loop {
                                    all_errors.push(BorrowError::MoveInLoop {
                                        variable: info.var.clone(),
                                        move_at: Location::new(*m),
                                    });
                                } else {
                                    all_errors.push(BorrowError::MovedWhileBorrowed {
                                        variable: info.var.clone(),
                                        borrow_at: Location::new(0),
                                        move_at: Location::new(*m),
                                    });
                                }
                            } else if let Some((m, _)) = moves.iter().find(|(pp, _)| *pp < p_idx) {
                                // A move happened earlier than the reported Polonius move error
                                // — treat as UseAfterMove (used after an earlier move) conservatively.
                                all_errors.push(BorrowError::UseAfterMove {
                                    variable: info.var.clone(),
                                    moved_at: Location::new(*m),
                                    used_at: Location::new(p_idx),
                                });
                            }
                        }
                    }
                }
            }

            // Map loan errors
            for (point, loans) in &output.errors {
                let p_idx: usize = (*point).into();
                for loan in loans {
                    if let Some(info) = builder.loan_info.get(loan) {
                        // 1) If a move occurred at this point on the same variable -> MovedWhileBorrowed
                        if let Some(pid) = builder.path_map.get(&info.var) {
                            if let Some(moves) = builder.path_moves.get(pid) {
                                if moves.iter().any(|(pp, _)| *pp == p_idx) {
                                    all_errors.push(BorrowError::MovedWhileBorrowed {
                                        variable: info.var.clone(),
                                        borrow_at: Location::new(info.origin_point),
                                        move_at: Location::new(p_idx),
                                    });
                                    continue;
                                }
                                // If a prior move exists -> UseAfterMove
                                if let Some((m, _)) = moves.iter().find(|(pp, _)| *pp < p_idx) {
                                    all_errors.push(BorrowError::UseAfterMove {
                                        variable: info.var.clone(),
                                        moved_at: Location::new(*m),
                                        used_at: Location::new(p_idx),
                                    });
                                    continue;
                                }
                            }
                        }

                        // 2) Borrow conflict heuristics: double mutable, mut while shared, shared while mut
                        // Field-aware: only consider conflicts when the affected fields overlap
                        // (same field name) or when either loan targets the whole variable.
                        let this_field = info.field.clone();
                        if info.mutable {
                            if let Some(first) = builder.borrow_events.iter().find(|(v, m, pt, _)| v == &info.var && *m && *pt < info.origin_point) {
                                let prev_field = builder.loan_info.get(&first.3).and_then(|li| li.field.clone());
                                if prev_field.is_none() || this_field.is_none() || prev_field == this_field {
                                    all_errors.push(BorrowError::DoubleMutBorrow {
                                        variable: info.var.clone(),
                                        first_borrow: Location::new(first.2),
                                        second_borrow: Location::new(info.origin_point),
                                    });
                                    continue;
                                }
                            }
                            if let Some(shared) = builder.borrow_events.iter().find(|(v, m, pt, _)| v == &info.var && !*m && *pt < info.origin_point) {
                                let prev_field = builder.loan_info.get(&shared.3).and_then(|li| li.field.clone());
                                if prev_field.is_none() || this_field.is_none() || prev_field == this_field {
                                    all_errors.push(BorrowError::MutBorrowWhileShared {
                                        variable: info.var.clone(),
                                        shared_at: Location::new(shared.2),
                                        mut_at: Location::new(info.origin_point),
                                    });
                                    continue;
                                }
                            }
                        } else {
                            if let Some(mb) = builder.borrow_events.iter().find(|(v, m, pt, _)| v == &info.var && *m && *pt < info.origin_point) {
                                let prev_field = builder.loan_info.get(&mb.3).and_then(|li| li.field.clone());
                                if prev_field.is_none() || this_field.is_none() || prev_field == this_field {
                                    all_errors.push(BorrowError::SharedBorrowWhileMut {
                                        variable: info.var.clone(),
                                        mut_at: Location::new(mb.2),
                                        shared_at: Location::new(info.origin_point),
                                    });
                                    continue;
                                }
                            }
                        }

                        // 3) Field-borrow conflicts
                        if let Some(field) = &info.field {
                            // check other borrow events on same field
                            if builder.borrow_events.iter().any(|(v, m, pt, _)| v == &info.var && *pt < info.origin_point && *m) {
                                all_errors.push(BorrowError::FieldBorrowConflict {
                                    variable: info.var.clone(),
                                    field: field.clone(),
                                    first_borrow: Location::new(info.origin_point),
                                    second_borrow: Location::new(p_idx),
                                });
                                continue;
                            }
                        }

                        // Fallback: UseAfterMove with origin_point as moved_at (best-effort)
                        all_errors.push(BorrowError::UseAfterMove {
                            variable: info.var.clone(),
                            moved_at: Location::new(info.origin_point),
                            used_at: Location::new(p_idx),
                        });
                    }
                }
            }

            // Heuristic fallback checks
            if output.errors.is_empty() && output.move_errors.is_empty() {
                // helper: resolve var name from var id
                let find_var_name = |vid: Id| -> Option<String> {
                    builder
                        .var_map
                        .iter()
                        .find_map(|(k, v)| if *v == vid { Some(k.clone()) } else { None })
                };

                // Build assign map for quick lookup: path -> sorted assigned points
                let mut assigned_map: HashMap<Id, Vec<usize>> = HashMap::new();
                for (path, pt) in &builder.facts.path_assigned_at_base {
                    assigned_map.entry(*path).or_default().push(usize::from(*pt));
                }

                // Build move map from recorded bookkeeping as well
                let mut moved_map: HashMap<Id, Vec<usize>> = HashMap::new();
                for (pid, vec) in &builder.path_moves {
                    for (mp, _) in vec.iter() {
                        moved_map.entry(*pid).or_default().push(*mp);
                    }
                }

                // 1) Detect double-mut / mut-while-shared / shared-while-mut using borrow_events
                // Field-aware: use loan_info to consult field names and only report conflicts
                // when fields overlap or when either loan targets the whole variable.
                for (i, (var, mutable, pt, loan_id)) in builder.borrow_events.iter().enumerate() {
                    let varname = var.clone();
                    let pt_i = *pt;
                    let cur_field = builder.loan_info.get(loan_id).and_then(|li| li.field.clone());

                    // check previous events
                    for j in 0..i {
                        let (v2, m2, pt2, loan2) = &builder.borrow_events[j];
                        if v2 != var {
                            continue;
                        }

                        // check if earlier loan was killed between the two borrows
                        // (i.e., after the first borrow `pt2` but before `pt_i`).
                        let loan_killed = builder
                            .facts
                            .loan_killed_at
                            .iter()
                            .any(|(lk, kill_pt)| *lk == *loan2 && usize::from(*kill_pt) > *pt2 && usize::from(*kill_pt) < pt_i);

                        if loan_killed {
                            continue;
                        }

                        let prev_field = builder.loan_info.get(loan2).and_then(|li| li.field.clone());

                        if *mutable && *m2 {
                            // double mutable borrow on overlapping field(s)
                            if prev_field.is_some() && cur_field.is_some() && prev_field == cur_field {
                                let field_name = prev_field.unwrap();
                                let err = BorrowError::FieldBorrowConflict {
                                    variable: varname.clone(),
                                    field: field_name.clone(),
                                    first_borrow: Location::new(*pt2),
                                    second_borrow: Location::new(pt_i),
                                };
                                if !all_errors.contains(&err) {
                                    if env::var("POLONIUS_DEBUG").is_ok() {
                                        eprintln!("HEURISTIC: detected FieldBorrowConflict {} field {} @{} vs {}", varname, field_name, pt2, pt_i);
                                    }
                                    all_errors.push(err);
                                }
                                break;
                            } else if prev_field.is_none() || cur_field.is_none() || prev_field == cur_field {
                                let err = BorrowError::DoubleMutBorrow {
                                    variable: varname.clone(),
                                    first_borrow: Location::new(*pt2),
                                    second_borrow: Location::new(pt_i),
                                };
                                if !all_errors.contains(&err) {
                                    if env::var("POLONIUS_DEBUG").is_ok() {
                                        eprintln!("HEURISTIC: detected DoubleMutBorrow {} @{} vs {}", varname, pt2, pt_i);
                                    }
                                    all_errors.push(err);
                                }
                                break;
                            }
                        }

                        if *mutable && !*m2 {
                            // mut while shared exists; if same field -> FieldBorrowConflict
                            if prev_field.is_some() && cur_field.is_some() && prev_field == cur_field {
                                let field_name = prev_field.clone().unwrap();
                                let err = BorrowError::FieldBorrowConflict {
                                    variable: varname.clone(),
                                    field: field_name.clone(),
                                    first_borrow: Location::new(*pt2),
                                    second_borrow: Location::new(pt_i),
                                };
                                if !all_errors.contains(&err) {
                                    if env::var("POLONIUS_DEBUG").is_ok() {
                                        eprintln!("HEURISTIC: detected FieldBorrowConflict {} field {} shared@{} mut@{}", varname, field_name, pt2, pt_i);
                                    }
                                    all_errors.push(err);
                                }
                                break;
                            } else if prev_field.is_none() || cur_field.is_none() || prev_field == cur_field {
                                
                                // If the earlier loan wasn't killed, a mutable borrow while a
                                // shared borrow is outstanding is a real conflict. Report
                                // it unless we have explicit evidence the earlier loan was killed.
                                let err = BorrowError::MutBorrowWhileShared {
                                    variable: varname.clone(),
                                    shared_at: Location::new(*pt2),
                                    mut_at: Location::new(pt_i),
                                };
                                if !all_errors.contains(&err) {
                                    if env::var("POLONIUS_DEBUG").is_ok() {
                                        eprintln!("HEURISTIC: detected MutBorrowWhileShared {} shared@{} mut@{}", varname, pt2, pt_i);
                                    }
                                    all_errors.push(err);
                                }
                                break;
                            }
                        }

                        if !*mutable && *m2 {
                            // shared while mut exists on overlapping field(s)
                            if prev_field.is_some() && cur_field.is_some() && prev_field == cur_field {
                                let field_name = prev_field.clone().unwrap();
                                let err = BorrowError::FieldBorrowConflict {
                                    variable: varname.clone(),
                                    field: field_name.clone(),
                                    first_borrow: Location::new(*pt2),
                                    second_borrow: Location::new(pt_i),
                                };
                                if !all_errors.contains(&err) {
                                    if env::var("POLONIUS_DEBUG").is_ok() {
                                        eprintln!("HEURISTIC: detected FieldBorrowConflict {} field {} mut@{} shared@{}", varname, field_name, pt2, pt_i);
                                    }
                                    all_errors.push(err);
                                }
                                break;
                            } else if prev_field.is_none() || cur_field.is_none() || prev_field == cur_field {
                                
                                // If the earlier mutable borrow wasn't killed, a later shared
                                // borrow is a conflict. Report it unless there is explicit
                                // evidence the mutable loan ended in between.
                                let err = BorrowError::SharedBorrowWhileMut {
                                    variable: varname.clone(),
                                    mut_at: Location::new(*pt2),
                                    shared_at: Location::new(pt_i),
                                };
                                if !all_errors.contains(&err) {
                                    if env::var("POLONIUS_DEBUG").is_ok() {
                                        eprintln!("HEURISTIC: detected SharedBorrowWhileMut {} mut@{} shared@{}", varname, pt2, pt_i);
                                    }
                                    all_errors.push(err);
                                }
                                break;
                            }
                        }
                    }
                }

                // 2) Use-after-move and moved-while-borrowed detection using var_used_at and path_moves
                for (var_id, use_pt) in &builder.facts.var_used_at {
                    if let Some(varname) = find_var_name(*var_id) {
                        if let Some(pid) = builder.path_map.get(&varname) {
                            if let Some(moves) = moved_map.get(pid) {
                                // find latest move before use_pt
                                if let Some(last_move) = moves.iter().filter(|m| **m < usize::from(*use_pt)).max() {
                                    // check if any assignment re-initialized the path after move
                                    let reinit = assigned_map
                                        .get(pid)
                                        .map(|v| v.iter().any(|a| *a > *last_move && *a < usize::from(*use_pt)))
                                        .unwrap_or(false);

                                    if !reinit {
                                        let err = BorrowError::UseAfterMove {
                                            variable: varname.clone(),
                                            moved_at: Location::new(*last_move),
                                            used_at: Location::new(usize::from(*use_pt)),
                                        };
                                        if !all_errors.contains(&err) {
                                            all_errors.push(err);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 3) moved-while-borrowed: any move where an earlier borrow exists with no kill
                for (pid, moves) in &builder.path_moves {
                    // find variable name
                    let varname = builder
                        .path_info
                        .get(pid)
                        .map(|pi| pi.var.clone())
                        .or_else(|| {
                            // fallback: find by path_map reverse lookup
                            builder
                                .path_map
                                .iter()
                                .find_map(|(k, v)| if v == pid { Some(k.clone()) } else { None })
                        });
                    if varname.is_none() {
                        continue;
                    }
                    let varname = varname.unwrap();

                    for (mv_pt, inside_loop) in moves.iter() {
                        // any borrow event earlier than mv_pt and not killed
                        if let Some((_, _, bpt, _loan)) = builder
                            .borrow_events
                            .iter()
                            .filter(|(v, _m, bpt, _)| v == &varname && *bpt < *mv_pt)
                            .next()
                        {
                            // check if the loan was killed strictly between the borrow and move
                            let killed = builder.facts.loan_killed_at.iter().any(|(_lk, kpt)| {
                                let kp = usize::from(*kpt);
                                kp > *bpt && kp < *mv_pt
                            });
                            if !killed {
                                let err = BorrowError::MovedWhileBorrowed {
                                    variable: varname.clone(),
                                    borrow_at: Location::new(*bpt),
                                    move_at: Location::new(*mv_pt),
                                };
                                if !all_errors.contains(&err) {
                                    all_errors.push(err);
                                }
                            }
                        }
                        // loop moves -> MoveInLoop
                        if *inside_loop {
                            let err = BorrowError::MoveInLoop {
                                variable: varname.clone(),
                                move_at: Location::new(*mv_pt),
                            };
                            if !all_errors.contains(&err) {
                                all_errors.push(err);
                            }
                        }
                    }
                }

                // 4) Field borrow conflicts: check borrow_events with field info in loan_info
                for (_loan, info) in &builder.loan_info {
                    if let Some(field_name) = &info.field {
                        // find earlier borrow on same field that is mutable
                        if let Some(prev) = builder.borrow_events.iter().find(|(v, m, pt, _)| v == &info.var && *pt < info.origin_point && *m) {
                            let err = BorrowError::FieldBorrowConflict {
                                variable: info.var.clone(),
                                field: field_name.clone(),
                                first_borrow: Location::new(info.origin_point),
                                second_borrow: Location::new(prev.2),
                            };
                            if !all_errors.contains(&err) {
                                all_errors.push(err);
                            }
                        }
                    }
                }

                // 5) Mutation-of-immutable detection: find assignments to variables declared immutable
                for (name, pid) in &builder.path_map {
                    if let Some(is_mut) = builder.var_mutable.get(name) {
                        if !*is_mut {
                            let decl_pt = builder.var_decl_point.get(name).copied().unwrap_or(0usize);
                            for (p, assign_pt) in &builder.facts.path_assigned_at_base {
                                if p == pid {
                                    let ap = usize::from(*assign_pt);
                                    if ap != decl_pt {
                                        let err = BorrowError::MutationOfImmutable { variable: name.clone(), assign_at: Location::new(ap) };
                                        if !all_errors.contains(&err) {
                                            all_errors.push(err);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 6) Return-local-reference detection: scan function AST for return of borrowed local
                // Simple scan: find any Return(expr) where expr is Borrow(Identifier(name)) and the
                // declaration point for `name` is present and > 0.
                fn scan_returns_for_local_return(func: &ast::Function, builder: &FactBuilder) -> Vec<(String, usize)> {
                    let mut res = Vec::new();
                    fn walk_block(block: &ast::Block, res: &mut Vec<(String, usize)>, builder: &FactBuilder) {
                        for stmt in &block.statements {
                            match stmt {
                                ast::Statement::Return(expr_opt) => {
                                    if let Some(expr) = expr_opt {
                                        if let ast::Expression::Borrow { expr: inner, .. } = expr {
                                            if let ast::Expression::Identifier(name) = inner.as_ref() {
                                                if let Some(decl) = builder.var_decl_point.get(name) {
                                                    if *decl > 0 {
                                                        // best-effort: use decl as marker for return location (approx)
                                                        res.push((name.clone(), *decl));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                ast::Statement::If { then_block, else_block, .. } => {
                                    walk_block(then_block, res, builder);
                                    if let Some(eb) = else_block { walk_block(eb, res, builder); }
                                }
                                ast::Statement::For { body, .. } => walk_block(body, res, builder),
                                ast::Statement::While { body, .. } => walk_block(body, res, builder),
                                ast::Statement::Loop { body } => walk_block(body, res, builder),
                                ast::Statement::Match { arms, .. } => {
                                    for arm in arms {
                                        match &arm.body {
                                            ast::MatchBody::Expr(e) => {
                                                if let ast::Expression::Borrow { expr: inner, .. } = e {
                                                    if let ast::Expression::Identifier(name) = inner.as_ref() {
                                                        if let Some(decl) = builder.var_decl_point.get(name) {
                                                            if *decl > 0 {
                                                                res.push((name.clone(), *decl));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            ast::MatchBody::Block(b) => walk_block(b, res, builder),
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    walk_block(&func.body, &mut res, builder);
                    res
                }

                for (name, _decl_pt) in scan_returns_for_local_return(func, &builder) {
                    // attempt to find the return point from var_used_at entries
                    if let Some(vid) = builder.var_map.get(&name) {
                        if let Some((_, use_pt)) = builder.facts.var_used_at.iter().find(|(v, _)| v == vid) {
                            let err = BorrowError::ReturnLocalReference {
                                variable: name.clone(),
                                return_at: Location::new(usize::from(*use_pt)),
                            };
                            if !all_errors.contains(&err) {
                                all_errors.push(err);
                            }
                        }
                    }
                }
            }
        }
    }

    all_errors
}
