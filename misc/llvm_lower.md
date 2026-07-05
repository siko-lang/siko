# MiniC to LLVM IR Lowering Plan

This document designs the next backend layer:

1. Existing Siko lowering keeps producing a `Siko.CBackend.MiniC.AST.CProgram`.
2. A new MiniC-to-LLVM pass lowers that `CProgram` into an in-memory LLVM IR module.
3. A separate LLVM formatter pass prints that LLVM IR module as textual LLVM IR.

The point of the split is clarity. MiniC lowering decides semantics. LLVM IR
lowering decides CFG, SSA, storage, layout, and ABI-facing representation. LLVM
formatting only serializes an already-valid IR module.

## Feasibility

This is feasible, and MiniC is a good source IR for it:

- MiniC is already low level: structs, globals, functions, simple expressions,
  switches, gotos, labels, calls, and explicit runtime helper functions.
- MiniC type information is explicit enough to build LLVM type declarations and
  to distinguish values from pointers.
- The existing backend already has the hard language-level decisions resolved:
  value type vs heap type, enum tags, generated constructors, runtime globals,
  and `main`.

The work is still non-trivial because the current C backend gets several things
for free from clang and C headers:

- C target ABI layout and calling convention.
- C typedef widths such as `size_t`, `uintptr_t`, `pthread_t`.
- C macros and header-provided names such as `NULL`, `GC_INIT`, `stderr`, and
  sometimes platform-specific libc globals.
- Struct/union layout, packed structs, aggregate returns, varargs calls, and
  target data layout strings.

The plan below keeps those details out of the MiniC lowering logic by putting
them behind `Siko.LLVM.Abi`.

## Design Boundaries

### MiniC remains the source of backend semantics

The existing pass in `Siko.CBackend.Lower` remains the producer of MiniC. The
LLVM backend should initially consume the exact same `CProgram` that
`Siko.CBackend.MiniC.Format` consumes today.

Do not fork language lowering for LLVM in the first implementation. If LLVM
needs a slightly different runtime helper, add an explicit LLVM runtime rewrite
or ABI hook in the LLVM layer rather than duplicating HIR-to-backend lowering.

### LLVM lowering and LLVM printing are separate passes

The LLVM lowering pass should produce a structured module:

```text
MiniC.CProgram
  -> Siko.LLVM.Lower.lower(program, abi)
  -> Siko.LLVM.IR.Module
```

The printer should only format:

```text
Siko.LLVM.IR.Module
  -> Siko.LLVM.Format.format(module)
  -> String
```

`Siko.LLVM.Format` must not inspect MiniC, call ABI layout queries, synthesize
control flow, or invent implicit declarations. If the formatter needs a fact,
that fact belongs in the IR module.

### ABI decisions live in `Siko.LLVM.Abi`

Platform ABI details should live behind `Siko.LLVM.Abi`.

The rule is: do whatever rustc does for the current target ABI, datatype, and
use case. Since the source is MiniC, the first ABI mode should match C ABI
behavior, roughly the `extern "C"` / `repr(C)` side of rustc's ABI handling,
not Rust's unstable `extern "Rust"` ABI.

The shape should intentionally resemble rustc's ABI vocabulary:

- `TargetDataLayout`: endianness, pointer size/alignment, aggregate alignment,
  stack alignment, integer alignments, target triple, LLVM data layout string.
- `Layout`: size, alignment, field offsets, field types, backend
  representation.
- `BackendRepr`: `Scalar`, `ScalarPair`, `Memory`, with vector cases left for
  later.
- `FnAbi`: lowered return ABI, lowered argument ABI, calling convention,
  varargs status, and attributes.
- `ArgAbi`: direct, indirect, ignore, extend, cast, byval, sret, align.
- `RuntimeAbi`: target-specific spellings for runtime/libc/GC symbols that C
  headers currently hide from us.

Reference points:

- LLVM IR modules need target datalayout and target triple metadata to match
  the backend that will consume the IR.
- rustc's `rustc_abi` crate treats ABI as layout plus calling convention
  reasoning, with target-specific calling convention implementation outside
  the common vocabulary.
- rustc separates backend representation from platform ABI. That distinction is
  useful here: a MiniC struct can be represented as memory inside a function
  while still being classified differently when passed to an ABI boundary.

