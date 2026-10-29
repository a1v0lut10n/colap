# Keywords as names

The lexer is driven by the parser state, so `plural`, `true` and `false`
are only keywords where a keyword can stand. Elsewhere they are names.
Accepted, not recommended.

```cola
plural:
    true: 1,
    false: 2
;
```
