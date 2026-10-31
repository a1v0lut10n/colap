# A schema that opts out of the `_utc` naming rule

```cola
schema:
    utc_suffix: false,
    unknown: allow,
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
