# Comments

```cola
// A comment before the first entity leads it.
llm plural llms: // after the header
    // before openai
    openai:
        api: type: "REST", key: "k"; // after a one-line entity
        url: "http://example.org", // not a comment inside the string
        tags: [alpha, // inside a list
            beta],
        // before the close of openai
    ;
    gpt-4.1: name: "gpt-4.1",
        price: 8.0;
; // after the close
// At the end of the block.
```

Prose.

```cola
// Only a comment in this block.
```
