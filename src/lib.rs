//! The [`expand`] macro replaces the common `macro_rules!` use
//! case where you define a macro and then immediately invoke it
//! one time and never again.
//! 
//! Here's an example of how such a macro could be simplified.
//! Imagine we have
//! ```
//! trait IntAdd {
//!     fn int_add(self, rhs: Self) -> Self;
//! }
//! ```
//! We can implement this for all integer types using `macro_rules!` like
//! ```
//! macro_rules! impl_int_add {
//!     ($($T:tt)*) => {$(
//!         impl IntAdd for $T {
//!             fn int_add(self, rhs: Self) -> Self {
//!                 self + rhs
//!             }
//!         }
//!     )*}
//! }
//! 
//! impl_int_add!{
//!     u8 u16 u32 u64 u128 usize
//!     i8 i16 i32 i64 i128 isize
//! }
//! ```
//! Using [`expand`] this could become just
//! ```
//! expand! {
//!     <--for $T in
//!         u8 u16 u32 u64 u128 usize
//!         i8 i16 i32 i64 i128 isize
//!     {
//!         impl IntAdd for $T {
//!             fn int_add(self, rhs: Self) -> Self {
//!                 self + rhs
//!             }
//!         }
//!     }
//! }
//! ```
//! This is arguably not that much less code, but the `$( )*` can be hard to read,
//! and we have to come up with a name for this macro.  Also, in cases where you need
//! a cartesian product (two for loops), you would need two macros one emitting the other
//! to do it with `macro_rules!`.  I would argue that the [`expand`] version is a lot more
//! straightforward and readable.
//! 
//! ### Syntax
//! 
//! Most tokens inside [`expand`] are interpreted literally.
//! 
//! There are two exceptions:
//! - [Directives](crate#directives), which begin with `<--`.
//! - [Interpolations](crate#interpolation), which begin with the [interpolation sigil](crate#use-directive), `$` by default.
//! 
//! ### Directives
//! 
//! #### For Directive
//! The [`for`](crate#for-directive) directive looks like
//! ```
//! <--for $loop_variable_name in token_expressions {
//!     body
//! }
//! ```
//! This `$` is actually the [interpolation sigil](crate#use-directive). (whatever it may be)
//!
//! `loop_variable_name` can be any identifier, and defines the metavariable
//! for the loop.
//! 
//! `token_expressions` is a list of single tokens or token sequences to be
//! substituted for the metavariable in sequence.  These can be either a single
//! non-group token (anything except `( )` `[ ]` or `{ }`), or `[tokens]` where
//! `tokens` is any sequence of tokens.  In the case of `[tokens]`, the `[ ]`
//! will not appear in the output. (but all contents, including any additional
//! `[ ]`, will appear).  A token expression may also be an interpolation (`$name`),
//! which will resolve to the result of the [interpolation](crate#interpolation).
//! 
//! `body` refers to the body of the loop, which will be repeated for each element
//! of `token_expressions` accessible as the `loop_variable_name` metavariable.
//! 
//! #### Repeat Directive
//! The [`repeat`](crate#repeat-directive) directive looks like
//! ```
//! <--repeat count {
//!     body
//! }
//! ```
//! 
//! `count` must be either an integer literal, or an [interpolation](crate#interpolation)
//! which resolves to a single integer literal.
//! 
//! `body` refers to the body of the loop, which will be repeated `count` times.
//! 
//! #### Let directive
//! The [`let`](crate#let-directive) directive looks like
//! ```
//! <--let $variable_name = value
//! ```
//! A [`let`](crate#let-directive) directive simply binds a particular metavalue to a metavariable
//! for use in future [interpolations](crate#interpolation).
//!
//! The `$` is actually the [interpolation sigil](crate#use-directive). (whatever it may be)
//! 
//! [`let`](crate#let-directive) directives are allowed to redefine existing metavariables.  Any
//! `( )`, `[ ]` or `{ }` will introduce a new scope for these bindings.
//! So any bindings created in a scope will last only for that scope.
//! 
//! `variable_name` is the name of the metavariable to assign.
//! 
//! `value` is the value to assign to the binding, it follows the same
//! rules as the `token_expressions` in a [`for`](crate#for-directive) directive, except that there can be only one.
//! 
//! #### Match Directive
//! The [`match`](crate#match-directive) directive looks like
//! ```
//! <--match value {
//!     (pattern 1) => {result 1}
//!     (pattern 2) => {result 2}
//!     ...
//! }
//! ```
//! A [`match`](crate#match-directive) directive allows you to compare a particular metavalue against
//! several patterns, and emit a specific token stream corresponding to the first pattern that matches.
//! 
//! The syntax for these patterns is as such:
//! - Any single token other than `$` (or whatever the [interpolation sigil](crate#use-directive) happens to be) will check 
//!   that the token from the input matches it exactly.
//! - When matching `( )`, `[ ]`, and `{ }`, the interior of the delimiters is another pattern that will be checked against
//!   their corresponding contents in the input.  This is almost certainly just "what you would expect to happen".
//! - A capture begins with `$` and then is followed by three optional components and one required component:
//!   - The first optional component is the capture name, which must be an identifier.
//!   - The second optional component is the quantifier, this can be a braced range like `{1..10}`, or one or the other
//!     bound can be omitted as in `{1..}` or `{..10}`, or it can be a single braced number like `{3}`.  Additionally,
//!     the quantifier can be `?`, `+`, or `*`, which are equivalent to `{0..1}`, `{1..}`, and `{0..}` respectively.
//!     The quantifier represents the number of repetitions allowed for this capture.
//!   - The third optional component is a negation like `^( )` or `^[ ]`.  This component prevents the capture from matching
//!     if the pattern inside the `( )` or `[ ]` does match.  In the case of `[ ]`, the tokens inside are each treated individually,
//!     and the pattern will fail to match if any of those match the stream.
//!   - The final component, the only required one, must be either `( )`, `[ ]`, or `.`.
//!     - For `( )`, all of the tokens inside must match the input in sequence.
//!     - For `[ ]`, any single token inside must match the input.
//!     - For `.`, this capture will match any single token from the input.
//! 
//! It is an error for the [`match`](crate#match-directive) to fail to match any of its arms.  If you would like to instead
//! simply produce nothing, you can achieve this by adding a fallback branch that matches anything, which would look like
//! ```
//! ($*.) => {}
//! ```
//! The pattern `$*.` will match a sequence of zero or more of any token.  That is, it matches any input.
//! 
//! #### Use directive
//! [`use`](crate#use-directive) directives allow you to change the interpolation sigil.
//! ```
//! <--use #
//! ```
//! for instance, will change the interpolation sigil to `#` with the
//! same scoping rules as `let` directives.  This is useful when invoking
//! `expand` in a context where `$` isn't available, such as a `macro_rules!`
//! definition. Any punctuation character
//! can be used in place of the `#` to use that as the interpolation sigil.
//! Notably, all instances of the interpolation sigil will introduce an
//! interpolation regardless of rust syntax rules, so using a common rust
//! character such as `+` would be ill-advised.  Good options include
//! - `~` Rust doesn't use this character at all, but it is a valid token.
//! - `#` This is only used in attributes. (`#[ ]`)
//! - `@` This is only used in pattern bindings (`let x @ some_pattern = ...`)
//! 
//! ### Interpolation
//! Interpolation is invoked by the [interpolation sigil](crate#use-directive), `$` by default.
//! 
//! This documentation will assume that the interpolation sigil is `$`,
//! note that all instances of `$` refer to whatever the interpolation sigil
//! happens to be.
//! 
//! An interpolation sequence beginning with `$` can have one of four forms:
//! - `$name`: This will be substituted for the `name` metavariable.
//! - `${ some_meta_expression }`: This evaluates a [meta expression](crate#meta-expressions).
//! - `$$`: This produces the interpolation sigil itself as a token.
//!
//! All `expand` invocations automatically start with one defined metavariable
//! named `dollar_sign` which evaluates to `$`.  This is not the [interpolation sigil](crate#use-directive),
//! but rather always a `$` character.  This is useful since it allows you to use
//! [`expand`] to create a `macro_rules!` as the output of another `macro_rules!`.
//! Doing this also requires a [`<--use #`](crate#use-directive) or similar directive.
//!
//! ### Meta Expressions
//! A meta expression consists of a sequence of units which will be concatenated
//! together to form a single token.
//! These are the types of unit:
//! - `name`: This will resolve to the metavariable `name`
//! - `(tokens)`: This resolves to the literal `tokens`
//! - `{tokens}`: This evaluates `{tokens}` as an inner meta expression and resolves to the result.
//! - `"name"`: This takes the value of the metavariable `name` and then stringifies it.
//!   You can use the string prefixes `b""` and `c""` to stringify into different kinds of string literals.
//! 
//! If multiple units are present within `{ }`, they will be concatenated to form a single token.
//! If this concatenation is impossible for any reason, it will produce a compiler error.
//! 
//! Additionally, each unit can be suffixed with `.operation_name` to perform operations on them.
//! The available operations are:
//! - `stringify`: Convert a token into a string.  `${variable.stringify}` produces the same results as `${"variable"}`.
//! - `snake_case`: Convert an identifier to `snake_case`.
//! - `upper_camel_case`: Convert an identifier to `UpperCamelCase`.
//! - `screaming_snake_case`: Convert an identifier to `SCREAMING_SNAKE_CASE`.
//! - `camel_case`: Convert an identifier to `camelCase`. This casing is never used in standard rust style.
//! - `upper_snake_case`: Convert an identifier to `Upper_Snake_Case`. This casing is never used in standard rust style.

