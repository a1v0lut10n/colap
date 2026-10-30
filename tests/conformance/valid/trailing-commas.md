# Trailing commas

A comma may follow the last field of an entity, whether `;` or a nested
entity comes next, and the last element of a list.

```cola
llm plural llms:
    openai:
        api: type: "REST", key: "k",;
        model plural models:
            gpt-4.1:
                name: "gpt-4.1",
                max_input_tokens: 1_048_576,
                tags: [
                    fast,
                    vision,
                ],
            ;
        ;
        region: "eu",
        limits: rpm: 60,;
    ;
;
```
