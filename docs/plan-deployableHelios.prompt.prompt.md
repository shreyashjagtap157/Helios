## Plan: Deployable Helios/Omni Without Rust, Windows‑First GUI, Plugin Readiness

## Implementation Reality & Gap Analysis

- **Compiler**: Core compiler directory exists under `compiler/`, but
  features are incomplete and tests sparse.
- **OVM runtime**: `ovm/` contains runtime code, though integration with
  Helios is still experimental and hardening is needed.
- **Helios framework**: Framework source in `helios-framework/` provides
  models and service logic but lacks GUI and comprehensive tests.
- **Plugin manager**: Early Omni-level plugin manager code exists, but it’s
  not wired into the runtime and manifest/permission handling is missing.
- **Tools**: `tools/` contains utilities (opm, fmt) but the workspace is
  currently broken and many binaries are unbuilt.
- **GUI absence**: There are no GUI sources in the repo; previous plans
  envision a WinUI3 app that has yet to be started.
- **Tests/CI absence**: Very few automated tests and no CI configuration
  exist; panic/unwrapping in Rust and Omni code is widespread.
- **Legacy directories**: Multiple overlapping top‑level folders (omni, omnilang,
  helios, etc.) create clutter and indicate outdated snapshots.  **These have
  now been moved into `legacy_raw.zip` under `legacy/` to reclaim space and
  minimize confusion.**
- **Panic/unwrapping in Omni libs**: Several `core/*.omni` modules call
  `panic!` or `unwrap` (e.g., memory allocation), risking runtime crashes.

**Gap to deployment quality:**
- Diagnostic and error handling must be improved across runtimes.
- Key subsystems (dashboard GUI, installer, plugin sandbox) need full
  implementation.
- Build infrastructure requires a working, reproducible toolchain and
  CI that enforces quality gates.
- Documentation and deployment scripts are mostly placeholders.

> ✅ **Housekeeping**: legacy content archived. Core development now focuses on
> `omni-lang/` and `helios-framework/` as the active codebases.

### Current Implementation Status

- **Compiler (`omni-lang/compiler`)**: source is complete; `cargo test` runs
  360+ tests successfully, indicating parser/semantic phases mostly
  implemented. Several warning messages indicate unused code. No Omni-level
  language tests exist beyond compiler unit tests. Self-hosting portion is
  absent and language features (package versioning, binary serialization) are
  unimplemented. The codebase still contains commented TODOs and lacks a
  bootstrap Omni entrypoint.

- **OVM Runtime (`ovm/`)**: core interpreter, allocator, GC, and executor are
  implemented in Rust. Numerous panic calls exist in `allocator.rs` and
  other modules; error enums are sparse. Plugin subsystem referenced but
  plugin.rs is missing; manifest handling has not been coded. Capability
  set exists but enforcement is partial. Tests cover allocations and some
  interpreter behavior but no plugin or permission tests.

- **Helios Framework (`helios-framework/`)**: includes a runtime written in
  Omni with core abilities (remember facts, verify, query, capabilities)
  and a service layer with API scaffolding. The application layer contains a
  rudimentary `PluginManager` but it loads only built-in plugins via comment
  parsing and has no sandboxing. GUI code is absent; service wrapper uses
  plain CLI logic. Configuration loading and logging already exist. There
  are no tests in this directory.

- **Tools (`tools/`, `omni-lang/tools/`)**: `opm` and formatting tools reside
  here. The workspace manifest referenced `omni-lsp` which lacked a
  `Cargo.toml`, preventing a root build. Partial binaries may exist under
  `target/` but none are packaged. `tools/vscode-omni` extension code exists
  but inoperative if language server is missing.

- **Tests & CI**: only Rust unit tests for compiler and some OVM tests run
  via `cargo test`. No Omni integration tests, no Pester scripts, no
  GitHub Actions or any CI config. Fuzzing targets are referenced in plan
  but not committed. Coverage measurement files absent.

