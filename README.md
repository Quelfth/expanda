The [`expand`] macro replaces the common `macro_rules!` use
case where you define a macro and then immediately invoke it
one time and never again.

Here's an example of how such a macro could be simplified.
Imagine we have
```
trait IntAdd {
    fn int_add(self, rhs: Self) -> Self;
}
```
We can implement this for all integer types using `macro_rules!` like
```
macro_rules! impl_int_add {
    ($($T:tt)*) => {$(
        impl IntAdd for $T {
            fn int_add(self, rhs: Self) -> Self {
                self + rhs
            }
        }
    )*}
}

impl_int_add!{
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
}
```
Using [`expand`] this could become just
```
expand! {
    <--for $T in
        u8 u16 u32 u64 u128 usize
        i8 i16 i32 i64 i128 isize
    {
        impl IntAdd for $T {
            fn int_add(self, rhs: Self) -> Self {
                self + rhs
            }
        }
    }
}
```
This is arguably not that much less code, but the `$( )*` can be hard to read,
and we have to come up with a name for this macro.  Also, in cases where you need
a cartesian product (two for loops), you would need two macros one emitting the other
to do it with `macro_rules!`.  I would argue that the [`expand`] version is a lot more
straightforward and readable.

### Syntax

Most tokens inside [`expand`] are interpreted literally.

There are two exceptions:
- [Directives](crate#directives), which begin with `<--`.
- [Interpolations](crate#interpolation), which begin with the [interpolation sigil](crate#use-directive), `$` by default.

### Directives

#### For Directive
The [`for`](crate#for-directive) directive looks like
```
<--for $loop_variable_name in token_expressions {
    body
}
```
This `$` is actually the [interpolation sigil](crate#use-directive). (whatever it may be)

`loop_variable_name` can be any identifier, and defines the metavariable
for the loop.

`token_expressions` is a list of single tokens or token sequences to be
substituted for the metavariable in sequence.  These can be either a single
non-group token (anything except `( )` `[ ]` or `{ }`), or `[tokens]` where
`tokens` is any sequence of tokens.  In the case of `[tokens]`, the `[ ]`
will not appear in the output. (but all contents, including any additional
`[ ]`, will appear).  A token expression may also be an interpolation (`$name`),
which will resolve to the result of the [interpolation](crate#interpolation).

`body` refers to the body of the loop, which will be repeated for each element
of `token_expressions` accessible as the `loop_variable_name` metavariable.

#### Let directive
The [`let`](crate#let-directive) directive looks like
```
<--let $variable_name = value
```
A [`let`](crate#let-directive) directive simply binds a particular metavalue to a metavariable
for use in future [interpolations](crate#interpolation).

The `$` is actually the [interpolation sigil](crate#use-directive). (whatever it may be)

[`let`](crate#let-directive) directives are allowed to redefine existing metavariables.  Any
`( )`, `[ ]` or `{ }` will introduce a new scope for these bindings.
So any bindings created in a scope will last only for that scope.

`variable_name` is the name of the metavariable to assign.

`value` is the value to assign to the binding, it follows the same
rules as the `token_expressions` in a [`for`](crate#for-directive) directive, except that there can be only one.

A `*` can be placed before the variable name (`$*name`) to define a metalist
instead of a metavariable.  Metalists have a separate namespace from metavariables.
When defining a metalist, the `value` should be in `( )`, and can be any number of values.
Unlike regular metavariables, metalists can only be expanded as the right hand argument
to a [`for`](crate#for-directive) directive.  In this situation, all of their values will be looped over by the
[`for`](crate#for-directive).

#### Use directive
[`use`](crate#use-directive) directives allow you to change the interpolation sigil.
```
<--use #
```
for instance, will change the interpolation sigil to `#` with the
same scoping rules as `let` directives.  This is useful when invoking
`expand` in a context where `$` isn't available, such as a `macro_rules!`
definition. Any punctuation character
can be used in place of the `#` to use that as the interpolation sigil.
Notably, all instances of the interpolation sigil will introduce an
interpolation regardless of rust syntax rules, so using a common rust
character such as `+` would be ill-advised.  Good options include
- `~` Rust doesn't use this character at all, but it is a valid token.
- `#` This is only used in attributes. (`#[ ]`)
- `@` This is only used in pattern bindings (`let x @ some_pattern = ...`)

### Interpolation
Interpolation is invoked by the [interpolation sigil](crate#use-directive), `$` by default.

This documentation will assume that the interpolation sigil is `$`,
note that all instances of `$` refer to whatever the interpolation sigil
happens to be.

An interpolation sequence beginning with `$` can have one of four forms:
- `$name`: This will be substituted for the `name` metavariable.
- `${ some_meta_expression }`: This evaluates a [meta expression](crate#meta-expressions).
- `$*list_name`: This will be substituted for a metalist.  This is allowed
    only as part of the right hand side of a [`for`](crate#for-directive) directive.
- `$$`: This produces the interpolation sigil itself as a token.

All `expand` invocations automatically start with one defined metavariable
named `dollar_sign` which evaluates to `$`.  This is not the [interpolation sigil](crate#use-directive),
but rather always a `$` character.  This is useful since it allows you to use
[`expand`] to create a `macro_rules!` as the output of another `macro_rules!`.
Doing this also requires a [`<--use #`](crate#use-directive) or similar directive.

### Meta Expressions
A meta expression consists of a sequence of units which will be concatenated
together to form a single token.
These are the types of unit:
- `name`: This will resolve to the metavariable `name`
- `[tokens]`: This resolves to the literal `tokens`
- `"name"`: This takes the value of the metavariable `name` and then stringifies it.
    You can use the string prefixes `b""` and `c""` to stringify into different kinds of string literals.

If multiple units are present within `{ }`, they will be concatenated to form a single token.
If this concatenation is impossible for any reason, it will produce a compiler error.