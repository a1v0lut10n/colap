# Messy

Prose stays `exactly` as it is.

```cola
llm plural llms:
    openai:
        api: type: "REST", key: "k";
        model plural models:
            gpt-4.1:
                name: "gpt-4.1",
                max_input_tokens: 1048576,
                output_price: 8.0,
                supports_vision: true
            ;
        ;
    ;
;
database: url: "";
```

Trailing prose, and a foreign fence that must not move:

```text
  spaced   out
```