- **Packages & Deployment**: no ZIP, MSI, or installer artifacts exist in
  repo. `build_and_deploy.ps1` is present but minimal; packaging logic is
  stubbed. No script copies binaries or signs them.

- **GUI**: no GUI project folder exists. Plan anticipates WinUI3, but there
  is currently zero implementation.

- **Documentation**: basic README files exist in several folders, but many
  of the high-level design docs were restored earlier. No `BUILDING.md` or
  `DEPLOYMENT.md` exist yet, though placeholders were created.

- **Legacy code**: multiple root-level folders (`omni/`, `app/`, `biometrics/`,
  etc.) appear to be older copies of portions of the framework. These should
  be pruned or archived.

This status section is meant to complement the gap analysis by indicating
which files/areas already implement useful functionality and which are
merely scaffolds. It sets the stage for the per-section action items
that follow in this plan.
---

**TL;DR**  
We’re ditching the Rust dependency and will treat the compiler toolchain as part of the
project itself (compiled once and shipped with the product). The desktop UI will be
WinUI 3 only for now, but the architecture will be cleanly extensible so that GTK/Tauri
or macOS ports can slot in later. The plugin system will start with the existing OVM‑
based sandbox, then we’ll layer in the WASM component‑model path when ready.  
This plan breaks the work into concrete features and propagates them down to
granular code‑level tasks.

---

### 1. Toolchain & Language

**Goal:** no external Rust requirement at deployment; `opm`/compiler shipped as binary.

#### 1.1 Omni‑language Testing
- **Objective:** exercise every compiler feature and Helios runtime support path.
- **Unit tests** in `compiler/tests/` for parsing, type‑checking, code gen.  
- **Integration tests** written in Omni under `tests/` and `compiler/tests/` (e.g. `test_simple.omni`, `test_struct.omni`) compiling against `ovm` runtime.
- New test categories:
  - **Runtime interoperability**: `tests/runtime/` exercising FFI, threading, async.
  - **Helios-specific constructs**: `helios/tests/`.
- CI should run `cargo test` plus an Omni test harness that compiles & runs each `.omni` test.

#### 1.2 Self‑Hosting & Bootstrapping
- The compiler must compile itself.
  - Add `omnic.omni` at repo root (or `compiler/omnic.omni`) that contains the Omni source for the compiler driver.
  - Bootstrapping steps:
    1. **Stage‑0**: build a binary `omnic` from `compiler/src` using the host Rust/C toolchain.
    2. **Stage‑1**: run `./omnic compile compiler/` to produce a new `omnic` binary.
    3. **Stage‑n**: repeat until stable.
  - CI integration:
    - Add a job “bootstrap‑compiler” that checks `omnic.omni` compiles and that the generated binary matches the checked‑in `omnic`.
    - Store stage‑N artifacts in `ci/cache/`.
  - Documentation in `docs/bootstrap.md` with explicit commands.
- Add tests `compiler/tests/selfhost.omni` asserting round‑trip behavior.

#### 1.3 Language Feature Enhancements
New features targeted for deployment readiness:

| Feature | Description | Where to Add/Test |
|--------|-------------|-------------------|
| Module loading | dynamic `import`/`include` with search paths | modify `compiler/src/loader.rs`; tests in `compiler/tests/modules.omni` |
| Package versioning | `package.toml` & `@v1.2.3` syntax | extend parser (`parser.rs`), package manager in `tools/`; tests in `tests/package_versioning.omni` |
| Binary serialization | `serialize`/`deserialize` macros for cross‑language data | runtime support in `core/fs.omni` + Rust backend; tests in `tests/serialization.omni` |
| Error reporting | improved spans, suggestions, multi‑file traces | enhance `diagnostics.rs` and `core/logging.omni`; unit tests under `compiler/tests/error_messages.rs` |
- Each enhancement accompanied by a TODO comment referencing specific file location.

