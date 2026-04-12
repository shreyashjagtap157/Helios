# OMNI PROGRAMMING LANGUAGE — COMPLETE IMPLEMENTATION PROMPT

## Project Identity

You are implementing the **Omni Programming Language** — a hybrid multi-level platform language with ownership, effects, and strong type safety. Your task is to complete the entire implementation from the current state to a fully functional, self-hosting compiler.

---

## CURRENT STATE ASSESSMENT

### What's Working ✅
- Lexer: Basic tokenization (keywords, operators, literals, comments)
- Parser: Recursive descent for functions, structs, variables, control flow
- Type Inference: Hindley-Milner basic implementation
- OVM Bytecode: Code generation and runtime execution
- Interpreter: Tree-walking execution
- Basic LSP: Syntax highlighting, minimal diagnostics

### What's Missing ❌
- **Effect System**: Not implemented (CRITICAL - v2.0 core feature)
- **Polonius Borrow Checker**: Uses NLL instead (WRONG ALGORITHM)
- **Indentation Blocks**: Uses braces, not indentation-based syntax
- **String Interpolation**: `f"..."` and `d"..."` not supported
- **Pattern Matching**: Not in parser
- **Linear Types**: Not implemented
- **Generational References**: Not implemented
- **Arena Allocation**: Not implemented
- **GC Compatibility Layer**: Not implemented
- **CST (Lossless)**: Parser produces lossy AST
- **Structured Concurrency**: Not enforced
- **Capability System**: Not implemented
- **Security/Sandboxing**: Not implemented
- **Stable Diagnostics**: Ad-hoc error codes
- **Full Standard Library**: Stubbed
- **Self-Hosting**: Only lexer + basic parse

### What's Broken 🔴
- Borrow checker uses wrong algorithm (needs Polonius)
- Self-hosting claim is misleading (minimal implementation)
- HELIOS framework exists prematurely (should be removed)

---

## PHASE 1: FOUNDATIONAL INFRASTRUCTURE

### 1.1 Complete Lexer Implementation

**File**: `omni-lang/compiler/src/lexer.rs`

Add the following token types and functionality:

```rust
// Add to TokenType enum:
INDENT,        // Indentation increase
DEDENT,        // Indentation decrease
FStringStart,  // f" prefix
FStringEnd,    // " suffix (for interpolation)
DStringStart,  // d" prefix (Debug)
DStringEnd,    // " suffix
EffectSlash,   // / for effect annotations
Linear,       // linear modifier
Inout,        // inout parameter
At,           // @ for attributes
Question,     // ? for optional/Result
FatArrow,     // => for match arms
Pipe,         // | for let-chains and match
Ampersand,    // & reference
DoubleAmpersand, // && logical and
Or,           // || logical or (rename existing Or)
DoubleColon,  // :: for paths

// Add new string literal handling:
FString(String),  // Interpolated strings
DString(String),   // Debug strings

// Add effect annotation handling:
EffectAnnotation {
    effects: Vec<EffectKind>,
}

// Effect kinds:
enum EffectKind {
    Io,
    Async,
    Throw(Box<Type>),  // throw<E>
    Panic,
    Alloc,
    Rand,
    Time,
    Log,
    Pure,
    Custom(String),    // User-defined effects
}
```

**Implementation Requirements**:
1. Track column position for indentation detection
2. Emit INDENT/DEDENT tokens based on whitespace changes
3. Handle string interpolation with `{expr}` parsing
4. Parse effect annotations in function signatures: `fn name() -> T / io + async`
5. Implement arena allocation for tokens (performance)
6. Add error recovery for malformed tokens

### 1.2 Complete Parser Implementation

**File**: `omni-lang/compiler/src/parser.rs`

Replace/enhance with:

```rust
// Complete AST Node Definitions
#[derive(Debug, Clone)]
pub enum Type {
    // Existing types...
    Never,           // ! type for diverging
    Nullable(Box<Type>), // T?
    Result(Box<Type>, Box<Type>), // Result<T, E>
    Function(Box<Type>, Box<Type>), // Fn(A) -> B
    TraitObject(String), // dyn Trait
    Generic(String), // T
    Applied(Box<Type>, Vec<Type>), // Vec<i32>
}

// Add pattern matching support
#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Identifier(String),
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Struct(String, Vec<(String, Pattern)>),
    Enum(String, Option<Box<Pattern>>),
    Or(Box<Pattern>, Box<Pattern>),
    Guard(Box<Pattern>, Expression),
}

// Add match expression
#[derive(Debug, Clone)]
pub struct Match {
    expr: Expression,
    arms: Vec<(Pattern, Expression)>,
}

// Add effect annotations
#[derive(Debug, Clone)]
pub struct EffectAnnotation {
    effects: Vec<EffectKind>,
}

// Add linear types
#[derive(Debug, Clone)]
pub struct LinearType {
    inner: Box<Type>,
}

// Add inout parameter
#[derive(Debug, Clone)]
pub struct InoutParam {
    name: String,
    param_type: Type,
}

// Add let-chain
#[derive(Debug, Clone)]
pub struct LetChain {
    bindings: Vec<(Pattern, Expression)>,
    body: Expression,
}

// Add async closure
#[derive(Debug, Clone)]
pub struct AsyncClosure {
    params: Vec<(String, Type)>,
    return_type: Type,
    body: Block,
}

// Add attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    name: String,
    value: Option<Expression>,
}

// Add effect handler
#[derive(Debug, Clone)]
pub struct EffectHandler {
    effect: EffectKind,
    body: Block,
}
```

**Parser Requirements**:
1. Implement Pratt parser for expressions (better precedence handling)
2. Add INDENT/DEDENT handling for indentation blocks
3. Implement `match` expression parsing with guards
4. Parse effect annotations: `fn foo() -> T / io async`
5. Parse linear types: `let linear x = ...`
6. Parse inout parameters: `fn foo(inout x: T)`
7. Parse let-chains: `let x = a and y = b in expr`
8. Parse async closures: `async |x: T| -> U { ... }`
9. Parse deconstructing parameters: `fn foo((a, b): (T, U))`
10. Implement error recovery with synchronization sets
11. Produce lossless CST (preserve all source info)

### 1.3 Complete AST/CST Implementation

**File**: `omni-lang/compiler/src/ast.rs`

Create Rowan-based CST:

```rust
use rowan::GreenNode;

// Lossless Concrete Syntax Tree
pub struct CST {
    green: GreenNode,
    text: String,
}

pub enum CstNode {
    SourceFile(Vec<CstNode>),
    ModuleItem(Item),
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Impl(Impl),
    Statement(Statement),
    Expression(Expression),
    Pattern(Pattern),
    Type(Type),
    Attribute(Attribute),
    // ... all syntax elements
}

impl CST {
    pub fn new(green: GreenNode, text: String) -> Self;
    pub fn root(&self) -> CstNode;
    pub fn text(&self) -> &str;
    pub fn span(&self, node: CstNode) -> Span;
}

pub struct Span {
    pub start: Location,
    pub end: Location,
}

pub struct Location {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

// Visitor traits
pub trait Visitor<T> {
    fn visit_source_file(&mut self, node: &CstNode) -> T;
    fn visit_function(&mut self, node: &CstNode) -> T;
    // ... all visit methods
}

pub trait VisitorMut: Sized {
    fn visit_source_file(&mut self, node: &mut CstNode);
    fn visit_function(&mut self, node: &mut CstNode);
    // ... all visit methods
}

// Pretty printer
pub fn pretty_print(node: &CstNode) -> String;
```

---

## PHASE 2: TYPE SYSTEM

### 2.1 Complete Type Inference

**File**: `omni-lang/compiler/src/semantic/type_inference.rs`

Implement bidirectional type inference (v2.0 requirement):

