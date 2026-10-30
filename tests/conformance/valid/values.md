# Value forms

Quoted strings decode their escapes; a block string keeps its text as
written, less the common indentation. Numbers may carry a sign, digit
separators and an exponent; a decimal point or an exponent makes a float.

```cola
sample:
    double: "say \"hi\"",
    single: 'it\'s',
    escapes: "tab\tnewline\nbackslash\\unicode\u{e9}",
    multi: "two
lines",
    block: """
        A gloss over
          two lines, one indented.
        """,
    neg: -1,
    pos: +2,
    float: 1.50,
    padded: 007,
    grouped: 1_000_000,
    exponent: 1.5e-3,
    big: 2E10,
    yes: true,
    no: false
;
```