/// The purpose of the crate.
/// See [crate-level documentation](crate).
pub use expanda_macros::expand;

/// This is a version of [`expand`] which can be used to generate just some
/// attributes on an item, since a normal function-style macro isn't capable
/// of this.
pub use expanda_macros::expand_attr;

/// This allows you to use a token sequence or token sequence list
/// declared by [`declare`] in an [`expand`] invocation.
/// 
/// Do so via
/// ```
/// #[using(foo)]
/// expand! {
///     ...
/// }
/// ```
/// Doing it like this will interpret `foo` as a token sequence and declare it
/// as a metavariable within the [`expand`].  You can write `#[using(*foo)]` to 
/// interpret it as a list and declare it as a metalist instead.
pub use expanda_macros::using;

/// This is an attribute macro version of the [`declare`] macro.
/// It will simply declare the entire item it is placed on under
/// the name passed as its argument.
/// 
/// Example:
/// ```
/// #[declare_item(foo)]
/// enum Foo {
///    A(A),
///    B(B),
///    C(C),
/// }
/// ```
/// Note! This macro evaluates to code that depends on the `expanda` crate
/// being accessible through the path `::expanda`.  It will break if the crate
/// is used under a different name.  This is a current limitation of attribute
/// macros.
pub use expanda_macros::declare_item;