```rust
// Bidirectional inference context
pub struct InferenceContext {
    // Function signatures known to inference
    signatures: HashMap<String, FunctionSignature>,
    
    // Type variables and their constraints
    variables: HashMap<TypeVarId, TypeVarState>,
    
    // Trait environment
    trait_env: TraitEnvironment,
    
    // Effect variable tracking
    effect_vars: HashMap<EffectVarId, EffectVarState>,
}

pub struct FunctionSignature {
    params: Vec<(String, Type)>,
    return_type: Type,
    effect_set: EffectSet,
    constraints: Vec<TraitConstraint>,
}

pub struct TraitEnvironment {
    // Registered traits with their associated types and bounds
    traits: HashMap<String, TraitDef>,
}

pub struct TraitDef {
    name: String,
    associated_types: Vec<AssociatedType>,
    bounds: Vec<TraitConstraint>,
    methods: Vec<MethodSignature>,
}

pub struct TraitConstraint {
    trait_name: String,
    type_param: Type,
    args: Vec<Type>,
}

pub struct TypeVarState {
    var: TypeVar,
    constraints: Vec<TypeConstraint>,
    occurs: Vec<TypeVarId>,
}

pub enum TypeConstraint {
    Eq(Type, Type),
    Subtype(Type, Type),
    Trait(Type, TraitConstraint),
    Effect(Type, EffectConstraint),
}

pub enum Type {
    // ... existing ...
    Never,
    Nullable(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Function(Box<Type>, Box<Type>),
    TraitObject(String),
    Generic(String),
    Applied(Box<Type>, Vec<Type>),
    EffectSet(Set<EffectKind>),
}
```

### 2.2 Trait System Implementation

**File**: `omni-lang/compiler/src/semantic/traits.rs`

```rust
// Trait definitions
pub trait Trait: Debug + Clone {
    fn name(&self) -> &str;
    fn associated_types(&self) -> &[AssociatedType];
    fn methods(&self) -> &[Method];
    fn supertraits(&self) -> &[TraitRef];
}

pub struct TraitDef {
    name: String,
    id: TraitId,
    associated_types: Vec<AssociatedType>,
    methods: Vec<MethodDef>,
    supertraits: Vec<TraitRef>,
    default_impls: HashMap<MethodId, ImplBlock>,
}

pub struct AssociatedType {
    name: String,
    bounds: Vec<TraitConstraint>,
    default: Option<Type>,
}

pub struct MethodDef {
    name: String,
    sig: MethodSignature,
    body: Option<Block>,
}

pub struct MethodSignature {
    params: Vec<FnParam>,
    return_type: Type,
    effect_set: EffectSet,
    bounds: Vec<TraitConstraint>,
}

// Trait implementation
pub struct ImplBlock {
    trait_ref: Option<TraitRef>,
    self_type: Type,
    items: Vec<ImplItem>,
}

pub enum ImplItem {
    Method(MethodDef),
    Type(AssociatedTypeImpl),
    Const(ConstDef),
}

// Trait bounds and specialization
pub enum TraitBound {
    Single(TraitRef),
    Composite(Vec<TraitBound>, TraitCompositor),
}

pub trait TraitCompositor {
    fn compose(&self, a: TraitRef, b: TraitRef) -> Result<TraitRef, CoherenceError>;
}

// Negative bounds (v2.0 requirement)
pub struct NegativeBound {
    type_param: Type,
    prohibited: TraitRef,
}

// Trait upcasting (v2.0 requirement)
pub fn trait_upcast<T: Trait, U: SuperTrait<T>>() -> U;
// where U: SuperTrait<T>
```

### 2.3 Generic Specialization

**File**: `omni-lang/compiler/src/semantic/monomorphization.rs`

```rust
// Monomorphization engine
pub struct MonoEngine {
    cache: HashMap<MonoKey, MonoInstance>,
    pending: Vec<MonoJob>,
}

pub struct MonoKey {
    generic_fn: DefId,
    type_args: Vec<Type>,
    effect_args: Vec<EffectKind>,
}

pub struct MonoJob {
    key: MonoKey,
    body: Expression,
    constraints: Vec<TraitConstraint>,
}

pub struct MonoInstance {
    def_id: DefId,
    specialized_fn: Function,
    vtable: VTable,
}

// Specialization rules
pub enum Specialization {
    Default,
    Impl(TraitImpl),
    Overload(Vec<TraitImpl>),
}

pub fn select_impl<T: Trait>(type_args: &[Type]) -> Option<&TraitImpl>;

// Compile-time evaluation for const generics
pub fn evaluate_const_expr(expr: &ConstExpr, generics: &Generics) -> Value;
```

---

## PHASE 3: EFFECT SYSTEM (CRITICAL)

### 3.1 Effect Definitions

**File**: `omni-lang/compiler/src/effects/mod.rs`

```rust
// Core effect types
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Effect {
    // Built-in effects
    Io,
    Async,
    Throw(Type),        // throw<E>
    Panic,
    Alloc,
    Rand,
    Time,
    Log,
    Pure,
    
    // User-defined effects
    Custom(String),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EffectSet {
    effects: FxHashSet<Effect>,
}

impl EffectSet {
    pub fn empty() -> Self;
    pub fn pure() -> Self;
    pub fn add(&self, effect: Effect) -> Self;
    pub fn union(&self, other: &EffectSet) -> Self;
    pub fn contains(&self, effect: &Effect) -> bool;
    pub fn is_pure(&self) -> bool;
}

// Effect annotations
#[derive(Debug, Clone)]
pub struct EffectAnnotation {
    effects: EffectSet,
    capability: Option<Capability>,
}

pub fn parse_effect_annotation(s: &str) -> Result<EffectAnnotation, ParseError>;

// Effect inference
pub struct EffectInferrer {
    env: EffectEnv,
    vars: HashMap<EffectVarId, EffectVar>,
    constraints: Vec<EffectConstraint>,
}

pub enum EffectConstraint {
    SubEffect(EffectVarId, Effect),
    VarEq(EffectVarId, EffectVarId),
    Combines(EffectVarId, EffectVarId, Effect),
}

pub fn infer_effects(expr: &Expression, ctx: &InferenceContext) -> EffectSet;
```

### 3.2 Effect Handlers

**File**: `omni-lang/compiler/src/effects/handlers.rs`

```rust
// Effect handler definition
pub struct EffectHandler {
    effect: Effect,
    param_name: String,
    param_type: Type,
    resume_type: Type,
    body: Block,
    is_resumable: bool,
}

// Handler registration
pub struct HandlerRegistry {
    handlers: HashMap<Effect, EffectHandler>,
    stack: Vec<EffectFrame>,
}

pub struct EffectFrame {
    handler: EffectHandler,
    env: Environment,
    continuation: Continuation,
}

// Continuation types
pub enum Continuation {
    Return(Value),
    Tail,
    Raise(EffectValue),
    Resumable(EffectValue, Box<Continuation>),
}

// Effect operations
pub enum EffectOp {
    Perform(Effect, Value),
    Resume(EffectValue, Value),
    Raise(EffectValue),
    Suspend(Effect, Box<Continuation>),
}

// Handler implementation
pub fn install_handler(effect: Effect, handler: EffectHandler) -> Result<(), HandlerError>;
pub fn perform_effect(effect: Effect, value: Value) -> Result<Value, EffectValue>;
pub fn with_handler<R>(effect: Effect, handler: EffectHandler, f: impl FnOnce() -> R) -> R;
```

### 3.3 Built-in Effect Implementations

**File**: `omni-lang/compiler/src/effects/builtins.rs`

