# Basic Block Lowering Final Investigation Plan

This document is the pre-implementation investigation plan for
`siko/LowerBasicBlock.sk`. It intentionally does not implement the pass. The
goal is to make the implementation straightforward by spelling out the current
state, the target invariants, the missing pieces, the order to investigate them,
and the checks that should fail loudly if an assumption is wrong.

The intended pass still operates on the AST-shaped HIR. It does not introduce a
new IR data type. The output shape is a function body that is one flat
`ExprKind.Block(BlockInfo(Vec[Statement]))`, where `Label` statements split the
statement list into implicit basic blocks and terminators explicitly connect
those blocks.

## Current State

The following pieces are already in place:

- `ExprKind.EnumTag(Expr)` exists.
- `ExprKind.Switch(Expr, Vec[Expr])` exists.
- `ExprKind.ReadVariant(Expr, Expr)` exists.
- `ExprKind.Label(String)` and `ExprKind.Goto(String)` already existed.
- Formatting exists for the new primitives:
  - `enum_tag!(...)`
  - `switch ... [...]`
  - `read_variant!(..., ...)`
- Earlier passes panic on the new primitives because no existing pre-basic-block
  pass should produce them.
- `LowerMatch` now lowers string, int, char/byte, tuple, and struct matching
  before the late backend path.
- After `LowerMatch`, `ExprKind.Match` is expected to mean enum match only.
- `LowerMatch` generates direct calls to:
  - `B.get_string_eq_qn()` for string switches,
  - `B.get_int_eq_qn()` for int switches,
  - `B.get_u8_eq_qn()` for char/byte switches.
- The C backend now rejects non-enum matches at `lower_match`.

The following pieces are still missing and must be investigated before coding
the pass:

- Real variant payload structs do not exist in `Program.structs` yet.
- The C backend still synthesizes enum payload structs internally.
- The C backend still lowers enum `Match` directly.
- The C backend currently panics on `EnumTag`, `Switch`, and `ReadVariant`.
- There is no `Siko.LowerBasicBlock` module yet.
- There is no late validator that proves the basic-block output is flat and
  well-terminated.

## Target Invariants

After `LowerBasicBlock`, every lowered function with a body should satisfy these
shape rules:

- The body is a single `ExprKind.Block`.
- The block contains a flat `Vec[Statement]`.
- There are no nested `ExprKind.Block` nodes in statement position.
- There are no `Statement.Loop` nodes.
- There are no `ExprKind.Break` or `ExprKind.Continue` nodes.
- There are no `ExprKind.Match` nodes.
- There are no `Statement.TrailingExpr` nodes.
- Every basic block starts with a `Statement.Expr(ExprKind.Label(...))`.
- Every basic block ends with one of:
  - `Return`,
  - `Goto`,
  - `If` whose branches are both `Goto`,
  - `Switch` whose arms are all `Goto`.
- Fallthrough between labels is made explicit with `Goto`.
- All `Goto` targets exist in the same function.
- Labels are unique inside each function.
- Generated labels never collide with existing coroutine labels.
- `Switch` is used only for dense enum tag dispatch.
- `ReadVariant` is used only in a block that statically corresponds to the
  variant payload type being read.

The C backend boundary should eventually become:

- `ExprKind.Match` is an error.
- `Statement.Loop` is an error.
- `ExprKind.Break` and `ExprKind.Continue` are errors.
- The backend lowers:
  - normal expressions,
  - `Label`,
  - `Goto`,
  - `EnumTag`,
  - `Switch`,
  - `ReadVariant`.

## Pipeline Position

The requested position is immediately before the C backend.

The current lowering pipeline ends like this:

1. `lower-coroutines`
2. `lower-tuples`
3. `lower-builtins`
4. `value-types`
5. `c`

The intended insertion point is:

1. `lower-coroutines`
2. `lower-tuples`
3. `lower-builtins`
4. `value-types`
5. `lower-basic-block`
6. `c`

This means `LowerBasicBlock` runs after monomorphization, tuple lowering,
builtin lowering, coroutine lowering, and value-type calculation.

