# Ownership Inference: Group Variables

## The problem

Flattening every reachable ownership slot into a datatype's type arguments does
not scale.

```text
Program
└── Map
    └── Node
        └── ...
```

On the self-hosted compiler this attempted to unfold large parts of the HIR
datatype graph. The ownership-inference pass allocated increasingly large
vectors and did not finish.

SCC processing makes recursion finite, but copying an SCC's complete slot pool
into every recursive occurrence can still be enormous.

## Group variables

A group variable is a variadic inference variable: it represents a collection
of ordinary ownership slots.

For a recursive datatype:

```siko
struct Node {
    value: Foo,
    next: Node
}
```

we must not construct:

```text
Node[value, next.value, next.next.value, ...]
```

Instead, the recursive occurrence carries the same group variable:

```text
Node[..G] {
    value: ...
    next: Node[..G]
}
```

The meaning is:

> Whatever collection of slots this `Node` has, the inner `Node` has that same
> slot structure, represented by `G`.

The group variable compactly closes the recursive datatype equation. It is not
itself a pointer set, and it is not a replacement for profile paths.

Only this idea is borrowed from the old Siko ownership implementation; its
surrounding compiler architecture was substantially different.

## Profiles still contain paths

Function analysis creates paths only for positions used by the function:

```text
Local(node).next.value → Local(tmp)
Local(tmp) → Result.cached
```

After local nodes are projected away, the published profile remains a list of
path-to-path links:

```text
Arg(0).next.value → Result.cached
```

The two mechanisms have separate jobs:

```text
group variables   compact recursive datatype slot structure
profile paths     describe pointer flow between concrete positions
```

The next implementation decision is how to represent and substitute a group
variable in the current ownership-analysis clone. The existing behavior of
appending every variable in the SCC pool at a recursive reference should not
remain.
