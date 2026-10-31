# Numbers of any size, kept as written

The grammar accepts any digit string and the model keeps it exactly:
no 64-bit limit, no rounding, no respelling beyond dropping separators.

```cola
limits:
    max: 99999999999999999999,
    precise: 0.30000000000000004441,
    trailing: 1.10,
    scaled: 2E10,
    wide: 123_456_789_012_345_678_901_234_567_890,
;
```