Important consequence: if `LowerBasicBlock` creates real variant payload
structs after `value-types`, it must set their `value_type` field itself. There
will be no later `value-types` pass to compute it. The simplest rule to
investigate is:

- payload struct `value_type = enum.value_type`.

That matches the current C backend behavior:

- value-type enum payloads are stored inline,
- non-value-type enum payloads are stored by pointer.

An alternate pipeline is possible:

1. generate variant payload structs before `value-types`,
2. run `value-types`,
3. lower basic blocks,
4. emit C.

Do not choose this alternate path unless investigation shows that setting
`value_type` directly is insufficient. The user's requested mental model is
"right before cbackend", so prefer keeping `lower-basic-block` after
`value-types`.

## Input Shape Investigation

Before implementing the pass, verify the actual program shape at the insertion
point.

Use the newly built compiler binary, not `./base.bin`, when inspecting the
effects of new code. `./base.bin` is the bootstrap compiler and can show stale
behavior.

Checklist:

- Dump after `lower-match` for representative cases and confirm:
  - int matches are `if` chains,
  - char matches are `if` chains,
  - string matches are `if` chains,
  - struct matches are field-access lets plus child control flow,
  - remaining `match` nodes are enum matches.
- Dump after `lower-builtins` and confirm:
  - builtin int/u8 equality calls have become `BinaryOp.Eq`,
  - no `FunctionKind.Builtin` call reaches C lowering.
- Dump after `value-types` and confirm:
  - enum `value_type` flags are final,
  - existing generated tuple structs are in `Program.structs`,
  - enum definitions are monomorphic.
- Inspect coroutine-heavy tests after `lower-coroutines` and confirm:
  - existing `Label`/`Goto` expressions may already be present,
  - generated labels use names like `co_<...>_seg_<...>`,
  - the basic-block label allocator must avoid collisions.
- Confirm normalized function bodies after late lowering:
  - top-level body is `Block`,
  - most functions end in explicit `Return`,
  - `TrailingExpr` should not be present by the time the new pass runs,
  - control-flow expressions are statement-position expressions.

If any of those assumptions fail, document the exact counterexample before
writing the pass.

## Variant Payload Struct Investigation

The biggest remaining design point is turning variant payload structs into real
`Program.structs`.

### Required Properties

For every monomorphic enum variant, create one real `StructDef`:

- It has a stable compiler-owned qualified name.
- It is inserted into `program.structs`.
- Its fields are named:
  - `item0`,
  - `item1`,
  - and so on.
- Its fields use the variant item types after all prior lowerings.
- It has no methods.
- It has no default fields.
- It is not public.
- Its `value_type` mirrors the owning enum's `value_type`.
- It is reused consistently by:
  - enum container layout,
  - variant constructor lowering,
  - `ReadVariant`,
  - match binding field accesses.

Variants with no payload still need an explicit decision:

- Preferred invariant: generate an empty payload struct for every variant,
  including payload-less variants.
- Simpler lowering invariant: only emit `ReadVariant` when a variant arm has
  payload bindings.

The second option is easier and avoids useless lets. The first option makes enum
layout uniform. Decide before implementing C backend layout changes.

### Qualified Name Scheme

Investigate two options.

Option A: add a new `QualifiedName.BaseName` case for variant payload structs.

Possible shape:

```text
BaseName.VariantPayload(enum_name: I.String, variant_name: I.String)
```

or, if post-mono identity must include full monomorphic names:

```text
BaseName.VariantPayload(payload_owner: I.String)
```

Pros:

- No fake module names.
- No collision with user structs.
- Easy to recognize payload structs in later passes.
- Good long-term IR hygiene.

Cons:

- Requires updating `QualifiedName.name`, `short_name` if needed, formatting,
  and any exhaustive matches.

Option B: encode payload structs as ordinary structs in a compiler-reserved
module/name namespace.

Possible shape:

