# A schema that breaks the `_utc` naming rule

```cola
schema:
    entity plural entities:
        event:
            plural: events,
            field plural fields:
                recorded_at: type: instant;
                created_utc: type: string;
            ;
        ;
    ;
;
```
