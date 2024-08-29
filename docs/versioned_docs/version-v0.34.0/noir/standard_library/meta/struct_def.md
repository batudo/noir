---
title: StructDefinition
---

`std::meta::struct_def` contains methods on the built-in `StructDefinition` type.
This type corresponds to `struct Name { field1: Type1, ... }` items in the source program.

## Methods

### as_type

```rust title="as_type" showLineNumbers 
fn as_type(self) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L5-L7" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L5-L7</a></sub></sup>


Returns this struct as a type in the source program. If this struct has
any generics, the generics are also included as-is.

### generics

```rust title="generics" showLineNumbers 
fn generics(self) -> [Type] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L11-L13" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L11-L13</a></sub></sup>


Returns each generic on this struct.

Example:

```
#[example]
struct Foo<T, U> {
    bar: [T; 2],
    baz: Baz<U, U>,
}

comptime fn example(foo: StructDefinition) {
    assert_eq(foo.generics().len(), 2);

    // Fails because `T` isn't in scope
    // let t = quote { T }.as_type();
    // assert_eq(foo.generics()[0], t);
}
```

### fields

```rust title="fields" showLineNumbers 
fn fields(self) -> [(Quoted, Type)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/struct_def.nr#L18-L20" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/struct_def.nr#L18-L20</a></sub></sup>


Returns each field of this struct as a pair of (field name, field type).