```text
QualifiedName.new_struct(
    I.intern("$siko.variant_payload"),
    I.intern("<enum-hash>.<variant-name>")
)
```

Pros:

- Smaller immediate change.
- Reuses `StructDef` machinery with no `QualifiedName` enum extension.

Cons:

- Need to guarantee the reserved namespace cannot collide with user modules.
- Generated names must remain C-label/mangle safe enough after hashing.
- Harder to distinguish compiler payload structs later.

Preferred investigation outcome: choose Option A if the amount of plumbing is
small. Choose Option B only if adding a `QualifiedName` case creates excessive
churn.

### Payload Struct Map

`LowerBasicBlock` and the C backend need to map between:

- enum qname,
- variant qname,
- variant index,
- payload struct qname,
- payload struct type.

Investigate where this map should live.

Candidate local structure:

```text
Map[QualifiedName, PayloadInfo] // keyed by variant qname
```

where `PayloadInfo` stores:

- `enum_qname`,
- `variant_qname`,
- `variant_index`,
- `payload_qname`,
- `payload_ty`,
- `field_types`.

If the C backend needs the same mapping, either:

- recompute it in C backend from `program.enums` using the same helper module,
- or store enough identity in the payload struct qualified name to recover it.

Do not duplicate stringly naming logic in unrelated modules. If a helper is
needed, create a small shared module for variant payload metadata/naming.

## C Backend Layout Changes To Plan

The current C backend creates payload structs in `lower_enum` using
`variant_struct_name(edef, variant)` and fields named by `variant_field_name`.
That is exactly what the user does not want long-term: payload structs should be
real program structs, not backend-only artifacts.

The C backend changes should be staged and investigated before implementing
`LowerBasicBlock`.

### Data Definitions

Current behavior:

- `lower_structs` emits all `program.structs`.
- `lower_enum` emits enum container structs.
- `lower_enum` also emits backend-synthesized per-variant payload structs.

Target behavior:

- `lower_structs` emits real payload structs because they are in
  `program.structs`.
- `lower_enum` emits only enum container structs.
- enum container payload union fields refer to the real payload struct types.
- `lower_enum` no longer synthesizes payload structs.

This requires `lower_enum` to find each variant's payload struct qname.

### Constructors

Current behavior:

- `lower_variant_ctor` builds backend-synthesized payload structs directly.
- Payload fields use backend names like `field_0`.

Target behavior:

- `lower_variant_ctor` uses the real payload struct type.
- Payload fields use normal struct field names produced from `item0`, `item1`,
  etc.
- The enum container stores either:
  - the payload struct value for value-type enums,
  - the payload struct pointer for non-value-type enums.

Possible implementation strategies:

- Build the payload struct directly in C as today, but use real payload struct
  qname and real field names.
- Or call the generated struct constructor for the payload struct.

Prefer direct construction in C backend if it avoids adding temporary
high-level AST calls. The payload struct exists in `Program.structs`; the
backend already has enough information to assign its fields.

### `EnumTag`

`EnumTag(value)` should lower to the tag field read.

For value-type enum:

```text
value.tag
```

For heap enum:

```text
value->tag
```

The result type should be a builtin integer type chosen by the pass. Prefer
`Std.Int.Int` for now unless we decide tag values should be `U32`. Current C
layout stores `uint32_t`, while `Discriminator` currently returns `Int`.

Investigation question:

- Should `EnumTag` return `Int` to match existing `discriminator`, or `U32` to
  reflect storage?

Preferred answer: return `Int` initially, because existing code treats tags as
compiler integers and `Discriminator` already casts to `Int`.

### `Switch`

`Switch(tag, arms)` should lower to a C `switch`.

The `arms` vector is dense and ordered:

- `arms[0]` handles tag 0,
- `arms[1]` handles tag 1,
- etc.

Each arm expression should be a `Goto(label)` after basic-block lowering. The C
backend can either:

- require every arm to be `Goto` and produce compact `case N: goto label;`,
- or lower each arm as a statement branch.

Preferred boundary: require every arm to be `Goto`, because that is the
basic-block invariant.