## Proposed Module Layout

```text
siko/LLVM/IR.sk
siko/LLVM/Format.sk
siko/LLVM/LowerMiniC.sk
siko/LLVM/Abi.sk
siko/LLVM/Verify.sk          optional, can be added after the printer exists
```

### `Siko.LLVM.IR`

Defines the LLVM IR module AST. This is not MiniC and should not reuse MiniC
nodes.

Core types:

```text
Module {
  source_filename: Option[String],
  target_triple: String,
  data_layout: String,
  type_defs: Vec[TypeDef],
  globals: Vec[Global],
  declarations: Vec[FunctionDecl],
  functions: Vec[FunctionDef],
  attributes: Vec[AttributeGroup],
}

Type =
  Void
  Int(bits)
  Ptr
  Array(count, elem)
  StructRef(name)
  StructLiteral(fields, packed)
  Function(ret, params, variadic)

Value =
  Local(name)
  Global(name)
  ConstInt(bits, value)
  ConstNull(Type)
  ConstString(bytes)
  Undef(Type)
  Poison(Type)
```

Use opaque pointers (`ptr`) in textual IR. Keep pointee/base types in
instructions that need them, especially `load`, `store`, `alloca`, and
`getelementptr`.

Function bodies:

```text
FunctionDef {
  sig: FunctionSig,
  blocks: Vec[BasicBlock],
}

BasicBlock {
  label: String,
  instrs: Vec[Instruction],
  term: Terminator,
}
```

The AST should make malformed IR hard to build:

- Basic blocks always have exactly one terminator.
- Non-void instructions with results always carry their result name.
- Calls carry the callee function type, not only the callee pointer.
- Memory operations carry explicit value type and alignment.

### `Siko.LLVM.Format`

Formats `Siko.LLVM.IR.Module` to textual LLVM IR.

Responsibilities:

- Escape identifiers, string constants, and comments.
- Print `target datalayout` and `target triple`.
- Print identified struct types before globals/functions.
- Print globals, external declarations, function definitions, and attributes.
- Keep deterministic ordering for stable test output.

Non-responsibilities:

- No ABI decisions.
- No layout calculation.
- No MiniC-specific lowering.
- No implicit runtime declarations.
- No CFG repair.

The first printer should target LLVM's current textual IR style with opaque
pointers. Older typed-pointer IR is not worth supporting unless the toolchain
forces it.

### `Siko.LLVM.Abi`

Centralizes target ABI and runtime-platform facts.

Initial API sketch:

```text
pub struct TargetAbi {
  pub triple: String,
  pub data_layout: String,
  pub endian: Endian,
  pub pointer_bits: Int,
  pub pointer_align: Align,
  pub c_int: IntSpec,
  pub c_char_signed: Bool,
  pub size_t: IntSpec,
  pub uintptr_t: IntSpec,
  pub runtime: RuntimeAbi,
}

pub struct Layout {
  pub size: Size,
  pub align: Align,
  pub backend_repr: BackendRepr,
  pub fields: Vec[FieldLayout],
}

pub enum BackendRepr {
  Scalar(Scalar),
  ScalarPair(Scalar, Scalar),
  Memory,
}

pub enum PassMode {
  Ignore,
  Direct(Type, Vec[ParamAttr]),
  Indirect(Type, Vec[ParamAttr]), // sret/byval/byref/out-pointer style
}

pub struct ArgAbi {
  pub original_ty: MiniC.CType,
  pub layout: Layout,
  pub pass_mode: PassMode,
}

pub struct FnAbi {
  pub calling_conv: CallingConv,
  pub ret: ArgAbi,
  pub args: Vec[ArgAbi],
  pub is_variadic: Bool,
}
```

Important ABI queries:

```text
current_target(config) -> TargetAbi
layout_of(abi, ctype) -> Layout
llvm_storage_type(abi, ctype) -> IR.Type
llvm_union_storage_type(abi, fields) -> IR.Type
fn_abi(abi, sig, abi_mode) -> FnAbi
typedef_type(abi, name) -> IR.Type
runtime_symbol(abi, name) -> RuntimeSymbol
```

`AbiMode` should start with:

```text
AbiMode.C
AbiMode.SikoInternal
```