/// Invocations of this macro are emitted by macros generated by
/// [`declare`].  It allows for recursive construction
/// of arbitrary code containing some provided token
/// sequence.
#[macro_export]
macro_rules! fold {
    (
        [$($tokens:tt)*]
        < [$($right:tt)*]
        $($rest:tt)*
    ) => {
        $crate::fold!{[$($tokens)* $($right)*] $($rest)*}
    };
    (
        [$($tokens:tt)*]
        > [$($left:tt)*]
        $($rest:tt)*
    ) => {
        $crate::fold!{[$($left)* $($tokens)* ] $($rest)*}
    };
    (
        [$($tokens:tt)*]
        ()
        $($rest:tt)*
    ) => {
        $crate::fold!{[($($tokens)*)] $($rest)*}
    };
    (
        [$($tokens:tt)*]
        []
        $($rest:tt)*
    ) => {
        $crate::fold!{[[$($tokens)*]] $($rest)*}
    };
    (
        [$($tokens:tt)*]
        {}
        $($rest:tt)*
    ) => {
        $crate::fold!{[{$($tokens)*}] $($rest)*}
    };
    ([$($tokens:tt)*]) => { $($tokens)* };
}

/// This macro allows you to declare a token sequence or list
/// of token sequences for use by other [`expand`] invocations.
/// The syntax is
/// ```
/// declare! {
///     var = (values)
/// }
/// ```
/// Where `values` is any sequence of tokens to be interpreted
/// by [`expand`] like the value for a [`let`](crate#let-directive) directive.
/// 
/// Any number of `var = (values)` can be placed inside a single
/// [`declare`] invocation.
///
/// To use a declaration with [`expand`], use the [`using`] macro.
#[macro_export]
macro_rules! declare {
    ($($tokens:tt)*) => {
        $crate::__declare!{($) $($tokens)*}
    }
}

/// This is an implementation detail of [`declare`].
/// It may be removed in the future and doing so will
/// not be considered a breaking change.
#[doc(hidden)]
#[macro_export]
macro_rules! __declare {
    (($dollar:tt) $(
        $(#[$($attr:tt)*])*
        $v:vis $name:ident = ($($value:tt)*)
    )*
    ) => {$(
        $(#[$($attr)*])*
        macro_rules! $name {
            ($dollar($dollar tokens:tt)*) => {
                $crate::fold!{[$($value)*] $dollar($dollar tokens)*}
            }
        }
        $v use $name;
    )*}
}