### `ReadVariant`

`ReadVariant(value, tag)` returns the payload struct for the statically selected
variant. The `tag` argument documents the dependency and keeps the operation
close to future LLVM lowering, but the result type must identify which payload
struct is being read.

The C backend should lower it as a read/cast of the enum payload union member.

Questions to answer before implementation:

- Does the backend recover the variant from `ReadVariant` result type?
- Does the backend need a `payload_qname -> variant_qname` map?
- Should the `tag` operand be lowered/evaluated in C, or is it only a semantic
  operand?

Preferred answer:

- Recover the variant from `ReadVariant` result type using a payload metadata map.
- Do not emit code for the `tag` operand unless needed for assertions/debugging.
- Keep `tag` as part of AST because future LLVM lowering can use it to justify
  the read path.

## LowerBasicBlock Module Shape

The module should be `siko/LowerBasicBlock.sk`.

Potential imports:

- `Std.Map`
- `Std.Set`
- `Siko.AST.Expr`
- `Siko.AST.Statement as S`
- `Siko.AST.Pattern`
- `Siko.AST.Type`
- `Siko.AST.Name`
- `Siko.AST.Identifier`
- `Siko.AST.QualifiedName`
- `Siko.AST.Data`
- `Siko.HIR.Program`
- `Siko.Location`
- `Siko.Utils.VarAllocator`
- `Siko.Constants.Builtins as B`
- shared variant-payload helper module, if created

Core state:

```text
BasicBlockLowerer {
    program: Program,
    var_alloc: VarAllocator,
    label_counter: Int,
    loop_stack: Vec[LoopLabels],
    payload_info: Map[QualifiedName, PayloadInfo],
    loc: Location,
}
```

`LoopLabels` should include:

- `continue_label`,
- `break_label`,
- `break_used`.

The pass entry point should probably mutate `Program` like the existing lowering
passes:

```text
pub fn lower_basic_blocks(program: Program)
```

It should:

1. Build or update variant payload structs.
2. Build payload metadata.
3. Rewrite each function body.
4. Store the updated `var_allocator` back into the function.
5. Optionally run an internal validator over every rewritten body.

## Label Allocation

Existing coroutine lowering already emits labels and gotos, so generated labels
must be collision-free.

Investigation checklist:

- Scan each function body for existing `Label` names before lowering.
- Use a compiler-owned label prefix that is a valid C label without mangling.
- Avoid `$`, `.`, `-`, brackets, and spaces because C backend currently prints
  labels directly.
- Suggested prefix:

```text
bb_<counter>_<hint>
```

where `<hint>` is one of:

- `entry`,
- `then`,
- `else`,
- `join`,
- `match`,
- `arm`,
- `loop`,
- `break`.

If a generated label collides with an existing label, increment the counter
until it does not.

The function should begin with an entry label. If the original first statement
already starts with a label, still consider inserting a generated entry label
and a `Goto` to the original first label only if needed. The simpler invariant
is:

- always emit a generated entry label first,
- then lower the original statements.

That creates a stable first block.

## Terminator Model

The pass needs a small internal notion of whether the current emitted block is
terminated.

Terminating expressions:

- `Return`
- `Goto`
- `If` with goto branches
- `Switch` with goto arms

Terminating statements:

- `Statement.Expr(terminating expression)`

When a `Label` is about to be emitted and the current block is not terminated,
insert `Goto(label)` first. This turns fallthrough into explicit control flow.

When a function finishes and the current block is not terminated:

- this should usually be an error, because the normalizer should have inserted
  explicit returns,
- except for proven infinite loops without a break, where there is no current
  fallthrough block.

Do not silently invent a return for non-unit functions.

## Statement Lowering Plan

The pass should lower normalized statements, not arbitrary source syntax.

### `Let`

Expected RHS:

- value expression,
- no `If`,
- no `Match`,
- no `Loop`,
- no `Block`.

Action:

- recursively lower children of RHS only if needed,
- emit the `Let` unchanged otherwise.

