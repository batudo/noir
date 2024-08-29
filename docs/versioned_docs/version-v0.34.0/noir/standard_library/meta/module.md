---
title: Module
---

`std::meta::module` contains methods on the built-in `Module` type which represents a module in the source program.
Note that this type represents a module generally, it isn't limited to only `mod my_submodule { ... }`
declarations in the source program.

## Methods

### name

```rust title="name" showLineNumbers 
fn name(self) -> Quoted {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L13-L15" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L13-L15</a></sub></sup>


Returns the name of the module.

### functions

```rust title="functions" showLineNumbers 
fn functions(self) -> [FunctionDefinition] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L8-L10" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L8-L10</a></sub></sup>


Returns each function in the module.

### is_contract

```rust title="is_contract" showLineNumbers 
fn is_contract(self) -> bool {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/module.nr#L3-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/module.nr#L3-L5</a></sub></sup>


`true` if this module is a contract module (was declared via `contract foo { ... }`).