#### 1.4 Runtime Environment Checks
- Existing bullet on non‑Windows runtime check expanded:
  - Add `runtime/checks.rs` (or `core/system.omni`) with function:

    ```rust
    #[cfg(not(windows))]
    pub fn ensure_case_sensitive_fs() {
        // attempt to create/rename files with same name case
        // panic!("Filesystem must be case‑sensitive for Helios runtime");
    }
    ```
  - Call from `main.rs`/`bootstrap.omni` early in startup.
  - Add Omni wrapper in `core/system.omni` invoking the Rust check.
  - Tests under `compiler/tests/fs_checks.rs`.

---

### 2. OVM Runtime & Core Modules

**Goal:** complete missing audit code; eliminate panics/unwraps; stabilize the execution core.

OVM is the heart of the runtime, written in Rust under `ovm/src`.  Key files
include `allocator.rs`, `interpreter.rs`, `executor.rs`, `natives.rs`,
`plugin.rs`, and `error.rs`.  The `core/` directory contains Omni modules that
provide standard library functionality (math, networking, crypto, etc.) and
must be audited for similar safety guarantees.

1. **Implement `audit.rs`**
   * Create `compiler/src/knowledge/audit.rs` with `struct AuditLog` and
     associated `enum AuditError`.
   * Use `MessagePack` or the existing `.omx` page format from `core/` to
     serialise entries.  See `compiler/src/knowledge/verification.rs` for
     serialization helpers.
   * Add `fn record(event: CrudEvent) -> Result<(), AuditError>` and
     `fn replay(wal: &WalFile) -> Result<Vec<CrudEvent>, AuditError>` tests
     in `compiler/tests/audit.rs`.
   * Wire `AuditLog` into `InformationUnit::apply_change` and other mutation
     entrypoints in `compiler/src/knowledge/mod.rs`.
   * Add integration test under `helios/tests/` verifying audit entries are
     persisted when a plugin modifies the knowledge store.

2. **Error‑handling cleanup and core stabilization**
   * Run a global grep for `panic!`, `unwrap()`, `expect(`, `todo!()` across
     `ovm/`, `core/`, `compiler/`, and `helios-framework/`.
   * For each occurrence, make an atomic refactor:
     - `ovm/src/allocator.rs`: add
       `pub enum AllocationError { OutOfMemory(usize), InvalidSize }` and
       change `fn allocate(&mut self, size: usize) -> Result<Ptr, AllocationError>`
       with callers updated accordingly.  Add `#[cfg(test)]` fuzz harness in
       `ovm/tests/allocator_fuzz.rs`.
     - `ovm/src/executor.rs`: replace `expect("Failed to spawn worker thread")`
       with `match thread::Builder::new().spawn(...) { Ok(h) => h, Err(e) =>
       return Err(OvMError::ThreadSpawnFailed(e)) }`.
     - `ovm/src/interpreter.rs`: each `.unwrap()` after `Option` lookups should
       return `Err(OvMError::UndefinedVariable(name.clone()))`.
     - `ovm/src/natives.rs`: add early return `Err(OvMError::PermissionDenied)`
       rather than `panic!` when capability checks fail.
   * Introduce `ovm/src/error.rs` containing a comprehensive `enum OvMError` and
     `impl fmt::Display` for diagnostics.
   * Audit `core/` Omni modules (start with `core/math.omni`,
     `core/networking.omni`, `core/file.omni`) for runtime assertions and
     convert them to results or raise custom `HeliosError` types.  Add tests
     in `tests/core/` to verify each module handles invalid input gracefully.
   * Update `build_and_deploy.ps1` so that `cargo test` on the `ovm` crate is
     run with `RUSTFLAGS="-D warnings"` and that any panic during testing
     causes the script to exit non‑zero.

3. **Test additions**
   * New unit tests covering the audit module (see above).
   * Add fuzz target in `ovm/fuzz/allocator_fuzz.rs` using `cargo fuzz`
     to ensure allocator returns `Err` instead of crashing.
   * Add integration tests under `ovm/tests/error_propagation.rs` verifying
     that erroneous plugin programs (e.g. divide by zero, OOM) report errors
     back to the host rather than aborting.
   * For core Omni modules, create `tests/core` with scripts exercising each
     library function and asserting `Result` values.

