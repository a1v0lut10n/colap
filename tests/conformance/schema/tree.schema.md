# A tree: a declaration that contains itself

`like` names another declaration; here the one it sits in.

```cola
schema:
    entity plural entities:
        folder:
            plural: folders,
            field plural fields:
                name: type: string, required: true;
            ;
            entity plural entities:
                folder: plural: folders, like: @schema/entity/folder;
            ;
        ;
    ;
;
```
