# A schema with constraints that do not fit their types

```cola
schema:
    entity plural entities:
        llm:
            plural: llms,
            field plural fields:
                basis: type: symbol, one_of: ["parsed", derived];
                size: type: integer, min: 10, max: 1;
                id: type: string, pattern: "[a-";
                count: type: integer, pattern: "[0-9]+";
                vendor: type: string, to: "vendor";
            ;
            entity plural entities:
                api:
                    like: @schema/entity/llm,
                    field plural fields: key: type: string; ;
                ;
                other: like: @schema;
            ;
        ;
        database:
            key_pattern: "x",
            field plural fields: url: type: string; ;
        ;
    ;
;
```
