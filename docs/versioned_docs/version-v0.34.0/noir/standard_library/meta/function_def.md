---
title: FunctionDefinition
---

`std::meta::function_def` contains methods on the built-in `FunctionDefinition` type representing
a function definition in the source program.

## Methods

### body

```rust title="body" showLineNumbers 
fn body(self) -> Expr {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L3-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L3-L5</a></sub></sup>


Returns the body of the function as an expression. This is only valid 
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### name

```rust title="name" showLineNumbers 
fn name(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L8-L10" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L8-L10</a></sub></sup>


Returns the name of the function.

### parameters

```rust title="parameters" showLineNumbers 
fn parameters(self) -> [(Quoted, Type)] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L13-L15" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L13-L15</a></sub></sup>


Returns each parameter of the function as a tuple of (parameter pattern, parameter type).

### return_type

```rust title="return_type" showLineNumbers 
fn return_type(self) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L18-L20" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L18-L20</a></sub></sup>


The return type of the function.

### set_body

```rust title="set_body" showLineNumbers 
fn set_body(self, body: Expr) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L23-L25" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L23-L25</a></sub></sup>


Mutate the function body to a new expression. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

### set_parameters

```rust title="set_parameters" showLineNumbers 
fn set_parameters(self, parameters: [(Quoted, Type)]) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L28-L30" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L28-L30</a></sub></sup>


Mutates the function's parameters to a new set of parameters. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.

Expects a slice of (parameter pattern, parameter type) for each parameter. Requires
each parameter pattern to be a syntactically valid parameter.

### set_return_type

```rust title="set_return_type" showLineNumbers 
fn set_return_type(self, return_type: Type) {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/function_def.nr#L33-L35" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/function_def.nr#L33-L35</a></sub></sup>


Mutates the function's return type to a new type. This is only valid
on functions in the current crate which have not yet been resolved.
This means any functions called at compile-time are invalid targets for this method.
