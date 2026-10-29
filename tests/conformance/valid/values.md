# Value forms

Strings keep the raw text between their quotes, escapes included, and may
span lines. Numbers may carry a sign; a decimal point makes a float.

```cola
sample:
    double: "say \"hi\"",
    single: 'it\'s',
    multi: "two
lines",
    neg: -1,
    pos: +2,
    float: 1.50,
    padded: 007,
    yes: true,
    no: false
;
```
