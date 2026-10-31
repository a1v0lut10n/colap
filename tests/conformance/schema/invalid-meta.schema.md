# A schema the meta-schema rejects

```cola
schema:
    entity plural entities:
        llm:
            plural: llms,
            requried: true,
            field plural fields:
                name: type: strnig;
                size: type: integer, min: "one";
            ;
            entity plural entities:
                api: field plural fields: key: type: string; ; ;
            ;
            misc: x: 1;
        ;
    ;
;
```
