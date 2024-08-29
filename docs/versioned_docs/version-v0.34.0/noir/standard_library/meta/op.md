---
title: UnaryOp and BinaryOp
---

`std::meta::op` contains the `UnaryOp` and `BinaryOp` types as well as methods on them.
These types are used to represent a unary or binary operator respectively in Noir source code.

## Types

### UnaryOp

Represents a unary operator. One of `-`, `!`, `&mut`, or `*`.

### Methods

#### is_minus

```rust title="is_minus" showLineNumbers 
pub fn is_minus(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L6-L8" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L6-L8</a></sub></sup>


Returns `true` if this operator is `-`.

#### is_not

```rust title="is_not" showLineNumbers 
pub fn is_not(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L12-L14" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L12-L14</a></sub></sup>


`true` if this operator is `!`

#### is_mutable_reference

```rust title="is_mutable_reference" showLineNumbers 
pub fn is_mutable_reference(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L18-L20" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L18-L20</a></sub></sup>


`true` if this operator is `&mut`

#### is_dereference

```rust title="is_dereference" showLineNumbers 
pub fn is_dereference(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L24-L26" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L24-L26</a></sub></sup>


`true` if this operator is `*`

#### quoted

```rust title="unary_quoted" showLineNumbers 
pub fn quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L30-L32" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L30-L32</a></sub></sup>


Returns this operator as a `Quoted` value.

### BinaryOp

Represents a binary operator. One of `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&`, `|`, `^`, `>>`, or `<<`.

### Methods

#### is_add

```rust title="is_add" showLineNumbers 
pub fn is_add(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L52-L54" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L52-L54</a></sub></sup>


`true` if this operator is `+`

#### is_subtract

```rust title="is_subtract" showLineNumbers 
pub fn is_subtract(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L58-L60" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L58-L60</a></sub></sup>


`true` if this operator is `-`

#### is_multiply

```rust title="is_multiply" showLineNumbers 
pub fn is_multiply(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L64-L66" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L64-L66</a></sub></sup>


`true` if this operator is `*`

#### is_divide

```rust title="is_divide" showLineNumbers 
pub fn is_divide(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L70-L72" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L70-L72</a></sub></sup>


`true` if this operator is `/`

#### is_modulo

```rust title="is_modulo" showLineNumbers 
pub fn is_modulo(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L142-L144" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L142-L144</a></sub></sup>


`true` if this operator is `%`

#### is_equal

```rust title="is_equal" showLineNumbers 
pub fn is_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L76-L78" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L76-L78</a></sub></sup>


`true` if this operator is `==`

#### is_not_equal

```rust title="is_not_equal" showLineNumbers 
pub fn is_not_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L82-L84" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L82-L84</a></sub></sup>


`true` if this operator is `!=`

#### is_less_than

```rust title="is_less_than" showLineNumbers 
pub fn is_less_than(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L88-L90" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L88-L90</a></sub></sup>


`true` if this operator is `<`

#### is_less_than_or_equal

```rust title="is_less_than_or_equal" showLineNumbers 
pub fn is_less_than_or_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L94-L96" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L94-L96</a></sub></sup>


`true` if this operator is `<=`

#### is_greater_than

```rust title="is_greater_than" showLineNumbers 
pub fn is_greater_than(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L100-L102" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L100-L102</a></sub></sup>


`true` if this operator is `>`

#### is_greater_than_or_equal

```rust title="is_greater_than_or_equal" showLineNumbers 
pub fn is_greater_than_or_equal(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L106-L108" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L106-L108</a></sub></sup>


`true` if this operator is `>=`

#### is_and

```rust title="is_and" showLineNumbers 
pub fn is_and(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L112-L114" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L112-L114</a></sub></sup>


`true` if this operator is `&`

#### is_or

```rust title="is_or" showLineNumbers 
pub fn is_or(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L118-L120" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L118-L120</a></sub></sup>


`true` if this operator is `|`

#### is_shift_right

```rust title="is_shift_right" showLineNumbers 
pub fn is_shift_right(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L130-L132" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L130-L132</a></sub></sup>


`true` if this operator is `>>`

#### is_shift_left

```rust title="is_shift_right" showLineNumbers 
pub fn is_shift_right(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L130-L132" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L130-L132</a></sub></sup>


`true` if this operator is `<<`

#### quoted

```rust title="binary_quoted" showLineNumbers 
pub fn quoted(self) -> Quoted {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/op.nr#L148-L150" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/op.nr#L148-L150</a></sub></sup>


Returns this operator as a `Quoted` value.