```rust
// IO effect handler
pub struct IoHandler {
    stdin: Stdin,
    stdout: Stdout,
    stderr: Stderr,
}

impl EffectHandler for IoHandler {
    fn handle(&self, effect: Effect, value: Value) -> Result<Value, EffectValue> {
        match effect {
            Effect::Io(IoOp::Print(s)) => {
                println!("{}", s);
                Ok(Value::Unit)
            }
            Effect::Io(IoOp::ReadLine) => {
                let mut buf = String::new();
                self.stdin.read_line(&mut buf).map_err(|e| EffectValue::new(Effect::Io, e))?;
                Ok(Value::String(buf))
            }
            // ... other IO operations
            _ => Err(EffectValue::new(effect, value)),
        }
    }
}

// Async as Effect
pub struct AsyncEffectHandler;

impl AsyncEffectHandler {
    // Transform async/await into effect operations
    pub fn transform_async(fn_decl: &Function) -> Function;
    pub fn transform_await(expr: &Expression) -> EffectOp;
}

// Generator as Effect (yield)
pub struct GeneratorEffectHandler;

impl GeneratorEffectHandler {
    pub fn transform_generator(fn_decl: &Function) -> Function;
    pub fn transform_yield(expr: &Expression) -> EffectOp;
}

// Custom effect example
#[derive(Debug, Clone)]
pub enum CustomEffect {
    State(StateOp),
    Random(RandomOp),
    Log(LogLevel, String),
}

pub enum StateOp {
    Get,
    Set(Value),
}

pub enum RandomOp {
    Next(usize),
    Range(usize, usize),
}

pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
```

### 3.4 Effect Polymorphism

**File**: `omni-lang/compiler/src/effects/polymorphism.rs`

```rust
// Effect polymorphic functions
pub struct EffectPolyFn {
    type_params: Vec<TypeParam>,
    effect_params: Vec<EffectParam>,
    body: Expression,
    constraints: Vec<EffectConstraint>,
}

pub struct TypeParam {
    name: String,
    bounds: Vec<TraitBound>,
}

pub struct EffectParam {
    name: String,
    constraints: Vec<Effect>,
}

// Effect generic instantiation
pub fn instantiate_effect_poly(
    poly_fn: &EffectPolyFn,
    type_args: &[Type],
    effect_args: &[Effect],
) -> MonoFn;

// Effect trait (for generic effect handling)
pub trait EffectTrait {
    type Output;
    fn perform(self) -> Result<Self::Output, EffectValue>;
}

// Effect sets in generics
pub fn constrain_effects<T: EffectConstraint>(effects: &EffectSet) -> Result<EffectSet, EffectError>;
```

---

## PHASE 4: OWNERSHIP AND BORROWING

### 4.1 Polonius Borrow Checker (CRITICAL FIX)

**File**: `omni-lang/compiler/src/semantic/borrow_check.rs`

Implement Polonius algorithm:

```rust
use polonius_engine::{Algorithm, Input, Output};

// Polonius-based borrow checker
pub struct PoloniusChecker {
    algorithm: Algorithm,
    facts: PoloniusFacts,
    loan_graph: LoanGraph,
}

pub struct PoloniusFacts {
    // Core fact tables
    loan_issued: Vec<(Loan, Place, Location)>,
    loan_rejected: Vec<(Loan, Place, Location)>,
    loan_killed: Vec<(Loan, Location)>,
    loan_outlived: Vec<(Loan, Lifetime, Location)>,
    placeholder: Vec<(Placeholder, Place, Location)>,
    outlives: Vec<(Lifetime, Lifetime, Location)>,
    use_of_var_derefs_var: Vec<(Location, Place, Place)>,
    drop_of_var_derefs_var: Vec<(Location, Place, Place)>,
    move_error: Vec<(Location, Place, Place)>,
}

pub struct LoanGraph {
    nodes: Vec<LoanNode>,
    edges: Vec<LoanEdge>,
}

pub enum LoanNode {
    Loan(Loan),
    Place(Place),
    Variable(Variable),
}

pub enum LoanEdge {
    Issued,
    Reborrowed,
    Killed,
    Outlives,
}

// Input for Polonius
pub struct BorrowCheckInput {
    pub basic_blocks: Vec<BasicBlock>,
    pub place_desc: Vec<Place>,
    pub loans: Vec<Loan>,
    pub location_table: LocationTable,
    pub universal_regions: Vec<Region>,
}

pub struct BasicBlock {
    statements: Vec<Statement>,
    terminator: Terminator,
}

pub enum Statement {
    Assign(Place, Rvalue),
    Drop(Place),
    StorageDead(Place),
    StorageLive(Place),
    Nop,
}

pub enum Terminator {
    Return,
    Goto(BasicBlockId),
    If(Place, BasicBlockId, BasicBlockId),
    SwitchInt(Place, Vec<i128>, BasicBlockId),
    Call { args: Vec<Place>, target: BasicBlockId, cleanup: Option<BasicBlockId> },
    Assert { cond: Place, expected: bool, target: BasicBlockId },
}

pub enum Place {
    Local(LocalId),
    Field(Box<Place>, FieldId),
    Index(Box<Place>, Place),
    Deref(Box<Place>),
}

pub enum Rvalue {
    Use(Operand),
    Ref(Place, BorrowKind),
    BinaryOp(BinOp, Operand, Operand),
    UnaryOp(UnOp, Operand),
    Cast(Operand, Type),
}

pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
}

pub enum BorrowKind {
    Shared,
    Mutable,
    Unique,
}

// Run Polonius analysis
pub fn check_borrows(input: BorrowCheckInput) -> BorrowCheckResult {
    let mut engine = PoloniusEngine::new(Algorithm::Hybrid);
    let output = engine.compute(input);
    convert_output(output)
}

pub struct BorrowCheckResult {
    pub errors: Vec<BorrowError>,
    pub warnings: Vec<BorrowWarning>,
    pub liveness: LivenessAnalysis,
}

pub enum BorrowError {
    MoveWhileBorrowed { place: Place, loan: Loan },
    BorrowAfterMove { place: Place, loan: Loan },
    BorrowConflict { place: Place, loan_a: Loan, loan_b: Loan },
    UseAfterFree { place: Place, loan: Loan },
    BorrowNotValid { place: Place, loan: Loan },
}

pub struct LivenessAnalysis {
    pub live_vars: HashMap<Location, FxHashSet<Variable>>,
    pub drop_live: HashMap<Location, FxHashSet<Place>>,
}
```

### 4.2 Linear Types

**File**: `omni-lang/compiler/src/semantic/linear.rs`

```rust
// Linear type checking
pub struct LinearChecker {
    used_values: HashMap<ValueId, UsedState>,
    errors: Vec<LinearError>,
}

pub enum UsedState {
    Unused,
    UsedOnce,
    UsedMultiple,
    Moved,
    Dropped,
}

pub enum LinearError {
    ValueNotUsed { value: ValueId, location: Location },
    ValueUsedMultiple { value: ValueId, locations: Vec<Location> },
    LinearNotMoved { value: ValueId },
}

// Linear type modifier
pub struct LinearType {
    inner: Type,
}

// Check linear usage
pub fn check_linear_usage(body: &Block) -> Vec<LinearError>;
pub fn requires_linear(ty: &Type) -> bool;
```

### 4.3 Generational References

**File**: `omni-lang/compiler/src/semantic/generational.rs`

```rust
// Generational reference types
pub struct Gen<T> {
    generation: Generation,
    index: Index,
    _marker: PhantomData<T>,
}

pub struct Generation(pub u64);
pub struct Index(pub usize);

impl<T> Gen<T> {
    pub fn new(data: T) -> Self;
    pub fn get(&self) -> Option<&T>;
    pub fn get_mut(&mut self) -> Option<&mut T>;
    pub fn upgrade(&self) -> Option<Gen<T>>;
    pub fn generation(&self) -> Generation;
}

// SlotMap for efficient generational storage
pub struct SlotMap<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<SlotId>,
    next_generation: Generation,
}

pub struct Slot<T> {
    data: Option<T>,
    generation: Generation,
    state: SlotState,
}

pub enum SlotState {
    Empty,
    Occupied,
    Deleted,
}

impl<T> SlotMap<T> {
    pub fn insert(&mut self, data: T) -> Gen<T>;
    pub fn remove(&mut self, gen: &Gen<T>) -> Option<T>;
    pub fn get(&self, gen: &Gen<T>) -> Option<&T>;
    pub fn get_mut(&mut self, gen: &Gen<T>) -> Option<&mut T>;
}
```

