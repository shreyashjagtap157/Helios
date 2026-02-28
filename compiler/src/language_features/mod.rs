/// Language Features Module
/// Provides complete implementations of all deferred and advanced language features

pub mod lazy_static;
pub mod variadics;
pub mod operator_overloading;
pub mod default_params;

pub use lazy_static::{LazyStat};
pub use variadics::VariadicFunction;
pub use operator_overloading::{Operator, OperatorRegistry};
pub use default_params::{Parameter, FunctionSignature, CallResolver, Argument};

/// Re-export all public items for easy access
pub mod prelude {
    pub use crate::language_features::{
        LazyStat, VariadicFunction, Operator, OperatorRegistry, Parameter, FunctionSignature,
        CallResolver, Argument,
    };
}
