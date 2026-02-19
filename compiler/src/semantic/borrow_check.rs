//! Borrow checking enforcement
//!
//! Validates borrowing rules:
//! - No use-after-move
//! - No multiple mutable borrows
//! - No borrow-after-move
//! - Proper borrow scope tracking

use std::collections::HashMap;

/// Track active borrows and their scopes
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowConstraint {
    /// Variable is uniquely owned
    Owned,
    /// Variable can have multiple shared borrows
    SharedBorrow(Vec<usize>),
    /// Variable has exclusive mutable borrow
    MutBorrow(usize),
    /// Variable has been moved
    Moved(usize),
}

/// Borrow checker state
pub struct BorrowChecker {
    /// Variable -> current borrow state
    states: HashMap<String, BorrowConstraint>,
    /// Variable -> scope where it was bound
    binding_scopes: HashMap<String, usize>,
    /// Active borrow sites
    borrow_sites: Vec<(String, usize, bool)>, // (var, scope, is_mut)
    /// Current scope depth
    current_scope: usize,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            binding_scopes: HashMap::new(),
            borrow_sites: Vec::new(),
            current_scope: 0,
        }
    }

    /// Enter a new scope
    pub fn push_scope(&mut self) {
        self.current_scope += 1;
    }

    /// Exit current scope and invalidate all borrows
    pub fn pop_scope(&mut self) {
        let scope_to_exit = self.current_scope;
        self.current_scope = self.current_scope.saturating_sub(1);

        // Remove borrows created in exited scope
        self.borrow_sites
            .retain(|(_, scope, _)| *scope <= self.current_scope);
    }

    /// Declare a new variable (binding)
    pub fn bind_variable(&mut self, name: String) -> Result<(), String> {
        if self.binding_scopes.contains_key(&name) {
            return Err(format!("Variable {} already bound in current scope", name));
        }
        self.states.insert(name.clone(), BorrowConstraint::Owned);
        self.binding_scopes.insert(name, self.current_scope);
        Ok(())
    }

    /// Use a variable immutably (shared borrow)
    pub fn borrow_shared(&mut self, name: &str) -> Result<(), String> {
        let state = self
            .states
            .get(name)
            .ok_or_else(|| format!("Variable {} not found", name))?;

        match state {
            BorrowConstraint::Owned => {
                // Create new shared borrow
                let borrow_id = self.borrow_sites.len();
                self.states.insert(
                    name.to_string(),
                    BorrowConstraint::SharedBorrow(vec![borrow_id]),
                );
                self.borrow_sites
                    .push((name.to_string(), self.current_scope, false));
                Ok(())
            }
            BorrowConstraint::SharedBorrow(ref ids) => {
                // Add to existing shared borrows
                let mut new_ids = ids.clone();
                let borrow_id = self.borrow_sites.len();
                new_ids.push(borrow_id);
                self.states.insert(
                    name.to_string(),
                    BorrowConstraint::SharedBorrow(new_ids),
                );
                self.borrow_sites
                    .push((name.to_string(), self.current_scope, false));
                Ok(())
            }
            BorrowConstraint::MutBorrow(_) => {
                Err(format!(
                    "Cannot borrow {} immutably while mutably borrowed",
                    name
                ))
            }
            BorrowConstraint::Moved(_) => {
                Err(format!("Variable {} used after move", name))
            }
        }
    }

    /// Use a variable mutably (exclusive borrow)
    pub fn borrow_mut(&mut self, name: &str) -> Result<(), String> {
        let state = self
            .states
            .get(name)
            .ok_or_else(|| format!("Variable {} not found", name))?;

        match state {
            BorrowConstraint::Owned => {
                // Create exclusive mutable borrow
                let borrow_id = self.borrow_sites.len();
                self.states
                    .insert(name.to_string(), BorrowConstraint::MutBorrow(borrow_id));
                self.borrow_sites
                    .push((name.to_string(), self.current_scope, true));
                Ok(())
            }
            BorrowConstraint::SharedBorrow(_) => {
                Err(format!(
                    "Cannot borrow {} mutably while immutably borrowed",
                    name
                ))
            }
            BorrowConstraint::MutBorrow(_) => {
                Err(format!(
                    "Cannot borrow {} mutably while already mutably borrowed",
                    name
                ))
            }
            BorrowConstraint::Moved(_) => {
                Err(format!("Variable {} used after move", name))
            }
        }
    }

    /// Move a variable (transfer ownership)
    pub fn move_var(&mut self, name: &str) -> Result<(), String> {
        let state = self
            .states
            .get(name)
            .ok_or_else(|| format!("Variable {} not found", name))?;

        match state {
            BorrowConstraint::Owned => {
                let move_id = self.borrow_sites.len();
                self.states
                    .insert(name.to_string(), BorrowConstraint::Moved(move_id));
                self.borrow_sites
                    .push((name.to_string(), self.current_scope, false));
                Ok(())
            }
            BorrowConstraint::SharedBorrow(_) => {
                Err(format!(
                    "Cannot move {} while immutably borrowed",
                    name
                ))
            }
            BorrowConstraint::MutBorrow(_) => {
                Err(format!("Cannot move {} while mutably borrowed", name))
            }
            BorrowConstraint::Moved(_) => {
                Err(format!("Variable {} used after move", name))
            }
        }
    }

    /// Validate that variable can be read
    pub fn can_read(&self, name: &str) -> Result<(), String> {
        let state = self
            .states
            .get(name)
            .ok_or_else(|| format!("Variable {} not found", name))?;

        match state {
            BorrowConstraint::Moved(_) => Err(format!("Variable {} used after move", name)),
            _ => Ok(()),
        }
    }

    /// Validate that variable can be written to
    pub fn can_write(&self, name: &str) -> Result<(), String> {
        let state = self
            .states
            .get(name)
            .ok_or_else(|| format!("Variable {} not found", name))?;

        match state {
            BorrowConstraint::Owned => Ok(()),
            BorrowConstraint::SharedBorrow(_) => {
                Err(format!(
                    "Cannot mutate {} while immutably borrowed",
                    name
                ))
            }
            BorrowConstraint::MutBorrow(_) => {
                Err(format!("Cannot mutate {} while mutably borrowed", name))
            }
            BorrowConstraint::Moved(_) => {
                Err(format!("Variable {} used after move", name))
            }
        }
    }

    /// Return a borrowed variable (end borrow)
    pub fn return_borrow(&mut self, name: &str) -> Result<(), String> {
        let state = self
            .states
            .get(name)
            .ok_or_else(|| format!("Variable {} not found", name))?;

        match state {
            BorrowConstraint::SharedBorrow(ids) if !ids.is_empty() => {
                // Remove last borrow (LIFO order)
                let mut new_ids = ids.clone();
                new_ids.pop();
                if new_ids.is_empty() {
                    self.states
                        .insert(name.to_string(), BorrowConstraint::Owned);
                } else {
                    self.states.insert(
                        name.to_string(),
                        BorrowConstraint::SharedBorrow(new_ids),
                    );
                }
                Ok(())
            }
            BorrowConstraint::MutBorrow(_) => {
                self.states
                    .insert(name.to_string(), BorrowConstraint::Owned);
                Ok(())
            }
            _ => Err(format!("No borrow to return for {}", name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_variable() {
        let mut checker = BorrowChecker::new();
        assert!(checker.bind_variable("x".to_string()).is_ok());
        assert!(checker.bind_variable("x".to_string()).is_err());
    }

    #[test]
    fn test_borrow_owned_variable() {
        let mut checker = BorrowChecker::new();
        checker.bind_variable("x".to_string()).unwrap();
        assert!(checker.borrow_shared("x").is_ok());
    }

    #[test]
    fn test_exclusive_mut_borrow() {
        let mut checker = BorrowChecker::new();
        checker.bind_variable("x".to_string()).unwrap();
        assert!(checker.borrow_mut("x").is_ok());
        // Cannot borrow again while mutably borrowed
        assert!(checker.borrow_shared("x").is_err());
    }

    #[test]
    fn test_move_owned_variable() {
        let mut checker = BorrowChecker::new();
        checker.bind_variable("x".to_string()).unwrap();
        assert!(checker.move_var("x").is_ok());
        // Cannot use after move
        assert!(checker.can_read("x").is_err());
    }

    #[test]
    fn test_cannot_move_borrowed_variable() {
        let mut checker = BorrowChecker::new();
        checker.bind_variable("x".to_string()).unwrap();
        checker.borrow_shared("x").unwrap();
        assert!(checker.move_var("x").is_err());
    }

    #[test]
    fn test_scope_tracking() {
        let mut checker = BorrowChecker::new();
        checker.bind_variable("x".to_string()).unwrap();
        checker.push_scope();
        assert!(checker.borrow_shared("x").is_ok());
        checker.pop_scope();
        // Borrow should be invalidated after scope exit
        assert_eq!(checker.borrow_sites.len(), 0);
    }

    #[test]
    fn test_cannot_write_borrowed() {
        let mut checker = BorrowChecker::new();
        checker.bind_variable("x".to_string()).unwrap();
        checker.borrow_shared("x").unwrap();
        assert!(checker.can_write("x").is_err());
    }
}