### 4.4 Arena Allocation

**File**: `omni-lang/compiler/src/memory/arena.rs`

```rust
// Arena allocator
pub struct Arena<T> {
    ptr: *mut T,
    capacity: usize,
    len: usize,
}

impl<T> Arena<T> {
    pub fn new() -> Self;
    pub fn with_capacity(cap: usize) -> Self;
    pub fn alloc(&mut self, value: T) -> &mut T;
    pub fn alloc_slice(&mut self, values: &[T]) -> &mut [T];
    pub fn reset(&mut self);
}

// Typed arena for specific types
pub struct TypedArena<T> {
    chunk: Vec<T>,
    next_chunk: Vec<T>,
}

impl<T> TypedArena<T> {
    pub fn new() -> Self;
    pub fn alloc(&self, value: T) -> &T;
    pub fn alloc_iter(&self, iter: impl Iterator<Item = T>) -> &[T];
}

// Scratch arena for temporary allocations
pub struct ScratchArena {
    buffer: Vec<u8>,
    ptr: usize,
}

impl ScratchArena {
    pub fn new(size: usize) -> Self;
    pub fn alloc<T>(&mut self, value: T) -> &mut T;
    pub fn write(&mut self, bytes: &[u8]);
    pub fn reset(&mut self);
}
```

---

## PHASE 5: CONCURRENCY

### 5.1 Structured Concurrency

**File**: `omni-lang/compiler/src/concurrency/structured.rs`

```rust
// Spawn scope for structured concurrency
pub fn spawn_scope<F, R>(f: F) -> R
where
    F: FnOnce(&Scope) -> R,
    F: Send,
    R: Send;

pub struct Scope {
    children: Vec<ScopedTask>,
}

impl Scope {
    pub fn spawn<F, T>(&self, f: F) -> JoinHandle<T>
    where
        F: FnOnce(&Scope) -> T,
        F: Send + 'static,
        T: Send + 'static;

    pub fn spawn_async<F, T>(&self, f: F) -> AsyncJoinHandle<T>
    where
        F: Future<Output = T>,
        F: Send + 'static,
        T: Send + 'static;
}

pub trait Spawn {
    fn spawn(&self, task: Task) -> JoinHandle<()>;
    fn spawn_local(&self, task: Task) -> JoinHandle<()>;
}

// Task that cannot outlive scope
pub struct ScopedTask<T> {
    handle: JoinHandle<T>,
    _lifetime: PhantomData<&'scope ()>,
}

// Cancel token for explicit async cancellation
pub struct CancelToken {
    cancelled: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self;
    pub fn cancel(&self);
    pub fn is_cancelled(&self) -> bool;
    pub fn until_cancelled(&self) -> Cancelled;
}

// Global spawn capability
pub struct GlobalSpawnCap {
    // Capability token for spawn_global
}

impl GlobalSpawnCap {
    pub fn require() -> Result<Self, CapabilityError>;
    pub fn spawn(&self, task: Task);
}
```

### 5.2 Actor Model

**File**: `omni-lang/compiler/src/concurrency/actor.rs`

```rust
// Actor definition
pub struct Actor<T: ActorState> {
    mailbox: Mailbox<T::Message>,
    state: T,
    supervision: Option<SupervisorRef>,
}

pub trait ActorState: Sized + Send {
    type Message: Message;
    fn handle(&mut self, msg: Self::Message) -> impl Future<Output = ()>;
}

pub trait Message: Send + Sized {
    type Reply: Send;
}

// Typed mailbox
pub struct Mailbox<M: Message> {
    sender: mpsc::Sender<M>,
    receiver: mpsc::Receiver<M>,
}

impl<M: Message> Mailbox<M> {
    pub fn send(&self, msg: M) -> Result<(), SendError>;
    pub fn try_recv(&self) -> Option<M>;
    pub fn recv(&self) -> impl Future<Output = M>;
}

// Actor spawn and management
pub fn spawn_actor<T: ActorState>(state: T) -> ActorRef<T::Message>;
pub fn spawn_actor_with_supervisor<T: ActorState>(state: T, supervisor: &Supervisor) -> ActorRef<T::Message>;

// Supervision tree
pub struct Supervisor {
    children: Vec<ActorRef<AnyMessage>>,
    strategy: RestartStrategy,
}

pub enum RestartStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
    Never,
}
```

### 5.3 Typed Channels

**File**: `omni-lang/compiler/src/concurrency/channels.rs`

```rust
// Multi-producer single-consumer channel
pub struct Channel<T> {
    buffer: RingBuffer<T>,
    senders: Vec<Sender<T>>,
    receiver: Receiver<T>,
}

pub struct Sender<T> {
    channel: Arc<Channel<T>>,
}

pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError>;
    pub fn try_send(&self, value: T) -> Result<(), TrySendError>;
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> impl Future<Output = T>;
    pub fn try_recv(&self) -> Option<T>;
}

// Bounded channel with backpressure
pub struct BoundedChannel<T> {
    buffer: RingBuffer<T>,
    capacity: usize,
}

impl<T> BoundedChannel<T> {
    pub fn new(capacity: usize) -> Self;
    pub fn send(&self, value: T) -> impl Future<Output = Result<(), ChannelFull>>;
}

// Broadcast channel for one-to-many
pub struct BroadcastChannel<T> {
    subscribers: Vec<Arc<dyn Subscriber<T>>>,
}
```

---

## PHASE 6: MODULE AND PACKAGE SYSTEM

### 6.1 Module System

**File**: `omni-lang/compiler/src/modules/mod.rs`

```rust
// Module system
pub struct Module {
    name: ModuleName,
    items: Vec<ModuleItem>,
    visibility: Visibility,
    submodules: HashMap<String, Module>,
}

pub struct ModuleName {
    components: Vec<String>,
}

pub enum ModuleItem {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Impl(Impl),
    TypeAlias(TypeAlias),
    Constant(Constant),
    Module(Module),
}

// Visibility levels
pub enum Visibility {
    Private,
    PubMod,           // pub(mod)
    PubPkg,           // pub(pkg)
    Pub,              // pub
    PubCap(Capability), // pub(cap: X)
    PubFriend(ModuleName), // pub(friend: module)
}

pub fn resolve_visibility(item: &ModuleItem, target: &Module) -> Result<Visibility, VisibilityError>;

// Import resolution
pub fn resolve_import(path: &[String], current: &Module) -> Result<ResolvedImport, ImportError>;

pub struct ResolvedImport {
    target_module: ModuleId,
    items: Vec<ImportedItem>,
    is_public: bool,
}

// Circular dependency detection
pub fn check_circular_deps(module: &Module) -> Result<(), CircularDepError>;
```

### 6.2 Package Manager

**File**: `omni-lang/compiler/src/packages/mod.rs`