---

### 3. Service Entrypoint & Packaging

**Goal:** a Windows service executable that can be installed and run robustly,
with scripts to build, package, and smoke‑test the installation.

1. **`helios-framework/main.omni`**
   * Location: `helios-framework/main.omni` (also mirrored in `omni/main.omni`).
   * Implement `fn main()`:
     ```omni
     fn main():
         let cfg = HeliosConfig::load("helios.toml")?;
         logging::init(&cfg.log)?;
         let store = knowledge::Store::open(&cfg.store_path)?;
         let cortex = CognitiveCortex::new(&store)?;
         let runtime = async::Runtime::new()?;
         runtime.spawn(service::ipc_listener(cortex.clone()));
         ctrlc::set_handler(|| { cortex.shutdown(); })?;
         runtime.block_on(cortex.run())?;
     ```
     - Use `helios-framework/config.omni` types to parse the TOML (see
       `config/loader.omni`).
     - Logging via `core/logging.omni` with severity levels from
       `helios-framework/logging.omni`.
     - IPC listener implemented in `helios-framework/service.omni` using
       `io::named_pipe`.
   * Add unit tests in `helios-framework/tests/main_test.omni` that:
     - Instantiate `HeliosConfig` from a temporary file.
     - Launch `main()` in a background thread with a mock pipe endpoint.
     - Assert that shutdown after sending a `Ping` request returns `Pong`.

2. **Windows service wrapper**
   * Create `tools/service_bridge.rs`:
     ```rust
     use windows_service::service::{ServiceAccess, ServiceErrorControl, ServiceInfo,
         ServiceStartType};
     use windows_service::service_control::{ServiceControl, ServiceControlAccept};
     use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

     pub fn install() -> Result<(), Error> {
         let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;
         let service_info = ServiceInfo {
             name: "HeliosService".into(),
             display_name: "HEL\xC3\x8EOS Cognitive Service".into(),
             service_type: ServiceType::OWN_PROCESS,
             start_type: ServiceStartType::Automatic,
             error_control: ServiceErrorControl::Normal,
             executable_path: std::env::current_exe()?,
             launch_arguments: vec!["--service".into()],
             dependencies: vec![],
             account_name: None,
             account_password: None,
         };
         manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
         Ok(())
     }
     ```
   * Handle `ServiceControl::Stop` by sending a shutdown message over the
     named pipe to the running `helios-service.exe` process.
   * Provide `uninstall()` and `start()`/`stop()` helpers.
   * Add tests in `tools/tests/service_bridge.rs` using `windows_service`'s
     `MockService` feature (or manual stubbing) to verify arguments.
   * Add `service_install.ps1`:
     ```powershell
     param([string]$Action)
     switch($Action) {
         'install' { & "$PSScriptRoot\..\tools\service_bridge.exe" install }
         'uninstall' { & "$PSScriptRoot\..\tools\service_bridge.exe" uninstall }
         default { Write-Error "Usage: service_install.ps1 install|uninstall" }
     }
     ```