If a control-flow expression appears in RHS, panic with a message that names the
unexpected shape and says the normalizer should have lifted it.

### `Declare`

Emit unchanged.

### `Assign`

Expected RHS:

- atom/value expression.

Expected LHS:

- valid lvalue expression.

Action:

- rewrite child expressions if necessary,
- emit unchanged.

### `Expr(Block(...))`

Flatten by lowering the nested statement list into the current output.

This should be rare at the final pipeline point, but existing lowerings can
produce statement-position blocks. The basic-block pass owns final flattening.

### `TrailingExpr`

Expected not to appear.

Action:

- panic initially.

If investigation finds late `TrailingExpr` nodes still exist, decide whether to
lower them as `Expr` or convert them to `Return`. Do not guess silently.

### `Return`

Action:

- lower the return value as a value expression,
- emit `Return`,
- mark current block terminated.

If the return value contains control flow, panic.

### Existing `Goto`

Action:

- emit unchanged,
- mark current block terminated.

### Existing `Label`

Action:

- if current block is not terminated, emit `Goto(label)` before the label,
- emit label,
- mark current block open.

### `Loop`

Lower to labels and gotos.

Preferred shape:

```text
loop_header:
    <body>
    goto loop_header
loop_break:
```

`continue` targets `loop_header`.

`break` targets `loop_break`.

Important detail: only emit `loop_break` if a break in this loop was actually
lowered. Otherwise a final infinite loop would create a fake fallthrough block.

Nested loops need separate loop labels. A `break` or `continue` inside a nested
loop must target the nested loop, not the outer loop.

### `Break`

Action:

- require a loop stack entry,
- mark that loop's `break_used = True`,
- emit `Goto(break_label)`,
- mark current block terminated.

### `Continue`

Action:

- require a loop stack entry,
- emit `Goto(continue_label)`,
- mark current block terminated.

## If Lowering Plan

Input shape:

```text
Statement.Expr(ExprKind.If(cond, then_body, else_body))
```

Expected:

- `cond` is already an atom/value expression of `Bool`,
- `then_body` and `else_body` are statement-position unit blocks,
- no value-producing `if` remains.

Lowering shape:

```text
if cond {
    goto then_label;
} else {
    goto else_label;
}

then_label:
    <lower then_body>
    goto join_label   // only if branch can fall through

else_label:
    <lower else_body or empty>
    goto join_label   // only if branch can fall through

join_label:
```

If there is no else branch, the else label should represent the fallthrough path:

```text
if cond {
    goto then_label;
} else {
    goto join_label;
}

then_label:
    <lower then_body>
    goto join_label if needed

join_label:
```

If both branches terminate, the join label is not needed.

If one branch terminates and the other falls through, the join label is still
needed only if there are following statements. For a simpler first
implementation, always create the join label when any branch falls through.

The `If` expression emitted as a terminator should have:

- `then_branch = Goto(then_label)`,
- `else_branch = Some(Goto(else_label_or_join_label))`,
- unit type.

## Enum Match Lowering Plan

Input shape:

```text
Statement.Expr(ExprKind.Match(MatchInfo(scrutinee, arms)))
```

Expected:

- `scrutinee.ty` is a named enum type.
- Every arm pattern is either:
  - `PatternKind.Named` for an enum variant,
  - `PatternKind.Wildcard`.
- No int, char, string, tuple, or struct patterns remain.
- Variant subpatterns are only bindings or wildcards generated by `LowerMatch`.
- Match validation has already proved exhaustiveness.

If any of these fail, panic in `LowerBasicBlock` with a message that names the
unexpected pattern or scrutinee type.

### Basic Shape

For:

```text
match value {
    VariantA(a) => body_a,
    VariantB => body_b,
    _ => body_default,
}
```

emit:

```text
let tag = EnumTag(value);
switch tag [
    goto arm_0,
    goto arm_1,
    ...
]

arm_0:
    let payload = ReadVariant(value, tag);
    let a = payload.item0;
    <body_a>
    goto join

arm_1:
    <body_b>
    goto join

join:
```