For the first LLVM backend, use `AbiMode.C` for all MiniC functions to match
the behavior of compiling the generated C with clang. Later, Siko-internal
functions can use `AbiMode.SikoInternal` if we want simpler or faster internal
IR signatures.

#### Target support strategy

Start with the targets the current bootstrap already cares about:

- macOS host target.
- Linux host target.

Do not guess new ABIs ad hoc. For each target, populate the ABI module from
rustc-observable behavior:

1. Target triple and LLVM datalayout string.
2. Primitive type sizes and alignments.
3. Struct, packed struct, array, and union layout rules.
4. Aggregate argument and return classification.
5. Varargs rules for libc calls used by the runtime.
6. Platform spellings for runtime symbols that C headers currently hide.

Use small fixtures to compare against:

```text
rustc --emit=llvm-ir --target <triple> fixture.rs
clang -S -emit-llvm --target=<triple> fixture.c
```

For MiniC, clang output is the direct compatibility oracle. rustc output is the
ABI design oracle, especially for layout vocabulary and target-specific
argument classification.

#### Runtime ABI facts

LLVM IR cannot include headers, so these must be explicit:

- `NULL` lowers to a null pointer constant.
- `GC_INIT` should lower to the real Boehm entry point for the target, usually
  a declaration/call chosen by `RuntimeAbi`, not a literal call to a macro.
- `GC_malloc` is an external function declaration.
- `abort`, `fprintf`, `pthread_join`, and other libc calls need declarations.
- `stderr` is platform-specific. On some platforms it is an external global;
  on others it may be exposed through a platform-specific symbol or accessor.
- `environ` is platform-specific enough that it should also be routed through
  `RuntimeAbi`.
- `errno` and header-only macros/inlines should be rejected until explicit ABI
  support is added.

This module is also where `CType.Typedef("pthread_t")`, `size_t`, and
`uintptr_t` are resolved.

## MiniC to LLVM Lowering

### Lowering context

`Siko.LLVM.LowerMiniC` should use a function-local builder:

```text
LowerCtx {
  abi: TargetAbi,
  module: ModuleBuilder,
  current_fn: FunctionBuilder,
  locals: Map[String, LocalSlot],
  labels: Map[String, BlockId],
  string_literals: Map[String, GlobalName],
  temp_counter: Int,
  block_counter: Int,
}
```

`LocalSlot` tracks:

```text
LocalSlot {
  c_ty: MiniC.CType,
  llvm_ty: IR.Type,
  ptr: IR.Value,
  mutable: Bool,
}
```

Use allocas for MiniC locals and parameters at first. This keeps assignment,
address-taking, field access, and gotos straightforward. LLVM's optimizer can
recover SSA with mem2reg later.

### Type lowering

Type rules:

- `Void` -> `void`.
- Fixed integers -> `i8`, `i16`, `i32`, `i64`.
- `CInt`, `Char`, and `Typedef` -> query `Siko.LLVM.Abi`.
- `Ptr(_)` -> `ptr`.
- `Array(elem, n)` -> `[n x elem]`.
- `Named(name)` -> `%name`.
- `Union(fields)` -> ABI-selected storage type with the union's size and
  alignment.

Struct type definitions:

- Non-packed MiniC struct -> identified LLVM struct with ABI-selected field
  types and explicit padding fields if needed.
- Packed MiniC struct -> packed LLVM struct where that matches C layout.
  Confirm with clang fixtures before relying on it for every target.
- Forward declarations become opaque identified types first, then bodies are
  filled once every field type is known.

Union lowering needs care. LLVM has no native union type. A naive `[N x i8]`
field has alignment 1, which is wrong when the union must align to 4, 8, or
more inside an enclosing struct. `Siko.LLVM.Abi` should provide
`union_storage_type(size, align)`, for example a struct beginning with an
aligned scalar plus byte padding, so the field has the same size and alignment
the C union would have.

### Function signature lowering

Lower every MiniC `CFnSig` through `Siko.LLVM.Abi.fn_abi`.

The ABI result decides:

- LLVM return type.
- Whether an aggregate return becomes an explicit `sret` out-pointer.
- Whether aggregate arguments are passed directly, split, cast, byval, or by
  pointer.
- Which parameter attributes to print.
- Which calling convention to print.
- Whether the function is variadic.

For the first implementation:

- It is acceptable to support only the current Siko-generated shapes on the
  first target.
- It is not acceptable to scatter ABI special cases through expression
  lowering. If a signature needs special handling, express it in `FnAbi`.

### Lvalues and rvalues

MiniC expressions have C-like places. LLVM lowering should split them:

```text
lower_place(expr) -> Address { ptr, elem_ty, align }
lower_value(expr) -> TypedValue { value, ty }
```

Places:

- `Var(name)` when the name is a local alloca or global.
- `Deref(ptr_expr)`.
- `FieldDot(base, field)` for addressable aggregate values.
- `FieldArrow(base, field)`.
- `Index(base, idx)`.

Values:

- Constants.
- Loaded place values.
- Binary/unary operations.
- Calls.
- Casts.
- Compound literals.
- Ternaries.
- Statement expressions.

When a field is selected from a non-addressable aggregate value, materialize it
to a temporary alloca first. This is simpler than trying to use
`extractvalue` everywhere and preserves C-like addressability.

### Expressions

Lowering rules:

- `IntLiteral` -> integer constant with the requested type.
- `CharLiteral` -> `i8` constant.
- `StringLiteral` -> private constant global byte array plus pointer to first
  element.
- `Var` -> load from local/global unless used as a place or function/global
  reference.
- `BinaryOp` -> integer `add/sub/mul`, signed `sdiv/srem` for signed integer
  types, unsigned forms for unsigned integer types, bitwise ops, `icmp`.
- `UnaryOp.Neg` -> `sub 0, x`.
- `UnaryOp.BitNot` -> `xor x, -1`.
- `UnaryOp.LogNot` -> `icmp eq x, 0`, with type-directed handling.
- `Call` -> direct call through resolved `FnAbi`.
- `FnPtrCall` -> indirect call using the function type carried by MiniC.
- `FieldDot` / `FieldArrow` -> GEP through layout metadata, then load.
- `Cast` -> select one of `trunc`, `zext`, `sext`, `ptrtoint`, `inttoptr`,
  `bitcast`, or no-op, based on source and destination types.
- `Ternary` -> either `select` for simple scalar/pointer values or explicit
  branch blocks plus phi for aggregate/addressable values.
- `SizeOf` -> ABI layout size constant.
- `AddrOf` -> place pointer.
- `Deref` -> load through pointer unless used as a place.
- `Index` -> GEP into an array or pointer.
- `CompoundLiteral` -> create a temporary alloca, store each field, then load
  the aggregate or return the address depending on context.
- `StmtExpr` -> lower each contained statement, preserving the value of the
  last expression statement.

`StmtExpr` may require tightening MiniC itself. Today it stores `Vec[CStmt]`,
but the value is implicit in C syntax. LLVM lowering should either:

- Define that the final `ExprStmt` is the statement-expression result, or
- Change MiniC to represent statement expressions as `{ stmts; result_expr }`.

The second option is cleaner and should be considered before implementation.

### Statements and control flow

LLVM basic blocks have explicit terminators, so lowering must not mirror C
pretty-printing directly.

Rules:

- `Decl` -> entry-block `alloca` if possible, otherwise current-block alloca
  for dynamic cases. Current MiniC declarations are static-sized, so entry
  allocas should work.
- `Assign` -> `lower_place(lhs)` then store `lower_value(rhs)`.
- `ExprStmt` -> lower for side effects; discard result.
- `Return(expr)` -> lower according to function return ABI. Store into `sret`
  pointer when applicable.
- `ReturnVoid` -> `ret void`.
- `If` -> condition block, then block, optional else block, merge block.
- `Switch` -> LLVM `switch` terminator with case blocks and merge block.
- `WhileTrue` -> loop header/body blocks. MiniC's synthetic continue and break
  labels can still lower as labels/gotos.
- `Goto` -> unconditional branch to the label block.
- `Label` -> split the current block and continue in the label block.
- `Block` -> lower statements in order. Scoping is not relevant because MiniC
  names are already unique enough for C emission.

Pre-scan each function body for labels before lowering statements. This makes
forward gotos easy and avoids backpatching by name after the fact.

After emitting a terminator, any following statement must start in a fresh
unreachable continuation block unless it is a label.

### Globals

MiniC globals become LLVM globals:

