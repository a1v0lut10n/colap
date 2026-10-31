# A registry that matches its schema

```cola
llm plural llms:
    gpt-4.1:
        name: "gpt-4.1",
        max_input_tokens: 1_048_576,
        output_price: 8.0,
        supports_vision: true,
        basis: parsed,
        tags: [flagship, vision],
        vendor: @vendors/openai,
        recorded_at_utc: "2026-10-30T14:05:00Z",
        id: "model-41",
        weight: 0.333333333333333333333333,
        note: [1, "two", three],
        api: type: REST, key: "k";
        extra: anything: goes, here: too;
    ;
    tiny:
        name: "tiny",
        recorded_at_utc: "2026-10-30T14:05:00.250Z",
        api: type: gRPC;
    ;
;
vendor plural vendors:
    openai:
        name: "OpenAI",
        founded: "2015-12",
        active: "2015-12/..",
        meeting: "2026-10-30T09:00[Europe/Zurich]",
    ;
;
database: url: "sqlite://registry.db";
```