### Switch Arm Construction

Enum tags are compiler-owned, dense, start at 0, and follow
`EnumDef.variants` order.

Build a vector of target labels with length `enum_def.variants.len()`.

For each named variant arm:

- find variant index in `enum_def.variants`,
- set `targets[index] = arm_label`.

For wildcard arm:

- remember `default_label`,
- fill all unset target slots with `default_label`.

After filling:

- assert all target slots are set,
- emit `ExprKind.Switch(tag_var_expr, goto_exprs)`.

### Match Arm Blocks

For each named variant arm:

1. Emit the arm label.
2. If any variant subpattern needs binding:
   - find the payload struct type for that variant,
   - allocate a payload temp,
   - emit `Let(payload_tmp, payload_ty, ReadVariant(scrutinee, tag))`,
   - for each binding subpattern:
     - emit `Let(binding_var, item_ty, FieldAccess(payload_tmp, itemN))`.
3. Lower the arm body.
4. If the arm body can fall through, emit `Goto(join_label)`.

For a wildcard arm:

1. Emit the wildcard label.
2. Lower the arm body.
3. If it can fall through, emit `Goto(join_label)`.

Do not emit `ReadVariant` for a wildcard arm unless the representation later
requires it. Wildcard arms have no pattern bindings.

### Guards

`LowerMatch.lower_leaf` handles guarded matches by building ordinary `If`
expressions inside the leaf body. The enum match arms that survive into
`LowerBasicBlock` should not contain guard patterns.

Investigation check:

- add an assertion that `PatternKind.Guard` does not appear in a final enum
  match arm pattern.
- run `test/success/std/match_guard`.

If guards still appear in match arm patterns, fix `LowerMatch` first rather than
special-casing guards in basic-block lowering.

## Value Expression Rewriting

The basic-block pass should not be a second normalizer. It should reject
control-flow expressions in value position.

Allowed value-position expressions should include:

- literals,
- variables,
- function calls,
- function pointer calls,
- field access,
- tuple index if any remain,
- binary/unary ops after builtin lowering,
- transmute,
- sizeof,
- discriminator if any remain,
- deref/addrof,
- `EnumTag`,
- `ReadVariant`.

Disallowed value-position expressions:

- `If`,
- `Match`,
- `Block`,
- `Loop`,
- `Break`,
- `Continue`,
- `Goto`,
- `Label`,
- `Switch`,
- `Return`.

`Switch` should be statement-position only.

## Validator Plan

Add an internal validation step at the end of `lower_basic_blocks` before wiring
the C backend.

The validator should check:

- Body is `Block`.
- First statement is a label.
- No nested blocks remain.
- No `Match` remains.
- No `Loop` remains.
- No `Break`/`Continue` remains.
- No `TrailingExpr` remains.
- Every label is unique.
- Every goto target exists.
- Every block terminates before the next label.
- The final block terminates.
- Every `If` terminator has:
  - `then_branch` as `Goto`,
  - `else_branch` as `Some(Goto)`.
- Every `Switch` terminator has only `Goto` arms.
- Every `Switch` tag type is the chosen enum-tag type.
- Every `ReadVariant` result type is a generated payload struct type.

This validator can be local to `LowerBasicBlock` rather than extending
`SanityCheck`, because the standard sanity checker currently panics on the new
basic-block primitives. Once the pass is stable, decide whether to teach
`SanityCheck` about post-basic-block mode.

## C Backend Guard Plan

Once `LowerBasicBlock` is wired in:

- Change C backend `ExprKind.Match` handling from "lower enum match" to a panic.
- Change C backend `Statement.Loop` handling to a panic.
- Change C backend `Break`/`Continue` handling to a panic.
- Keep `Label`/`Goto` lowering.
- Add `EnumTag`, `Switch`, and `ReadVariant` lowering.

During migration, it is acceptable to keep enum `Match` lowering temporarily
behind a clear comment if needed to keep tests passing between commits. The
final state should not rely on it.

