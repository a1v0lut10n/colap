# Comments

`//` starts a comment that runs to the end of the line, inside cola
blocks only. The parser skips it as layout; the model keeps it.

```cola
// Leads the entity below.
setting plural settings: // trails the header
    // leads `verbose`
    verbose: value: true; // trails the one-line entity's `;`
    url:
        value: "http://example.org/not-a-comment", // trails the field
        gloss: """
            A // inside a block string is text too.
        """, // trails the block string's field
        tags: [
            alpha, // inside a list: leads the field after formatting
            beta,
        ],
        // leads the close of `url`
    ; // trails the close of `url`
;
// Ends the block.
```

A block may hold nothing but a comment:

```cola
// Nothing here.
```
