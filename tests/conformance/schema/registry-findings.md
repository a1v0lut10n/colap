# A registry with one of every finding

```cola
llm plural llms:
    Bad:
        max_input_tokens: "many",
        output_price: 101,
        supports_vision: "yes",
        basis: guessed,
        tags: alpha,
        vendor: @llms/other,
        recorded_at_utc: "2026-02-30T00:00:00Z",
        id: "X",
        nmae: "typo",
        api: type: SOAP;
    ;
    other:
        name: "other",
        max_input_tokens: 0,
        basis: [parsed],
        recorded_at_utc: "2026-10-30T14:05:00+02:00",
        api: type: REST;
    ;
    huge:
        name: "huge",
        max_input_tokens: 99999999999999999999,
        recorded_at_utc: "2026-10-30T14:05:00Z",
        api: type: REST;
    ;
    fractional:
        name: "fractional",
        max_input_tokens: 8.0,
        recorded_at_utc: "2026-10-30T14:05:00Z",
        api: type: REST;
    ;
;
vendor plural vendors:
    acme:
        name: "Acme",
        founded: "1914-13",
        active: "1914??",
        meeting: "2026-10-30T09:00Z",
        motto: "unknown, but only a warning here",
    ;
;
database plural databases:
    main: url: "x";
;
stray: x: 1;
```