```rust
// Package manifest
pub struct PackageManifest {
    package: PackageSection,
    dependencies: Dependencies,
    dev_dependencies: Dependencies,
    build_dependencies: Dependencies,
    workspace: Option<WorkspaceConfig>,
    features: HashMap<String, Vec<String>>,
}

pub struct PackageSection {
    name: PackageName,
    version: Version,
    edition: String,
    authors: Vec<String>,
    license: Option<String>,
    description: Option<String>,
}

pub struct Dependencies {
    deps: HashMap<PackageName, VersionSpec>,
}

pub enum VersionSpec {
    Exact(Version),
    Caret(Version),
    Tilde(Version),
    Star,
    Range(VersionRange),
}

// PubGrub resolver
pub struct PackageResolver {
    packages: HashMap<PackageName, Package>,
    cache: ResolverCache,
}

pub fn resolve_dependencies(
    manifest: &PackageManifest,
    registry: &PackageRegistry,
) -> Result<ResolvedDeps, ResolverError>;

// Lockfile
pub struct Lockfile {
    version: u32,
    packages: Vec<LockedPackage>,
    metadata: LockfileMetadata,
}

pub struct LockedPackage {
    name: PackageName,
    version: Version,
    source: Source,
    dependencies: Vec<(PackageName, Version)>,
    checksums: Checksums,
}

// Workspace support
pub struct Workspace {
    root: PathBuf,
    members: Vec<WorkspaceMember>,
    shared_dependencies: HashMap<PackageName, Version>,
}
```

### 6.3 Comptime Build Scripts

**File**: `omni-lang/compiler/src/build/mod.rs`

```rust
// Build script execution
pub struct BuildScript {
    source: String,
    ctx: BuildContext,
}

pub struct BuildContext {
    target: TargetTriple,
    opt_level: OptLevel,
    env: HashMap<String, String>,
    out_dir: PathBuf,
    manifest_dir: PathBuf,
}

pub fn run_build_script(path: &Path, ctx: &BuildContext) -> BuildScriptResult;

pub struct BuildScriptResult {
    // Links to add
    links: Vec<LinkFlag>,
    
    // Rust flags
    rust_flags: Vec<RustFlag>,
    
    // Environment variables to set
    env_vars: HashMap<String, String>,
    
    // Generated files
    generated_files: Vec<PathBuf>,
    
    // Dependencies
    build_deps: Vec<PackageName>,
}

// Comptime evaluation
pub trait Comptime {
    fn evaluate(&self, ctx: &ComptimeContext) -> ConstantValue;
}

pub fn evaluate_comptime_expr(expr: &Expression, ctx: &ComptimeContext) -> ConstantValue;
```

---

## PHASE 7: ERROR HANDLING

### 7.1 Structured Error System

**File**: `omni-lang/compiler/src/diagnostics/mod.rs`

```rust
// Stable error codes
pub struct ErrorCode(pub u32);

impl ErrorCode {
    // Type errors (E0100-E0199)
    pub const E0100: ErrorCode = ErrorCode(100); // Type mismatch
    pub const E0101: ErrorCode = ErrorCode(101); // Cannot infer type
    pub const E0102: ErrorCode = ErrorCode(102); // Trait not satisfied
    pub const E0103: ErrorCode = ErrorCode(103); // Cyclic type definition
    
    // Borrow errors (E0200-E0299)
    pub const E0200: ErrorCode = ErrorCode(200); // Use after move
    pub const E0201: ErrorCode = ErrorCode(201); // Mutable borrow in immutable context
    pub const E0202: ErrorCode = ErrorCode(202); // Borrow conflicts with move
    pub const E0203: ErrorCode = ErrorCode(203); // Invalid borrow
    
    // Effect errors (E0300-E0399)
    pub const E0300: ErrorCode = ErrorCode(300); // Missing effect capability
    pub const E0301: ErrorCode = ErrorCode(301); // Effect not handled
    pub const E0302: ErrorCode = ErrorCode(302); // Effect type mismatch
    
    // Module errors (E0400-E0499)
    pub const E0400: ErrorCode = ErrorCode(400); // Unresolved import
    pub const E0401: ErrorCode = ErrorCode(401); // Circular dependency
    pub const E0402: ErrorCode = ErrorCode(402); // Visibility violation
    
    // Parse errors (E0500-E0599)
    pub const E0500: ErrorCode = ErrorCode(500); // Syntax error
    pub const E0501: ErrorCode = ErrorCode(501); // Expected token
    pub const E0502: ErrorCode = ErrorCode(502); // Indentation error
    
    // ... more error codes
}

// Diagnostic structure
pub struct Diagnostic {
    pub code: ErrorCode,
    pub level: DiagnosticLevel,
    pub message: String,
    pub primary_span: Span,
    pub secondary_spans: Vec<Span>,
    pub help_note: Option<String>,
    pub suggested_fix: Option<CodeFix>,
    pub region: Option<SourceRegion>,
}

pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

pub struct CodeFix {
    pub replacement: Replacement,
    pub applicability: Applicability,
    pub message: String,
}

pub enum Applicability {
    MachineApplicable,
    HasPlaceholders,
    MaybeIncorrect,
    Unspecified,
}

// Machine-readable output
pub fn emit_json(diagnostics: &[Diagnostic]) -> String;
pub fn emit_stable(diagnostics: &[Diagnostic]) -> String;

// "Did you mean?" suggestions
pub fn suggest_similar(name: &str, candidates: &[String], threshold: usize) -> Option<String>;

// Internationalization
pub struct DiagnosticCatalog {
    messages: HashMap<ErrorCode, HashMap<String, String>>,
}

impl DiagnosticCatalog {
    pub fn translate(&self, code: ErrorCode, locale: &str) -> String;
}
```

### 7.2 Error Recovery

**File**: `omni-lang/compiler/src/diagnostics/recovery.rs`

```rust
// Error recovery strategies
pub struct ErrorRecovery {
    sync_tokens: FxHashSet<TokenType>,
    max_errors: usize,
    recovery_enabled: bool,
}

impl ErrorRecovery {
    pub fn new() -> Self;
    pub fn with_sync(sync_tokens: FxHashSet<TokenType>) -> Self;
    pub fn recover(&self, parser: &mut Parser, error: &ParseError) -> Option<()>;
}

// Synchronization points
pub const STATEMENT_SYNC: &[TokenType] = &[
    TokenType::Let,
    TokenType::Fn,
    TokenType::Struct,
    TokenType::If,
    TokenType::While,
    TokenType::Return,
    TokenType::Eof,
];

pub const EXPRESSION_SYNC: &[TokenType] = &[
    TokenType::LeftBrace,
    TokenType::RightBrace,
    TokenType::Semicolon,
    TokenType::Eof,
];
```

---

## PHASE 8: STANDARD LIBRARY

### 8.1 Core Traits

**File**: `omni-lang/omni/stdlib/core.omni`

```omni
// Core traits that must be implemented
trait Copy: // Types that can be copied bit-for-bit

trait Clone: // Types that can be cloned
    fn clone(&self) -> own Self

trait Drop: // Custom destructor logic
    fn drop(&mut self)

trait Sized: // Types with known size at compile time

trait Send: // Can be transferred between threads

trait Sync: // Can be safely shared between threads

trait Eq: // Equality comparison
    fn eq(&self, other: &Self) -> bool

trait PartialEq<Rhs = Self>: // Partial equality
    fn eq(&self, other: &Rhs) -> bool
    fn ne(&self, other: &Rhs) -> bool

trait Ord: // Total ordering
    fn cmp(&self, other: &Self) -> Ordering

trait PartialOrd<Rhs = Self>: // Partial ordering
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>
    fn lt(&self, other: &Rhs) -> bool
    fn le(&self, other: &Rhs) -> bool
    fn gt(&self, other: &Rhs) -> bool
    fn ge(&self, other: &Rhs) -> bool

trait Hash: // Hashable types
    fn hash<H: Hasher>(&self, state: &mut H)

trait Default: // Default value
    fn default() -> Self

trait Display: // User-facing display
    fn fmt(&self, f: &mut Formatter) -> bool

trait Debug: // Debug formatting
    fn fmt(&self, f: &mut Formatter) -> bool

trait From<T>: // Conversion from
    fn from(other: T) -> Self

trait Into<T>: // Conversion into
    fn into(self) -> T

trait TryFrom<T>: // Fallible conversion from
    fn try_from(other: T) -> Result<Self, ConversionError>

trait TryInto<T>: // Fallible conversion into
    fn try_into(self) -> Result<T, ConversionError>

trait Iterator:
    type Item
    fn next(&mut self) -> Option<Self::Item>

trait IntoIterator:
    type Item
    fn into_iter(&self) -> Self::IntoIter

trait FromIterator<T>: // Collect into
    fn from_iter<T: IntoIterator<Item = T>>(iter: T) -> Self

trait Fn<Args...>: // Callable (immutable)
    fn call(&self, args: Args...) -> Self::Output

trait FnMut<Args...>: // Callable (mutable)
    fn call_mut(&mut self, args: Args...) -> Self::Output

trait FnOnce<Args...>: // Callable (consuming)
    fn call_once(self, args: Args...) -> Self::Output

// Try trait for ? operator (v2.0 extensible)
trait Try:
    type Ok
    type Error
    fn from_ok(value: Self::Ok) -> Self
    fn from_err(error: Self::Error) -> Self
    fn into_ok(self) -> Self::Ok
    fn into_err(self) -> Self::Error
```