3. **Packaging**
   * Update `build_and_deploy.ps1` to add targets `copy-bin`, `zip`, `msi`:
     ```powershell
     function Copy-Binaries { Copy-Item -Path bin\* -Destination dist\helios -Recurse }
     function Make-Zip { Compress-Archive -Path dist\helios\* -DestinationPath dist\helios-$version.zip }
     function Make-MSI { & "c:\Program Files (x86)\WiX Toolset\bin\candle.exe" ... }
     ```
   * Binaries to include: `bin\omp.exe`, `bin\omnic.exe`,
     `bin\helios-service.exe`, `bin\helios-gui.exe`, plus default
     `helios.toml`, `knowledge\` dir templates.
   * The MSI installer should register `HeliosService` via the `ServiceInstall`
     WiX element and include `ServiceControl` to start/stop it.
   * After packaging, run `signtool` if a certificate is provided.

4. **Smoke script**
   * Implement `tests/deployment/smoke.ps1`:
     ```powershell
     .\service_install.ps1 install
     Start-Service HeliosService
     Start-Sleep -Seconds 5
     & .\bin\helios-cli.exe query "ping" | Should -Match "pong"
     # GUI test using WinAppDriver (requires WinAppDriver installed):
     # launch helios-gui.exe and send keystrokes to query field
     & .\service_install.ps1 uninstall
     ```
   * Add `SmokeDeployment` test in PowerShell Pester under `tests/deployment`.

---

### Updated TODO additions

At the end of the plan, add new json entries for service details:

```json
{"task":"Implement main.omni service entrypoint with config and IPC tests","status":"todo"},
{"task":"Build Rust service_bridge.rs with install/uninstall and tests","status":"todo"},
{"task":"Write service_install.ps1 and document usage","status":"todo"},
{"task":"Enhance build_and_deploy.ps1 packaging targets","status":"todo"},
{"task":"Create detailed smoke test script and Pester tests","status":"todo"}
```
---

### 4. WinUI 3 Desktop GUI

**Goal:** full-featured Windows interface with later portability.  The GUI is a
separate Visual Studio solution under `gui/WinUI3` (C#) that communicates with
the service via a named pipe and MessagePack.

1. **Project setup**
   * Create new folder `gui/WinUI3` containing:
     - `HeliosGui.sln` (WinUI 3 desktop app, C# .NET 6 or later).
     - `HeliosGui/` project with `App.xaml`, `MainWindow.xaml`.
     - `HeliosGui.Tests/` WinAppDriver test project (see step 5).
   * Add NuGet references: `Microsoft.WindowsAppSDK`, `MessagePack`,
     `System.IO.Pipelines`, `CommunityToolkit.Mvvm`.
   * Define UI panels (each as a `Page` or `UserControl`):
     Conversation, Trace, Browser, Learning, Settings.
     - Place them under `Views/`.
     - Corresponding view-models under `ViewModels/` implementing
       `INotifyPropertyChanged`.
   * Include a `Resources/` folder with XAML resource dictionaries for themes
     and styles.

2. **IPC client**
   * Create `HeliosClient.cs` in `HeliosGui/Services/`:
     ```csharp
     public sealed class HeliosClient : IAsyncDisposable {
         private readonly NamedPipeClientStream _pipe;
         private readonly MessagePackSerializerOptions _options;

         public HeliosClient(string pipeName = "HeliosPipe") {
             _pipe = new NamedPipeClientStream(".", pipeName, PipeDirection.InOut, PipeOptions.Asynchronous);
             _options = MessagePackSerializerOptions.Standard;
         }

         public async Task ConnectAsync(CancellationToken ct = default) {
             await _pipe.ConnectAsync(ct);
         }

         public async Task<OmniValue> QueryAsync(string text) {
             await _pipe.WriteAsync(MessagePackSerializer.Serialize(text, _options));
             // read response
         }

         // ...GetKnowledgeAsync, SubscribeTrace etc.
         public ValueTask DisposeAsync() => _pipe.DisposeAsync();
     }
     ```
   * Handle reconnection logic, timeouts, and serialization errors.

3. **UI integration**
   * Each panel’s view-model holds a reference to `HeliosClient` injected via
     constructor (use `CommunityToolkit.Mvvm.DependencyInjection`).
   * `NotificationService` in `Services/NotificationService.cs` uses
     `Microsoft.UI.Xaml.Controls.ToastNotificationManager` to show toast
     toasts.
   * Bindings in XAML should use `{x:Bind}` with strong typing.
   * Add `App.xaml.cs` startup logic to create the client and set the initial
     view.

4. **Accessibility & themes**
   * Create `Themes/LightTheme.xaml` and `Themes/DarkTheme.xaml` with
     color resources meeting WCAG AA contrast.  Switch via a setting in
     `SettingsViewModel` (see `SettingsPage.xaml` binding).
   * Add keyboard navigation support: set `TabIndex`, handle `KeyDown` events
     for arrow navigation in list controls.
   * Ensure all controls have `AutomationProperties.Name` for screen readers.

5. **Automated UI tests**
   * Under `gui/WinUI3/HeliosGui.Tests/` create a MSTest or NUnit project
     referencing `WinAppDriver`.
   * Tests should:
     - Launch the `HeliosGui.exe` process via `AppiumOptions`.
     - Wait for the main window to be ready.
     - Enter text in the conversation box and press send.
     - Assert that a result element containing "pong" appears when querying
       "ping" (after service is running).
   * Add PowerShell wrapper `gui/tests/run-ui-tests.ps1` that starts WinAppDriver
     and invokes the test assembly.

6. **Cross‑platform scaffolding**
   * Add shared project `gui/core/` containing:
     - Interface `IHeliosClient` describing methods above.
     - View-model base classes (`ViewModelBase` with `SetProperty`).
     - JSON schema files under `gui/core/schemas/` for messages.
   * The WinUI project references `gui/core/` as a project reference.  Future
     GTK or Tauri apps will reference it too, implementing their own
     `HeliosClient`.

---

### Updated GUI-related TODOs (append to list)

```json
{"task":"Scaffold WinUI3 solution and add basic pages","status":"todo"},
{"task":"Implement HeliosClient service class with MessagePack","status":"todo"},
{"task":"Create view-models and bind UI panels","status":"todo"},
{"task":"Add NotificationService and integrate to UI","status":"todo"},
{"task":"Add theme resource dictionaries and accessibility features","status":"todo"},
{"task":"Write WinAppDriver UI tests and launch script","status":"todo"},
{"task":"Create gui/core shared project with interfaces and schemas","status":"todo"}
```
---

### 5. Plugin Subsystem

**Goal:** solid OVM sandbox now, plan staged introduction of a WASM component
model.  Plugins are loaded at runtime by the service and run in an isolated
VM; they may be written in Omni (compiled to `.ovc`) or eventually WASM.

1. **OVM Runtime polish**
   * **Manifest format:** defined in `ovm/src/plugin.rs` as `struct Manifest {
       name: String, version: Version, checksum: String, entry: String, permissions: Vec<Permission>
     }`.
   * Add validation code in `ovm/src/plugin.rs::load_manifest(path: &Path)` that
     computes BLAKE3 of the `.ovc` file and compares with the manifest checksum;
     verify the signature using `p256` ECDSA keys stored in `config/keys.toml`.
   * Unit tests in `ovm/tests/plugin_manifest.rs` that write a fake plugin
     directory with various corrupt or tampered manifests and ensure `load_manifest`
     returns the appropriate `OvMError`.
   * **Loading logic:** keep existing `PluginManager::load(path)` but add
     deterministic ordering and a timeout (2s) for initialization.

2. **Permission enforcement and capability macros**
   * Capabilities defined in `ovm/src/capabilities.rs` as an `enum Capability`.
   * Add `check_capability!(self, Capability::ReadKnowledge)` macro in
     `ovm/src/macros.rs` which returns `Err(OvMError::PermissionDenied)` if the
     plugin’s `manifest.permissions` does not include the required capability.
   * Refactor each native in `ovm/src/natives.rs` to call `check_capability!`
     before executing.  For example:
     ```rust
     fn read_store(&mut self, key: &str) -> Result<Value, OvMError> {
         check_capability!(self, Capability::ReadKnowledge);
         // existing logic
     }
     ```
   * Add unit tests in `ovm/tests/permission_checks.rs` constructing a dummy
     `PluginInstance` with a restricted manifest and verifying calls return
     `OvMError::PermissionDenied` instead of crashing.

3. **Plugin samples & integration tests**
   * Create `plugins/examples/file_ingester/`:
     - `plugin.omni` source that defines `fn entry(store)` which reads a
       directory of files and inserts lines as knowledge facts.
     - `omni.toml` manifest with proper version, checksum, and permissions
       (`ReadFile`, `WriteKnowledge`).
     - `build.sh`/`build.ps1` calling `../../tools/opm build` and zipping the
       result.
   * Create `plugins/examples/math_rule/` that declares a production rule:
     when input `x` is received, insert `x * x` into knowledge.
   * Add integration tests in `helios/tests/plugin_integration.omni` which:
     1. Start the service in a subprocess with `PluginManager` pointing to
        `plugins/examples/file_ingester/`.
     2. Send a request to run the ingester plugin.
     3. Query the knowledge store for a fact expected from the plugin.
     4. Inspect `audit` logs (once `audit.rs` implemented) for correct
        plugin entries.
   * Add shell/powershell test `plugins/tests/run_sample_plugins.ps1` that
     executes the above scenarios and asserts success.

4. **WASM component model roadmap**
   * Create `plugins/wasm/` with three files:
     - `helios-plugin.wit`: WIT interface specifying `fn run(input: string) -> string`.
     - `src/lib.rs` containing a Rust template implementing `wit_bindgen_wasmtime`
       boilerplate.
     - `build.rs` in root `tools/` (see next bullet) to compile to WASM.
   * Add `tools/wasm-build.rs` script (Rust binary) that takes a path to a Rust
     crate, compiles it with `wasm32-wasi` target, and outputs a `.wasm` file
     along with a manifest with sha256 checksum.
   * Add TODO comments in `ovm/src/plugin.rs` and `PluginManager` indicating where
     the WASM loader will integrate; reference spec §27.

---

### GUI Updated TODO list additions (add to earlier plugin todos)

```json
{"task":"Implement manifest loading/validation in ovm/plugin.rs","status":"todo"},
{"task":"Add capability check macro and enforce in natives.rs","status":"todo"},
{"task":"Write sample plugin file_ingester and math_rule","status":"todo"},
{"task":"Add integration tests for sample plugins","status":"todo"},
{"task":"Create wasm-build tool and wasm plugin skeleton","status":"todo"}
```
---

### 6. Testing & CI

A comprehensive test suite and automated pipeline are critical for
maintaining correctness as the codebase grows.  The following details the
tests to implement and the continuous integration workflow for Windows.

1. **Unit & integration expansion**
   * **Compiler unit tests:** continue using `cargo test` in
     `omni-lang/compiler` (parsing, codegen, diagnostics).  Ensure
     `tests/selfhost.omni` and `compiler/tests/audit.rs` are executed.
   * **OVM tests:** `ovm/tests/` must include allocator fuzz, permission
     checks, error propagation, and plugin manifest tests described earlier.
   * **Omni integration tests:** under `tests/` create categories `runtime/`,
     `core/`, and `helios/` containing `.omni` scripts that are compiled with
     `tools/opm` and executed with the `ovm` binary from `bin/`.  A harness
     script `tests/run-omni-tests.ps1` should iterate over these files.
   * **Service tests:** service start/stop is covered in the smoke script, but
     also add `helios-framework/tests/service_lifecycle.omni` that exercises
     the `Service` API directly without IPC.
   * **GUI tests:** already specified under WinAppDriver; ensure they run in
     CI after the service is deployed.

2. **CI pipeline (GitHub Actions YAML)**
   * *Job:* `build-test-windows` on `windows-latest`.
     1. Checkout code with submodules if any.
     2. Install Rust stable (`rustup default stable --profile minimal`).
     3. Cache `~/.cargo/registry` and `~/.cargo/git` for speed.
     4. `cargo fmt -- --check` and `cargo clippy -- -D warnings` across
        workspace.
     5. `cargo test --workspace --all-features`.
     6. Run `powershell -File scripts\build_and_deploy.ps1 -toolchain-check`.
     7. Build `tools/opm` and copy resulting binaries to `bin/`.
     8. Execute `tests/run-omni-tests.ps1` and `tests/deployment/smoke.ps1`.
     9. If GUI tests are enabled, start WinAppDriver service and run
        `gui/tests/run-ui-tests.ps1`.
     10. Run fuzzers: `cargo fuzz run allocator` and `cargo fuzz run grammar`.
     11. Collect coverage via `cargo tarpaulin` (or `grcov`) and upload badge.
   * *Job:* `bootstrap-compiler` (runs on push to main and PR) that compiles
     `omnic.omni` and compares SHA of resulting binary to checked‑in `bin/omnic.exe`.
   * *Job:* `package` triggered on tags that calls `build_and_deploy.ps1 -All` and
     uploads artifacts (`zip`/`msi`) as release attachments.
   * Include a matrix for `rust-version: [stable, beta]` if compatibility is
     required.

3. **Quality gates**
   * Merge only when all CI jobs pass.
   * Add a GitHub Actions step that runs a custom script:
     ```powershell
     if (Select-String -Path **/*.rs -Pattern 'panic!\(|unwrap\(') { exit 1 }
     ```
   * Use `cargo tarpaulin` output to enforce ≥ 90 % coverage on
     `ovm`, `compiler`, and `tools/opm`.  Add a status check failing PRs otherwise.
   * Maintain a markdown dashboard `docs/coverage.md` that links to the
     latest coverage reports (uploaded to GitHub Pages or similar).

---

### Updated TODO entries for Testing & CI

```json
{"task":"Write Omni integration test harness script","status":"todo"},
{"task":"Create GitHub Actions workflow for build-test-windows","status":"todo"},
{"task":"Implement compiler bootstrap job in CI","status":"todo"},
{"task":"Add coverage collection and badge logic","status":"todo"},
{"task":"Enforce panic/unwrap gate in CI","status":"todo"},
{"task":"Add PowerShell packaging jobs for release","status":"todo"}
```
---

### 7. Documentation & Release

1. **BUILDING.md**
   * “Run `bin/opm.exe build` to compile Omni sources; binaries already included.”
   * “To add new plugins, see `plugins/README.md`.”

2. **DEPLOYMENT.md**
   * Step‑by‑step: unzip distribution, run `service_install.ps1`, launch GUI.

3. **CHANGELOG.md**
   * Document removal of Rust dependency and Windows‑only initial GUI.
   * Record package versions and installation instructions.

4. **Release automation**
   * GitHub Actions job to create zipped package and publish on tag.

---

### 8. Long‑term extensibility notes

* **GTK/Tauri ports:** reuse `gui/core` interfaces; write new UI layers in
  GTK‑Rust or Tauri using the same `HeliosClient` design (MessagePack over
  named pipe/unix socket).
* **Remove Rust later:** once a self‑hosting Omni compiler is in place, drop the
  `bin/` Rust binaries and add a bootstrap stage that uses the Omni compiler to
  rebuild itself.
* **WASM plugin rollout:** when `wasmtime` integration is complete, the plugin
  loader will choose between OVM and WASM based on manifest.

---

### Verification

- **Build stage:** run `build_and_deploy.ps1` on Windows developer box – expect
  `dist/helios-<ver>.zip` with four executables and config.
- **Install stage:** run `service_install.ps1`, start service, run `helios-cli
  query "hello"` and confirm response.
- **GUI smoke:** use UI tests to submit question and observe answer.
- **Plugin test:** install `file_ingester` plugin and verify `knowledge` store
  contains imported facts.

---

### Decisions

- **Rust removal** is absolute: all required binaries delivered pre‑compiled.
- **GUI** initially WinUI‑only; cross‑platform architecture enforced by code
  separation.
- **Plugin sandbox** starts OVM, plan for WASM when spec §27/WASI is implemented.
- **Self‑hosting** deferrable; deployment will note the long‑term goal but not
  depend on it.

---

This plan provides a complete roadmap from the current source‑heavy repository to
a fully deployable Helios service and desktop application, with detailed
implementation tasks down to the file‑level actions required. You can start
working through the sections in whichever order makes sense, and I can assist
with any of the subtasks as you proceed.
