# The meta-schema

The schema dialect, declared in the dialect. Every schema `colap`
loads is first validated against this document with the same
validator that then validates documents against the schema; this
document validates itself. The one thing the mirrored tree cannot say
on its own, that a declaration contains declarations, `like` says.

```cola
schema:
    name: "cola-schema",
    version: 1,
    entity plural entities:
        schema:
            required: true,
            field plural fields:
                name: type: string;
                version: type: integer, min: 1;
                utc_suffix: type: boolean;
                unknown: type: symbol, one_of: [error, warn, allow];
            ;
            entity plural entities:
                entity:
                    plural: entities,
                    doc: "One declaration per document entity, keyed by its name.",
                    field plural fields:
                        plural: type: symbol;
                        required: type: boolean;
                        unknown: type: symbol, one_of: [error, warn, allow];
                        key_pattern: type: string;
                        like: type: reference;
                        doc: type: string;
                    ;
                    entity plural entities:
                        field:
                            plural: fields,
                            doc: "One declaration per field, keyed by its name.",
                            field plural fields:
                                type:
                                    type: symbol,
                                    one_of: [
                                        string,
                                        integer,
                                        float,
                                        number,
                                        decimal,
                                        boolean,
                                        symbol,
                                        reference,
                                        instant,
                                        date,
                                        datetime,
                                        edtf,
                                        any,
                                    ],
                                ;
                                list: type: boolean;
                                required: type: boolean;
                                one_of: type: any, list: true;
                                min: type: decimal;
                                max: type: decimal;
                                pattern: type: string;
                                to: type: string;
                                doc: type: string;
                            ;
                        ;
                        entity:
                            plural: entities,
                            like: @schema/entity/schema/entity/entity,
                        ;
                    ;
                ;
            ;
        ;
    ;
;
```
