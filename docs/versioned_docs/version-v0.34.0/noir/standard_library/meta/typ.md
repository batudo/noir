---
title: Type
---

`std::meta::typ` contains methods on the built-in `Type` type used for representing
a type in the source program.

## Methods

### as_array

```rust title="as_array" showLineNumbers 
fn as_array(self) -> Option<(Type, Type)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L6-L8" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L6-L8</a></sub></sup>


If this type is an array, return a pair of (element type, size type).

Example:

```rust
comptime {
    let array_type = quote { [Field; 3] }.as_type();
    let (field_type, three_type) = array_type.as_array().unwrap();

    assert(field_type.is_field());
    assert_eq(three_type.as_constant().unwrap(), 3);
}
```

### as_constant

```rust title="as_constant" showLineNumbers 
fn as_constant(self) -> Option<u32> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L11-L13" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L11-L13</a></sub></sup>


If this type is a constant integer (such as the `3` in the array type `[Field; 3]`),
return the numeric constant.

### as_integer

```rust title="as_integer" showLineNumbers 
fn as_integer(self) -> Option<(bool, u8)> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L16-L18" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L16-L18</a></sub></sup>


If this is an integer type, return a boolean which is `true`
if the type is signed, as well as the number of bits of this integer type.

### as_slice

```rust title="as_slice" showLineNumbers 
fn as_slice(self) -> Option<Type> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L21-L23" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L21-L23</a></sub></sup>


If this is a slice type, return the element type of the slice.

### as_struct

```rust title="as_struct" showLineNumbers 
fn as_struct(self) -> Option<(StructDefinition, [Type])> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L26-L28" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L26-L28</a></sub></sup>


If this is a struct type, returns the struct in addition to
any generic arguments on this type.

### as_tuple

```rust title="as_tuple" showLineNumbers 
fn as_tuple(self) -> Option<[Type]> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L31-L33" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L31-L33</a></sub></sup>


If this is a tuple type, returns each element type of the tuple.

### get_trait_impl

```rust title="get_trait_impl" showLineNumbers 
fn get_trait_impl(self, constraint: TraitConstraint) -> Option<TraitImpl> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L36-L38" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L36-L38</a></sub></sup>


Retrieves the trait implementation that implements the given
trait constraint for this type. If the trait constraint is not
found, `None` is returned. Note that since the concrete trait implementation
for a trait constraint specified from a `where` clause is unknown,
this function will return `None` in these cases. If you only want to know
whether a type implements a trait, use `implements` instead.

Example:

```rust
comptime {
    let field_type = quote { Field }.as_type();
    let default = quote { Default }.as_trait_constraint();

    let the_impl: TraitImpl = field_type.get_trait_impl(default).unwrap();
    assert(the_impl.methods().len(), 1);
}
```

### implements

```rust title="implements" showLineNumbers 
fn implements(self, constraint: TraitConstraint) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L41-L43" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L41-L43</a></sub></sup>


`true` if this type implements the given trait. Note that unlike
`get_trait_impl` this will also return true for any `where` constraints
in scope.

Example:

```rust
fn foo<T>() where T: Default {
    comptime {
        let field_type = quote { Field }.as_type();
        let default = quote { Default }.as_trait_constraint();
        assert(field_type.implements(default));

        let t = quote { T }.as_type();
        assert(t.implements(default));
    }
}
```

### is_bool

```rust title="is_bool" showLineNumbers 
fn is_bool(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L46-L48" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L46-L48</a></sub></sup>


`true` if this type is `bool`.

### is_field

```rust title="is_field" showLineNumbers 
fn is_field(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/typ.nr#L51-L53" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/typ.nr#L51-L53</a></sub></sup>


`true` if this type is `Field`.

## Trait Implementations

```rust
impl Eq for Type
```
Note that this is syntactic equality, this is not the same as whether two types will type check
to be the same type. Unless type inference or generics are being used however, users should not
typically have to worry about this distinction.
