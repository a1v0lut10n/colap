# Symbols, lists and references

An identifier in value position is a symbol; `plural` is one too, since
it is a keyword only where a keyword can stand. A list holds scalars of
any kind, and may be empty. A reference names an entity by its path from
the root, and may point forward or into another block.

```cola
claim plural claims:
    cl-1:
        basis: parsed,
        shape: plural,
        tags: [alpha, "two words", 3, 1.5, true, @claims/cl-2],
        none: [],
        supports: @claims/cl-2
    ;
;
```

A plural is reachable by either of its names, and a nested entity by the
names along the way.

```cola
claim plural claims:
    cl-2:
        basis: inferred,
        tags: [beta,],
        supports: @claim/cl-1,
        detail: note: @claims/cl-2/detail;
    ;
;
```