- `Static` -> `internal global`.
- `Extern` -> `external global`.
- `Plain` -> default external linkage for definitions.

Initializers must be constants. Current MiniC global initializers are simple
enough (`0`, `NULL`) but the lowering should reject non-constant initializers
instead of trying to emit invalid IR.

String literals should become private unnamed-address constants:

```text
@.str.N = private unnamed_addr constant [len x i8] c"...\00", align 1
```

### Directives and declarations

MiniC directives are C preprocessor directives. LLVM cannot emit them directly.

Handling:

- `#include` directives are ignored by the formatter and consumed only as
  hints for ABI/runtime declaration resolution.
- `#define` directives are not generally lowerable. Known defines such as
  `GC_THREADS` should become build/link configuration or runtime declaration
  choices. Unknown defines should be rejected until modeled.
- Extern functions from the Siko program should become LLVM declarations from
  their MiniC signatures.
- Runtime helpers and libc functions used by generated helpers should be
  declared explicitly.

## Pass Pipeline

Current backend pass:

```text
c:
  CBackend.lower(driver.program) -> CProgram
  MiniC.format(cprogram) -> driver.c_source
```

Proposed additions:

```text
minic:
  CBackend.lower(driver.program) -> driver.minic_program

llvm-lower:
  LLVM.LowerMiniC.lower(driver.minic_program, Abi.current_target(config))
    -> driver.llvm_module

llvm:
  LLVM.Format.format(driver.llvm_module) -> driver.llvm_source
```

Possible driver fields:

```text
pub minic_program: Option[CProgram]
pub llvm_module: Option[LLVM.Module]
pub llvm_source: String
```

Use real Siko syntax when implementing; the above is only shape.

Debug passes:

- `--pass minic`: print MiniC as C using the existing formatter.
- `--pass llvm-lower`: print a structural/debug representation of the LLVM
  module, or initially print textual LLVM IR if no debug formatter exists yet.
- `--pass llvm`: print textual LLVM IR.

The existing `--pass c` behavior should keep working until the LLVM backend is
ready to replace linking.

## Linking Strategy

Initial linking should keep using clang as the driver:

```text
clang -x ir - -o output <gc flags> <san flags>
```

Keep the existing `pkg-config --cflags --libs bdw-gc` path for Boehm GC flags.
Sanitizer flags can stay at the clang driver layer.

Later options:

- `llvm-as` + `llc` + platform linker.
- Emit object files directly.
- Add optimization with `opt` before linking.

Do not make direct object emission part of the first implementation. Textual IR
plus clang keeps verification and debugging simple.

## Implementation Phases

### Phase 1: LLVM IR AST and printer skeleton

Add `Siko.LLVM.IR` and `Siko.LLVM.Format`.

Target output:

- A hand-built module with one function can be formatted.
- `llvm-as` or `clang -x ir` accepts the result.

No MiniC lowering yet.

### Phase 2: ABI module MVP

Add `Siko.LLVM.Abi` for one host target.

Minimum:

- Target triple.
- LLVM datalayout string.
- Pointer size/alignment.
- `CInt`, `Char`, `size_t`, `uintptr_t`.
- Basic layout for integers, pointers, arrays, structs, packed structs.
- Runtime symbol table for `NULL`, `GC_malloc`, `GC_INIT`, `abort`, `environ`.

Validation:

- Compare simple struct layout and `sizeof` constants against clang-generated
  LLVM IR or compiled C probes.
- Compare rustc layout/ABI behavior for equivalent `repr(C)` fixtures where
  useful.

### Phase 3: Type, global, and declaration lowering

Lower:

- Struct definitions.
- Function declarations.
- Globals.
- String literal globals.
- Runtime declarations.

Target output:

- A MiniC program with no function bodies can become a valid LLVM module.
- The formatter emits target triple, datalayout, type defs, globals, and
  declarations.

### Phase 4: Straight-line function body lowering

Support:

- Local declarations.
- Assignment.
- Integer arithmetic.
- Loads/stores.
- Direct calls.
- Returns.
- Struct constructors for simple value and pointer-return cases.

Target tests:

- `hello_world`.
- integer arithmetic.
- simple struct construction/access.
- simple enum constructor if union storage is ready.

### Phase 5: Control flow

Support:

- `if`.
- `switch`.
- `while true`.
- `goto` and `label`.
- break/continue labels from MiniC loops.

