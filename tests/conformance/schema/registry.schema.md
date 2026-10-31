# The registry schema

Every declaration the dialect has, for the documents beside this file.

```cola
schema:
    name: "registry",
    version: 1,
    entity plural entities:
        llm:
            plural: llms,
            required: true,
            key_pattern: "[a-z][a-z0-9.-]*",
            doc: "One entry per model.",
            field plural fields:
                name: type: string, required: true;
                max_input_tokens: type: integer, min: 1;
                output_price: type: float, max: 100;
                supports_vision: type: boolean;
                basis: type: symbol, one_of: [mirrored, parsed, derived];
                tags: type: symbol, list: true;
                vendor: type: reference, to: "vendor";
                recorded_at_utc: type: instant, required: true;
                id: type: string, pattern: "[a-z]+-[0-9]+";
                weight: type: decimal;
                note: type: any;
            ;
            entity plural entities:
                api:
                    required: true,
                    field plural fields:
                        type: type: symbol, one_of: [REST, gRPC], required: true;
                        key: type: string;
                    ;
                ;
                extra: unknown: allow;
            ;
        ;
        vendor:
            plural: vendors,
            unknown: warn,
            field plural fields:
                name: type: string, required: true;
                founded: type: date;
                active: type: edtf;
                meeting: type: datetime;
            ;
        ;
        database:
            field plural fields:
                url: type: string, required: true;
            ;
        ;
    ;
;
```