### 8.2 Option and Result

```omni
enum Option<T>:
    Some(T)
    None

enum Result<T, E>:
    Ok(T)
    Err(E)
```

### 8.3 Collections

**File**: `omni-lang/omni/stdlib/collections.omni`

```omni
// Vector<T> - growable array
struct Vector<T>:
    data: &[T]
    len: usize
    capacity: usize

impl<T> Vector<T>:
    fn new() -> own Self
    fn with_capacity(cap: usize) -> own Self
    fn push(&mut self, value: T)
    fn pop(&mut self) -> Option<T>
    fn get(&self, index: usize) -> Option<&T>
    fn get_mut(&mut self, index: usize) -> Option<&mut T>
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn capacity(&self) -> usize
    fn clear(&mut self)
    fn resize(&mut self, new_len: usize, value: T)
    fn extend(&mut self, other: &Vector<T>)

// HashMap<K, V> - hash table
struct HashMap<K, V>:
    // ...

impl<K, V> HashMap<K, V> where K: Hash + Eq:
    fn new() -> own Self
    fn insert(&mut self, key: K, value: V) -> Option<V>
    fn get(&self, key: &K) -> Option<&V>
    fn remove(&mut self, key: &K) -> Option<V>
    fn contains(&self, key: &K) -> bool
    fn len(&self) -> usize

// HashSet<T>
struct HashSet<T>:
    // ...

// BTreeMap<K, V>
struct BTreeMap<K, V> where K: Ord:
    // ...

// VecDeque<T>
struct VecDeque<T>:
    // ...
```

### 8.4 String Types

```omni
// String - owned UTF-8 string
struct String:
    data: Vector<u8>
    len: usize

impl String:
    fn new() -> own Self
    fn from(s: &str) -> own Self
    fn from_utf8(bytes: Vector<u8>) -> Result<own String, Utf8Error>
    fn as_str(&self) -> &str
    fn as_bytes(&self) -> &[u8]
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn push(&mut self, ch: char)
    fn push_str(&mut self, s: &str)
    fn pop(&mut self) -> Option<char>
    fn clear(&mut self)
    fn split(&self, sep: &str) -> Vec<&str>
    fn join(&self, sep: &str) -> String
    fn contains(&self, substr: &str) -> bool
    fn starts_with(&self, prefix: &str) -> bool
    fn ends_with(&self, suffix: &str) -> bool
    fn trim(&self) -> &str
    fn to_lowercase(&self) -> String
    fn to_uppercase(&self) -> String
    fn parse<T: FromStr>(&self) -> Result<T, ParseError>

// String interpolation (v2.0)
fn f"Hello {name}!" -> String  // Display
fn d"Value: {value:?}" -> String  // Debug
```

### 8.5 Tensor Module

**File**: `omni-lang/omni/stdlib/tensor.omni`

```omni
// Tensor<T, Shape>
struct Tensor<T, const N: usize>:
    data: Vector<T>
    shape: [usize; N]
    strides: [usize; N]

impl<T: Clone + Default, const N: usize> Tensor<T, N>:
    fn new(shape: [usize; N]) -> own Self
    fn zeros(shape: [usize; N]) -> own Self
    fn ones(shape: [usize; N]) -> own Self
    fn from_data(data: Vector<T>, shape: [usize; N]) -> own Self
    
    fn shape(&self) -> &[usize; N]
    fn len(&self) -> usize
    fn reshape(&mut self, new_shape: [usize; N])
    fn transpose(&self) -> Tensor<T, N>
    fn permute(&self, axes: [usize; N]) -> Tensor<T, N>
    
    // Element-wise operations
    fn add(&self, other: &Tensor<T, N>) -> Tensor<T, N>
    fn sub(&self, other: &Tensor<T, N>) -> Tensor<T, N>
    fn mul(&self, other: &Tensor<T, N>) -> Tensor<T, N>
    fn div(&self, other: &Tensor<T, N>) -> Tensor<T, N>
    
    // Matrix operations
    fn dot(&self, other: &Tensor<T, N>) -> Tensor<T, N>
    fn matmul(&self, other: &Tensor<T, N>) -> Tensor<T, N>
    
    // Reduction
    fn sum(&self, axis: Option<usize>) -> Tensor<T, N-1>
    fn mean(&self, axis: Option<usize>) -> Tensor<T, N-1>
    fn max(&self, axis: Option<usize>) -> Tensor<T, N-1>
    fn min(&self, axis: Option<usize>) -> Tensor<T, N-1>

// SIMD dispatch
impl<T: Simd + Clone> Tensor<T, N>:
    fn simd_add(&self, other: &Self) -> Self
    fn simd_mul(&self, other: &Self) -> Self
```

---

## PHASE 9: COMPILATION PIPELINE

### 9.1 Complete IR Definition

**File**: `omni-lang/compiler/src/ir/mod.rs`

```rust
// Mid-level IR (MIR) - Control Flow Graph based
pub struct MIR {
    basic_blocks: Vec<BasicBlock>,
    local_decls: Vec<LocalDecl>,
    pub span_info: SpanMap,
}

pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

pub enum Statement {
    Assign(Place, Rvalue),
    SetDiscriminant { place: Place, variant_index: usize },
    Deinit(Place),
    StorageLive(Local),
    StorageDead(Local),
    Nop,
}

pub enum Terminator {
    Return { value: Operand },
    Goto { target: BasicBlockId },
    SwitchInt { discr: Operand, values: &[i128], targets: &[BasicBlockId], otherwise: BasicBlockId },
    Call { func: Operand, args: Vec<Operand>, target: BasicBlockId, cleanup: BasicBlockId },
    Assert { cond: Operand, expected: bool, target: BasicBlockId, msg: &str },
    Drop { place: Place, target: BasicBlockId, unwind: BasicBlockId },
    Unwind,
    Unreachable,
    Resume,
}

pub struct Place {
    local: Local,
    projection: Vec<Projection>,
}

pub enum Projection {
    Field(usize),
    Index(Operand),
    Deref,
    Subslice { from: usize, to: usize },
}

pub enum Rvalue {
    Use(Operand),
    Ref(Place, BorrowKind),
    BinaryOp(BinOp, Operand, Operand),
    UnaryOp(UnOp, Operand),
    CheckedBinaryOp(BinOp, Operand, Operand),
    Cast(CastKind, Operand),
    Repeat(Operand, Operand),
    Len(Place),
    Discriminant(Place),
    Aggregate(AggregateKind, Vec<Operand>),
}

pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
}

pub struct LocalDecl {
    mutability: Mutability,
    ty: Type,
    align: Align,
}

pub struct Local(u32);
pub type BasicBlockId = Index;

// Liveness analysis
pub fn compute_liveness(mir: &MIR) -> LivenessResult;

pub struct LivenessResult {
    pub per_block: HashMap<BasicBlockId, BlockLiveness>,
    pub per_local: HashMap<Local, Vec<Location>>,
}
```

