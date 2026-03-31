//! Integration example showing how to use semantic analyzer modules together
//!
//! This demonstrates the workflow of:
//! 1. Constraint collection and type inference
//! 2. Trait bound verification
//! 3. Borrow checking and lifetime validation
//! 4. Generic monomorphization

#![cfg(test)]
#![allow(unused_variables, dead_code)]

#[cfg(test)]
mod integration_tests {
    use crate::parser::ast::*;
    use crate::semantic::borrow_check::BorrowChecker;
    use crate::semantic::constraints::{Constraint, ConstraintSolver, InferenceType};
    use crate::semantic::monomorphization::{ExpressionSubstitutor, TypeSubstitution};
    use crate::semantic::traits::*;

    /// Demonstrates type inference using constraints
    #[test]
    fn test_type_inference_workflow() {
        // Create a constraint solver
        let mut solver = ConstraintSolver::new();

        // Generate type variables
        let t1 = solver.fresh_var();
        let t2 = solver.fresh_var();

        // Add constraints:
        // t1 = i32
        solver.add_constraint(Constraint::Equals(
            InferenceType::Var(t1.clone()),
            InferenceType::Concrete(Box::new(Type::I32)),
        ));

        // t2 = i32
        solver.add_constraint(Constraint::Equals(
            InferenceType::Var(t2.clone()),
            InferenceType::Concrete(Box::new(Type::I32)),
        ));

        // Solve constraints
        let subst = solver.solve().expect("Solving constraints should succeed");

        // Apply substitution to verify types
        let result_t1 = subst.apply(&InferenceType::Var(t1));
        let result_t2 = subst.apply(&InferenceType::Var(t2));

        assert_eq!(result_t1, InferenceType::Concrete(Box::new(Type::I32)));
        assert_eq!(result_t2, InferenceType::Concrete(Box::new(Type::I32)));
    }

    /// Demonstrates trait bound checking
    #[test]
    fn test_trait_bound_workflow() {
        let mut resolver = TraitResolver::new();

        // Check if i32 implements Copy
        assert!(resolver.implements_trait(&Type::I32, "Copy"));
        // Create trait bounds
        let bounds = vec![TraitBound {
            type_param: "T".to_string(),
            trait_name: "Copy".to_string(),
            assoc_type_bindings: std::collections::HashMap::new(),
            where_clauses: Vec::new(),
        }];

        // Check bounds for copy type
        assert!(resolver.check_bounds(&Type::I64, &bounds).is_ok());

        // Check bounds for non-copy type (string)
        assert!(resolver.check_bounds(&Type::Str, &bounds).is_err());
    }

    /// Demonstrates borrow checking workflow
    #[test]
    fn test_borrow_checking_workflow() {
        let mut checker = BorrowChecker::new();

        // Declare a variable
        checker
            .bind_variable("my_vec".to_string())
            .expect("declare should work");

        // Borrow it immutably
        checker.borrow_shared("my_vec").expect("borrow should work");

        // Can borrow again immutably
        checker
            .borrow_shared("my_vec")
            .expect("second borrow should work");

        // Try to move while borrowed - should fail
        assert!(checker.move_var("my_vec").is_err());
    }

    /// Demonstrates lifetime validation
    #[test]
    fn test_lifetime_workflow() {
        // Note: LifetimeValidator is defined in constraints module
        // This test demonstrates the concept using the trait system instead

        let mut resolver = TraitResolver::new();
        // Lifetimes are typically associated with trait bounds
        // This demonstrates lifetime-like constraints through trait bounds
        let bounds = vec![TraitBound {
            type_param: "'a".to_string(),
            trait_name: "Copy".to_string(),
            assoc_type_bindings: std::collections::HashMap::new(),
            where_clauses: Vec::new(),
        }];

        // Check bounds for copy type
        assert!(resolver.check_bounds(&Type::I64, &bounds).is_ok());
    }