Target tests:

- `basics/loop`.
- integer match.
- enum match.
- nested match.

### Phase 6: Full expression coverage

Support:

- Function pointer calls.
- Ternary.
- Statement expressions.
- Compound literals.
- Address-of and dereference.
- Indexing.
- Cast matrix.
- `sizeof`.

This phase should include a MiniC cleanup if `StmtExpr(Vec[CStmt])` proves too
implicit.

### Phase 7: ABI-complete calls for supported targets

Implement enough `FnAbi` to match current C backend behavior on supported
targets:

- Aggregate return direct vs `sret`.
- Aggregate argument direct vs indirect/byval.
- Varargs calls used by runtime helpers, especially `fprintf`.
- Function pointer ABI.
- Packed aggregate edge cases.

This is where most target-specific test fixtures belong.

### Phase 8: Driver and link integration

Add:

- Driver storage for MiniC and LLVM module/source.
- Passes for `minic`, `llvm-lower`, and `llvm`.
- Optional build flag to choose C backend vs LLVM backend.
- Link path that streams LLVM IR to clang.

Keep the old C link path until LLVM passes the existing test suite on at least
one target.

## Validation Plan

Use three layers of validation:

1. Formatter validity:
   - Generated `.ll` passes `llvm-as` or `clang -x ir`.
2. Behavioral parity:
   - Compile the same Siko tests through C backend and LLVM backend.
   - Compare stdout/stderr/exit status.
3. ABI parity:
   - For small fixtures, compare our generated signatures/layout against
     `clang -S -emit-llvm` for the generated C shape.
   - Use rustc `repr(C)` / `extern "C"` fixtures to check the rustc ABI model
     for the same target and datatype class.

Useful initial fixtures:

- Primitive returns and arguments.
- Small structs of one/two integers.
- Large structs.
- Packed structs.
- Arrays.
- Enum container with union payload.
- Function pointer calls.
- Varargs `fprintf`.
- `pthread_join` helper.
- GC allocation path.

## Known Risks

- ABI drift is the main risk. Keep every target-specific choice in
  `Siko.LLVM.Abi`.
- LLVM union storage is easy to get subtly wrong because alignment must match
  the C union, not merely size.
- Header macros cannot be lowered mechanically. Every macro-like runtime use
  needs an explicit ABI/runtime model.
- `stderr`, `environ`, `errno`, and thread types differ across platforms.
- Aggregate call ABI is target-specific. Do not infer it from LLVM's aggregate
  type syntax alone.
- Statement expressions currently encode their result implicitly.
- Using allocas everywhere is simple but may produce noisy IR until optimized.
  That is acceptable for the first backend.

## Success Criteria

First milestone:

- `--pass llvm` prints syntactically valid LLVM IR for a tiny Siko program.
- The IR contains correct target triple and datalayout from `Siko.LLVM.Abi`.
- The MiniC-to-LLVM lowering and LLVM formatting are separate modules.

Backend parity milestone:

- LLVM backend can build and run the existing `test/success/nostd` basics on
  one host target.

Replacement-ready milestone:

- LLVM backend passes the same test suite as the C backend on macOS and Linux.
- ABI fixtures cover structs, packed structs, unions, arrays, varargs, function
  pointers, Boehm GC calls, and pthread helper calls.
- The C backend remains available as a fallback until bootstrap refresh through
  LLVM is proven repeatable.

## References

- LLVM Language Reference Manual: module structure, datalayout, target triple,
  identified structs, opaque structs, instructions, and textual syntax:
  https://llvm.org/docs/LangRef.html
- rustc `rustc_abi` crate docs: layout/calling-convention vocabulary and the
  separation between ABI reasoning and target-specific implementation:
  https://doc.rust-lang.org/nightly/nightly-rustc/rustc_abi/index.html
- rustc `TargetDataLayout` docs: parsed LLVM datalayout plus pointer, integer,
  aggregate, and enum layout facts:
  https://doc.rust-lang.org/nightly/nightly-rustc/rustc_abi/struct.TargetDataLayout.html
- rustc `BackendRepr` docs: scalar/scalar-pair/memory representation model:
  https://doc.rust-lang.org/nightly/nightly-rustc/rustc_abi/enum.BackendRepr.html