### 9.2 Complete Optimization Pipeline

**File**: `omni-lang/compiler/src/optimizer/mod.rs`

```rust
// Optimization passes
pub trait OptimizationPass {
    fn name(&self) -> &str;
    fn run(&mut self, mir: &mut MIR) -> bool;
}

// Core passes
pub struct ConstPropagation;
pub struct DCE;
pub struct CopyPropagation;
pub struct AlgebraicSimplification;
pub struct LoopInvariantCodeMotion;
pub struct LoopUnrolling;
pub struct Inline;
pub struct Mem2Reg;
pub struct GVN;
pub struct DSE;

// Analysis passes
pub struct LivenessAnalysis;
pub struct DominatorAnalysis;
pub struct LoopAnalysis;
pub struct AliasAnalysis;

// Optimization pipeline
pub struct Optimizer {
    passes: Vec<Box<dyn OptimizationPass>>,
    level: OptLevel,
}

impl Optimizer {
    pub fn new(level: OptLevel) -> Self;
    pub fn add_pass(&mut self, pass: Box<dyn OptimizationPass>);
    pub fn run(&mut self, mir: &mut MIR);
}

// Opt-level presets
pub fn default_passes(level: OptLevel) -> Vec<Box<dyn OptimizationPass>>;
```

### 9.3 Code Generation

**File**: `omni-lang/compiler/src/codegen/mod.rs`

```rust
// OVM bytecode generation
pub struct OVMCompiler {
    config: CodegenConfig,
}

impl OVMCompiler {
    pub fn compile_module(&self, mir: &MIR) -> OVMModule;
    pub fn compile_function(&self, fn_: &Function) -> OVMFunction;
}

pub struct OVMModule {
    functions: Vec<OVMFunction>,
    globals: Vec<OVMGlobal>,
    strings: Vec<String>,
}

pub enum OVMInstruction {
    // Stack operations
    Push(OVMValue),
    Pop,
    Dup,
    Swap,
    
    // Local variables
    LoadLocal(u32),
    StoreLocal(u32),
    
    // Control flow
    Jump(OVMLabel),
    JumpIf(OVMLabel),
    Call(u32),
    Ret,
    
    // Operations
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or, Not,
    
    // Memory
    Load(OVMAddress),
    Store(OVMAddress),
    Allocate(OVMType),
    Deallocate,
    
    // Function calls
    CallNative(String),
    
    // Effect operations
    PerformEffect(u32),
    HandleEffect(u32),
    Resume(u32),
}

// Native code generation (LLVM)
pub struct LLVMCompiler {
    context: LLVMContext,
    builder: LLVMBuilder,
    module: LLVMModule,
}

impl LLVMCompiler {
    pub fn compile(&self, mir: &MIR) -> LLVMModule;
    pub fn write_object_file(&self, path: &Path);
}

// Linker
pub struct Linker {
    inputs: Vec<LinkInput>,
    output: LinkOutput,
}

pub enum LinkInput {
    Object(Vec<u8>),
    Library(Library),
}

pub enum LinkOutput {
    Executable,
    DynamicLibrary,
    StaticLibrary,
}

pub fn link(inputs: &[LinkInput], output: &Path, target: TargetTriple) -> Result<()>;
```

---

## PHASE 10: TOOLING

### 10.1 LSP Server

**File**: `omni-lang/tools/omni-lsp/src/main.rs`

```rust
// LSP server implementation
pub struct LspServer {
    state: LspState,
    text_documents: HashMap<Url, TextDocument>,
}

impl LspServer {
    pub fn new() -> Self;
    pub fn handle_request(&mut self, method: &str, params: serde_json::Value) -> Response;
}

// Implement all LSP methods
pub fn text_document_did_open(params: DidOpenTextDocumentParams) -> Response;
pub fn text_document_did_change(params: DidChangeTextDocumentParams) -> Response;
pub fn text_document_completion(params: CompletionParams) -> Response;
pub fn text_document_definition(params: DefinitionParams) -> Response;
pub fn text_document_hover(params: HoverParams) -> Response;
pub fn text_document_signature_help(params: SignatureHelpParams) -> Response;
pub fn text_document_references(params: ReferencesParams) -> Response;
pub fn text_document_rename(params: RenameParams) -> Response;
pub fn text_document_formatting(params: DocumentFormattingParams) -> Response;

// Semantic features
pub fn text_document_semantic_tokens(params: SemanticTokensParams) -> Response;
pub fn text_document_inlay_hint(params: InlayHintParams) -> Response;
pub fn text_document_code_action(params: CodeActionParams) -> Response;

// Effect explorer (v2.0)
pub fn text_document_effect_explore(params: EffectExploreParams) -> Response;
pub fn text_document_effect_signature(params: EffectSignatureParams) -> Response;

// Borrow visualization
pub fn text_document_borrow_visualize(params: BorrowVisualizeParams) -> Response;
```

### 10.2 Formatter

**File**: `omni-lang/tools/omni-fmt/src/formatter.rs`

```rust
pub struct Formatter {
    config: FormatConfig,
    line_width: usize,
    indent_size: usize,
}

pub struct FormatConfig {
    pub indent_style: IndentStyle,
    pub line_width: usize,
    pub tab_width: usize,
    pub insert_final_newline: bool,
    pub normalize_newlines: bool,
}

pub enum IndentStyle {
    Spaces(usize),
    Tabs,
}

impl Formatter {
    pub fn new(config: FormatConfig) -> Self;
    pub fn format_module(&self, module: &CST) -> String;
    pub fn format(&self, source: &str) -> Result<String, FormatError>;
    pub fn format_file(&self, path: &Path) -> Result<(), FormatError>;
    pub fn check(&self, source: &str) -> bool;  // Idempotent check
}

// Specific formatting rules
pub fn format_function(&self, func: &Function) -> String;
pub fn format_struct(&self, struct_: &Struct) -> String;
pub fn format_match(&self, match_: &Match) -> String;
pub fn format_effects(&self, effects: &EffectSet) -> String;
```

### 10.3 Auto-fix

**File**: `omni-lang/tools/omni-fix/src/main.rs`

```rust
pub struct Fixer {
    fixes: FixRegistry,
}

pub struct FixRegistry {
    fixes: HashMap<ErrorCode, Vec<Box<dyn Fix>>>,
}

pub trait Fix {
    fn code(&self) -> ErrorCode;
    fn applicability(&self) -> Applicability;
    fn generate(&self, diag: &Diagnostic, source: &SourceFile) -> Option<Fix>;
}

pub fn run_fixes(diagnostics: &[Diagnostic], source: &SourceFile) -> Vec<FixedFile>;

// Common fixes
pub struct TypeAnnotationFix;
pub struct MissingImportFix;
pub struct RenameFix;
pub struct AddParensFix;
pub struct RemoveUnusedFix;
pub struct AddEffectFix;
```

---

## PHASE 11: TESTING

### 11.1 Test Annotations

**File**: `omni-lang/compiler/src/testing/mod.rs`

