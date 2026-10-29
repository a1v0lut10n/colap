# Nested batching under a key

```cola
llm plural llms:
    openai:
        model plural models:
            gpt-4:
                n: 1
            ;
        ;
    ;
;
```

Later.

```cola
llm plural llms:
    openai:
        model plural models:
            gpt-5:
                n: 2
            ;
        ;
    ;
;
```