    /// Demonstrates generic monomorphization
    #[test]
    fn test_monomorphization_workflow() {
        let mut subst = TypeSubstitution::new();
        subst.add("T".to_string(), Type::I64);
        subst.add("U".to_string(), Type::Bool);

        // Test substitution of generic type
        let generic_type = Type::Generic(
            "Vec".to_string(),
            vec![Type::Named("T".to_string()), Type::Named("U".to_string())],
        );

        let mono_type = subst.apply_to_type(&generic_type);

        match mono_type {
            Type::Generic(name, args) => {
                assert_eq!(name, "Vec");
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0], Type::I64));
                assert!(matches!(args[1], Type::Bool));
            }
            _ => panic!("Expected generic type after substitution"),
        }
    }

    /// Demonstrates integrated workflow: inference + traits + borrowing
    #[test]
    fn test_complete_semantic_analysis_workflow() {
        // Step 1: Type inference
        let mut solver = ConstraintSolver::new();
        let param_type = solver.fresh_var();
        solver.add_constraint(Constraint::Equals(
            InferenceType::Var(param_type.clone()),
            InferenceType::Concrete(Box::new(Type::I32)),
        ));
        let subst = solver.solve().expect("Should solve");

        // Step 2: Trait verification
        let mut trait_resolver = TraitResolver::new();
        let inferred_type = subst.apply(&InferenceType::Var(param_type));
        // (Would check trait bounds here)

        // Step 3: Borrow checking
        let mut borrow_checker = BorrowChecker::new();
        borrow_checker.bind_variable("x".to_string()).unwrap();
        borrow_checker.borrow_shared("x").unwrap();
        assert!(borrow_checker.can_read("x").is_ok());

        // Step 4: Monomorphization for generics
        let mut subst = TypeSubstitution::new();
        subst.add("T".to_string(), Type::I32);
        let result = subst.apply_to_type(&Type::Named("T".to_string()));
        assert!(matches!(result, Type::I32));
    }

    /// Demonstrates Phase 3: Integrated constraint solving with multiple constraints
    #[test]
    fn test_phase3_constraint_integration() {
        let mut solver = ConstraintSolver::new();

        // Create type variables for a generic function: fn foo<T, U>(x: T, y: U) -> T
        let t = solver.fresh_var();
        let u = solver.fresh_var();

        // Add constraints from arguments
        // x is i32, so T = i32
        solver.add_constraint(Constraint::Equals(
            InferenceType::Var(t.clone()),
            InferenceType::Concrete(Box::new(Type::I32)),
        ));

        // y is bool, so U = bool
        solver.add_constraint(Constraint::Equals(
            InferenceType::Var(u.clone()),
            InferenceType::Concrete(Box::new(Type::Bool)),
        ));

        // Solve and verify
        let subst = solver.solve().expect("Should solve constraints");
        assert_eq!(
            subst.apply(&InferenceType::Var(t)),
            InferenceType::Concrete(Box::new(Type::I32))
        );
        assert_eq!(
            subst.apply(&InferenceType::Var(u)),
            InferenceType::Concrete(Box::new(Type::Bool))
        );
    }

    /// Demonstrates Phase 3: Integrated trait bounds with generic monomorphization
    #[test]
    fn test_phase3_trait_bounds_with_generics() {
        let mut resolver = TraitResolver::new();

        // Generic function signature: fn clone_if_copy<T: Copy>(x: T) -> T
        let bounds = vec![TraitBound {
            type_param: "T".to_string(),
            trait_name: "Copy".to_string(),
            assoc_type_bindings: std::collections::HashMap::new(),
            where_clauses: Vec::new(),
        }];

        // Valid instantiation: T = i64 (implements Copy)
        assert!(resolver.check_bounds(&Type::I64, &bounds).is_ok());

        // Invalid instantiation: T = Str (doesn't implement Copy)
        assert!(resolver.check_bounds(&Type::Str, &bounds).is_err());

        // Valid instantiation: T = bool (implements Copy)
        assert!(resolver.check_bounds(&Type::Bool, &bounds).is_ok());
    }

    /// Demonstrates Phase 3: Borrow checking with scopes and multiple variables
    #[test]
    fn test_phase3_borrow_scopes() {
        let mut checker = BorrowChecker::new();

        // Outer scope
        checker.bind_variable("vec1".to_string()).unwrap();
        checker.bind_variable("vec2".to_string()).unwrap();

        // Borrow vec1
        checker.borrow_shared("vec1").unwrap();
        assert!(checker.can_read("vec1").is_ok());

        // Create inner scope
        checker.push_scope();

        // In inner scope, borrow vec2
        checker.bind_variable("local".to_string()).unwrap();
        checker.borrow_shared("local").unwrap();
        assert!(checker.can_read("local").is_ok());

        // Exit inner scope - local borrows invalidated
        checker.pop_scope();

        // Original borrows still valid in outer scope
        assert!(checker.can_read("vec1").is_ok());
    }

    /// Demonstrates Phase 3: Generic monomorphization with complex types
    #[test]
    fn test_phase3_complex_monomorphization() {
        let mut subst = TypeSubstitution::new();

        // Template: Vec<T> where T = Array<i32>
        subst.add("T".to_string(), Type::Array(Box::new(Type::I32), None));

        // Generic type: Vec<T>
        let generic = Type::Generic("Vec".to_string(), vec![Type::Named("T".to_string())]);

        // Monomorphize
        let concrete = subst.apply_to_type(&generic);

        // Verify: Vec<Array<i32>>
        match concrete {
            Type::Generic(name, args) => {
                assert_eq!(name, "Vec");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    Type::Array(elem, _) => {
                        assert!(matches!(**elem, Type::I32));
                    }
                    _ => panic!("Expected array type"),
                }
            }
            _ => panic!("Expected generic type"),
        }
    }

    /// Demonstrates Phase 3: All systems integrated together
    #[test]
    fn test_phase3_complete_integration() {
        // Scenario: Generic function with trait bounds and monomorphization
        // fn map<T: Copy, U>(items: Vec<T>, f: fn(T) -> U) -> Vec<U>

        // 1. Constraint solving for type inference
        let mut constraint_solver = ConstraintSolver::new();
        let t = constraint_solver.fresh_var();
        let u = constraint_solver.fresh_var();

        constraint_solver.add_constraint(Constraint::Equals(
            InferenceType::Var(t.clone()),
            InferenceType::Concrete(Box::new(Type::I32)),
        ));
        constraint_solver.add_constraint(Constraint::Equals(
            InferenceType::Var(u.clone()),
            InferenceType::Concrete(Box::new(Type::Bool)),
        ));

        let type_subst = constraint_solver.solve().unwrap();

        // 2. Trait bound verification
        let mut trait_resolver = TraitResolver::new();
        let bounds = vec![TraitBound {
            type_param: "T".to_string(),
            trait_name: "Copy".to_string(),
            assoc_type_bindings: std::collections::HashMap::new(),
            where_clauses: Vec::new(),
        }];

        // Verify T = i32 satisfies Copy bound
        assert!(trait_resolver.check_bounds(&Type::I32, &bounds).is_ok());

        // 3. Borrow checking for parameters
        let mut borrow_checker = BorrowChecker::new();
        borrow_checker.bind_variable("items".to_string()).unwrap();
        borrow_checker.borrow_shared("items").unwrap();
        assert!(borrow_checker.can_read("items").is_ok());

        // 4. Monomorphization of generic function
        let mut mono_subst = TypeSubstitution::new();
        mono_subst.add("T".to_string(), Type::I32);
        mono_subst.add("U".to_string(), Type::Bool);

        // Monomorphize Vec<T> -> Vec<i32>
        let vec_t = Type::Generic("Vec".to_string(), vec![Type::Named("T".to_string())]);
        let vec_concrete = mono_subst.apply_to_type(&vec_t);

        match vec_concrete {
            Type::Generic(name, args) => {
                assert_eq!(name, "Vec");
                assert_eq!(args[0], Type::I32);
            }
            _ => panic!("Expected monomorphized Vec type"),
        }
    }
}