```rust
// Test annotations
#[attribute]
pub struct Test {
    name: Option<String>,
    should_panic: bool,
    ignore: bool,
    timeout_ms: Option<u64>,
}

#[attribute]
pub struct TestShouldPanic {
    expected: Option<String>,
}

#[attribute]
pub struct TestIgnore {
    reason: String,
}

// Effect-aware testing
#[attribute]
pub struct EffectTest {
    mock_effects: Vec<(Effect, Box<dyn EffectMock>)>,
}

pub trait EffectMock {
    fn mock(&self, effect: Effect, args: Vec<Value>) -> Value;
}

// Contract annotations
#[attribute]
pub struct Requires {
    condition: String,  // Expression as string
}

#[attribute]
pub struct Ensures {
    condition: String,
}

#[attribute]
pub struct Invariant {
    condition: String,
}

// Test runner
pub fn discover_tests(module: &Module) -> Vec<TestCase>;
pub fn run_tests(tests: &[TestCase], config: &TestConfig) -> TestResult;

pub struct TestConfig {
    parallel: bool,
    workers: usize,
    filter: Option<String>,
    show_output: bool,
}
```

---

## PHASE 12: SECURITY AND CAPABILITIES

### 12.1 Capability System

**File**: `omni-lang/compiler/src/security/capabilities.rs`

```rust
// Capability tokens
pub struct Capability {
    name: CapabilityName,
    resource: Option<Resource>,
}

pub enum CapabilityName {
    FilesystemAccess,
    NetworkAccess,
    ProcessSpawn,
    EnvironmentAccess,
    Random,
    Time,
    // ... more
}

pub struct Resource {
    path: Option<PathBuf>,
    restrictions: Vec<Restriction>,
}

pub struct Restriction {
    read: bool,
    write: bool,
    execute: bool,
}

// Capability checking
pub fn check_capability(cap: &Capability) -> Result<(), CapabilityError>;

// Runtime capability enforcement
pub struct CapabilityGuard {
    granted: Vec<Capability>,
    revoked: Vec<Capability>,
}

impl CapabilityGuard {
    pub fn new(caps: Vec<Capability>) -> Self;
    pub fn check(&self, required: &Capability) -> bool;
    pub fn delegate(&self, to: &Capability) -> Capability;
    pub fn revoke(&mut self, cap: &Capability);
}
```

### 12.2 Sandboxing

**File**: `omni-lang/compiler/src/security/sandbox.rs`

```rust
// Sandboxed execution
pub struct Sandbox {
    limits: ResourceLimits,
    capabilities: Vec<Capability>,
}

pub struct ResourceLimits {
    max_memory: u64,
    max_cpu_time: u64,
    max_file_size: u64,
    max_open_files: usize,
}

impl Sandbox {
    pub fn new() -> Self;
    pub fn with_limits(limits: ResourceLimits) -> Self;
    pub fn with_capabilities(caps: Vec<Capability>) -> Self;
    pub fn execute<F, R>(&self, f: F) -> Result<R, SandboxError>
    where
        F: FnOnce() -> R + Send;
}

// FFI sandboxing
pub struct FfiSandbox {
    stack: ForeignStack,
}

impl FfiSandbox {
    pub fn new() -> Self;
    pub fn call<F, R>(&self, f: F, args: &[Value]) -> Result<R, FfiError>
    where
        F: Fn(*const ()) -> R;
}
```

---

## PHASE 13: SELF-HOSTING

### 13.1 Bootstrap Pipeline

**File**: `omni-lang/compiler/src/bootstrap/mod.rs`

```rust
// Multi-stage bootstrap
pub struct BootstrapPipeline {
    stage0: Stage0,
    stage1: Stage1,
    stage2: Stage2,
}

pub struct Stage0 {
    compiler: PathBuf,  // Rust-based compiler
}

pub struct Stage1 {
    compiler: PathBuf,  // Partial Omni compiler
    source: PathBuf,
}

pub struct Stage2 {
    compiler: PathBuf,  // Full Omni compiler (compiled by stage1)
    source: PathBuf,
}

impl BootstrapPipeline {
    pub fn new() -> Self;
    pub fn build_stage0(&self) -> Result<Stage0, BuildError>;
    pub fn build_stage1(&self, stage0: &Stage0) -> Result<Stage1, BuildError>;
    pub fn build_stage2(&self, stage1: &Stage1) -> Result<Stage2, BuildError>;
    pub fn verify_fixpoint(&self, stage1: &Stage1, stage2: &Stage2) -> Result<(), VerifyError>;
}

// Stage 1 -> Stage 2 comparison
pub fn compare_outputs(stage1: &Module, stage2: &Module) -> ComparisonResult;

pub fn verify_equivalence(
    stage1_ir: &IR,
    stage2_ir: &IR,
) -> Result<(), EquivalenceError>;
```

### 13.2 Module Migration Plan

For self-hosting, implement modules in this order:
1. Lexer
2. Parser
3. AST
4. Name Resolution
5. Type Inference
6. Trait System
7. Effect System
8. MIR
9. Borrow Checker
10. Optimizer
11. Codegen

---

## PHASE 14: HELIOS FRAMEWORK (NOT YET)

**DO NOT IMPLEMENT** until Phase 7+ is complete.

Current HELIOS in `helios-framework/` should be removed or clearly marked as premature experimental code.

---

## IMPLEMENTATION ORDER

### Immediate (Week 1-2)
1. **Fix borrow checker** to use Polonius
2. **Add INDENT/DEDENT** to lexer
3. **Add effect annotation** parsing
4. **Update documentation** to reflect actual self-hosting status

### Short-term (Week 3-8)
1. Implement complete effect system
2. Implement effect inference
3. Add pattern matching to parser
4. Implement CST
5. Add linear types
6. Add generational references

### Medium-term (Week 9-16)
1. Complete standard library
2. Implement `@test` annotation
3. Complete LSP
4. Implement `omni fix`
5. Add structured concurrency
6. Implement capability system

### Long-term (Month 5+)
1. Complete self-hosting
2. MLIR integration
3. GPU backend
4. FFI system

---

## QUALITY STANDARDS

All implementations must meet:
- **No `unwrap()` in library code** (except tests)
- **No `todo!()` in code that ships**
- **`cargo clippy --all-targets --all-features` passes**
- **`cargo fmt --check` passes**
- **All public items documented**
- **Unit tests for all new functionality**
- **Integration tests for pipeline**
- **Property-based tests where applicable**

---

## FILES TO CREATE/MODIFY

### New Files to Create
1. `compiler/src/effects/mod.rs`
2. `compiler/src/effects/handlers.rs`
3. `compiler/src/effects/builtins.rs`
4. `compiler/src/effects/polymorphism.rs`
5. `compiler/src/semantic/linear.rs`
6. `compiler/src/semantic/generational.rs`
7. `compiler/src/memory/arena.rs`
8. `compiler/src/concurrency/structured.rs`
9. `compiler/src/concurrency/actor.rs`
10. `compiler/src/concurrency/channels.rs`
11. `compiler/src/modules/mod.rs`
12. `compiler/src/packages/mod.rs`
13. `compiler/src/build/mod.rs`
14. `compiler/src/security/capabilities.rs`
15. `compiler/src/security/sandbox.rs`
16. `compiler/src/bootstrap/mod.rs`
17. `stdlib/core.omni` (restore full)
18. `stdlib/collections.omni`
19. `stdlib/tensor.omni`

### Major Modifications Required
1. `lexer.rs` - Add INDENT/DEDENT, effect tokens, string interpolation
2. `parser.rs` - Complete rewrite for all syntax features
3. `ast.rs` - Add CST support, all node types
4. `borrow_check.rs` - Replace with Polonius implementation
5. `type_inference.rs` - Add bidirectional, trait bounds, effect inference
6. `resolver.rs` - Add module resolution
7. `main.rs` - Add all pipeline stages

---

This is the complete implementation roadmap. Execute each phase in order, ensuring dependencies are satisfied before moving to subsequent phases.

The final product should be a fully functional Omni programming language with:
- Complete effect system
- Polonius-based borrow checking
- Full standard library
- Self-hosting capability
- Full tooling (LSP, formatter, package manager)
- Security and capability system