## Staged Implementation Sequence

Do not implement all pieces at once. Use the following order.

### Stage 1: Payload Metadata Investigation

Deliverables:

- Decide payload qname scheme.
- Decide whether empty payload structs are generated.
- Decide shared helper module location.
- Write a small helper that can compute payload info from `Program.enums`.

Validation:

- No behavior change yet.
- Compiler still builds.

### Stage 2: Real Payload Struct Generation

Deliverables:

- Insert payload structs into `program.structs`.
- Ensure fields are `item0`, `item1`, etc.
- Ensure `value_type` mirrors enum value type.
- Ensure no duplicate structs are inserted.

Validation:

- Dump after the new helper/pass and inspect generated structs.
- Confirm existing C backend still either ignores them safely or does not
  duplicate names in emitted C.

Important risk:

- Current C backend also synthesizes payload structs. If real payload structs
  use the same C names as synthesized ones, C emission will duplicate structs.
  Either change C backend in the same stage or choose names that make the
  transition safe.

### Stage 3: C Backend Payload Layout Update

Deliverables:

- Enum container uses real payload struct qnames.
- Variant constructors use real payload struct fields.
- Backend no longer synthesizes payload struct definitions.

Validation:

- Existing regression tests pass before `LowerBasicBlock` exists.
- Existing enum `Match` lowering still works through old C match path.

This stage makes the backend representation match the future `ReadVariant`
design before the control-flow rewrite lands.

### Stage 4: C Backend New Primitive Support

Deliverables:

- Lower `EnumTag`.
- Lower statement-position `Switch`.
- Lower `ReadVariant`.
- Keep `Match` fallback temporarily if needed.

Validation:

- Add a tiny manual/generated AST test only if there is already a local test
  pattern for pass-level IR. Otherwise rely on Stage 5.

### Stage 5: LowerBasicBlock Without Match First

Deliverables:

- Implement function body flattening.
- Implement label allocation.
- Implement existing label/goto preservation.
- Implement loop/break/continue lowering.
- Implement if lowering.
- Do not lower `Match` yet; leave it with a clear panic in the pass.

Validation:

- Run tests that exercise:
  - loops,
  - break,
  - continue,
  - coroutines,
  - existing labels/gotos.

This isolates control-flow flattening bugs from enum payload bugs.

### Stage 6: Enum Match To Switch

Deliverables:

- Lower enum `Match` to `EnumTag`, `Switch`, labels, `ReadVariant`, field lets,
  and gotos.
- Validate all match invariants.

Validation:

- Enum no-payload match.
- Enum payload match.
- Nested enum match.
- Enum match with wildcard.
- Enum match with guards.
- Coroutine code that matches `CoResult`.

### Stage 7: Wire Before C Backend

Deliverables:

- Add `import Siko.LowerBasicBlock`.
- Insert pass after `value-types` and before `c`.
- Add pass dump with `driver.program.format()`.
- Decide whether to run internal validator only inside the pass or also through
  `run_sanity_check`.

Validation:

- `./base.bin check siko`
- build a new compiler binary
- use new compiler binary for focused pass dumps
- full regression suite

### Stage 8: Remove Legacy C Match/Loop Paths

Deliverables:

- C backend panics on `Match`.
- C backend panics on `Loop`, `Break`, `Continue`.
- C backend only accepts basic-block primitives for late control flow.

Validation:

- Full regression suite.
- Focused test intentionally leaking a non-enum match, if feasible, should fail
  before C emission or at the C backend guard.

## Focused Test Matrix

Use existing tests first:

- `test/success/std/match_int`
  - proves int match does not reach C as `Match`.
- `test/success/std/match_string`
  - proves string match does not reach C as `Match`.
- `test/success/std/match_struct`
  - proves struct match destructures before C.
- `test/success/nostd/basics/match/basic`
  - enum match without payload.
- `test/success/nostd/basics/match/binding`
  - enum match with binding-like behavior.
- `test/success/nostd/basics/match/nested`
  - nested enum match.
