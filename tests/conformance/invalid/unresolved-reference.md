# A reference to nothing

Every reference must name an entity in the model; a key under the wrong
plural name, or a field, does not count.

```cola
claim plural claims:
    cl-1: supports: @claims/cl-9, also: [@claims/cl-1/basis], basis: parsed;
;
```