- `test/success/std/match_guard`
  - guard lowering interaction.
- `test/success/std/range`
  - loops plus match.
- `test/success/std/while_loop`
  - desugared loop behavior.
- `test/success/std/eventloop/echo5/async`
  - coroutine labels, gotos, loops, and enum matches.
- `test/success/std/eventloop/echo4`
  - non-async event loop path.

Add new focused tests only if the existing suite does not cover:

- char literal match after `LowerMatch`,
- enum payload match with multiple fields,
- wildcard enum arm with missing variants,
- loop with no break at function tail,
- nested loop with inner and outer break/continue.

## Pass Dump Checks

After the pass exists, useful dump checks:

- `--pass lower-match`
  - no non-enum matches remain.
- `--pass lower-basic-block`
  - function bodies are single flat blocks.
  - labels appear before every block.
  - no `match` remains.
  - no `loop` remains.
  - enum matches appear as `enum_tag!`, `switch`, `read_variant!`.
- `--pass c`
  - C contains labels/gotos/switches for basic-block control flow.
  - no backend match lowering is involved.

When inspecting pass dumps, use the newly built compiler binary. The bootstrap
`./base.bin` will not include the changes until rebuilt.

## Specific Failure Messages To Add

Good panic messages are part of the plan because this pass is enforcing a new
IR boundary.

Suggested messages:

- `lower-basic-block: function body is not a block`
- `lower-basic-block: TrailingExpr reached basic block lowering`
- `lower-basic-block: If in value position; normalizer should have lifted it`
- `lower-basic-block: Match in value position; normalizer should have lifted it`
- `lower-basic-block: non-enum match reached basic block lowering`
- `lower-basic-block: unexpected enum match pattern`
- `lower-basic-block: missing switch target for enum variant <name>`
- `lower-basic-block: break outside loop`
- `lower-basic-block: continue outside loop`
- `lower-basic-block: duplicate label <label>`
- `lower-basic-block: goto target does not exist <label>`
- `lower-basic-block: unterminated block before label <label>`
- `lower-basic-block: final block is unterminated`
- `cbackend: Match reached backend after basic block lowering`
- `cbackend: Loop reached backend after basic block lowering`
- `cbackend: ReadVariant result type is not a generated payload struct`

## Open Questions To Resolve Before Code

1. Payload qualified name scheme:
   - new `QualifiedName.BaseName` case, or reserved ordinary struct namespace?

2. Empty payload structs:
   - generate for every variant, or only for variants with payload?

3. Payload struct `value_type`:
   - mirror enum `value_type`, or compute differently?

4. `EnumTag` result type:
   - `Int` to match `Discriminator`, or `U32` to match storage?

5. `ReadVariant` lowering:
   - recover variant from result payload type, or carry explicit metadata
     elsewhere?

6. C backend transition:
   - update real payload structs before basic-block lowering, or in the same
     stage as match-to-switch?

7. Infinite loops:
   - implement lazy break-label emission, or add a separate loop-break analysis?

8. Existing labels:
   - always insert entry label, or reuse the first existing label when present?

9. Validation:
   - local post-pass validator only, or update `SanityCheck` to understand a
     post-basic-block mode?

The first implementation should not proceed until these are answered in code
comments or in an update to this document.

## Acceptance Criteria

The basic-block work is ready when all of the following are true:

- `LowerBasicBlock` is wired immediately before C backend lowering.
- Every function body after `lower-basic-block` is a flat label/goto/switch/if
  block.
- No `Match` reaches C backend.
- No `Loop`, `Break`, or `Continue` reaches C backend.
- Enum matches lower to `EnumTag`, `Switch`, `ReadVariant`, field lets, and
  gotos.
- Non-enum matches are still eliminated by `LowerMatch`.
- Variant payload structs are real `Program.structs`.
- C backend no longer synthesizes variant payload structs.
- C backend lowers the new basic-block primitives.
- Existing regression tests pass.
- Focused pass dumps show the expected IR shape for int, string, struct, enum,
  loop, and coroutine cases.

