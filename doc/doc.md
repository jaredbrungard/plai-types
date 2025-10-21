Type checker
============

Write a type checker as the function `tc` in `interp.rs` with the
following type signature:

    pub fn tc(e: &Exp, tnv: &TEnv) -> Result<Type, String>

`TEnv` and `Type` are defined in `main.rs` along with all other code
interpreter types.

The structure of `tc` should be very similar overall to that of
`interp` as the book and lecture notes showed, but you should be
returning a `Type` for each expression checked. Follow the typing
rules outlined in the book.

Note that we are omitting recursive types for now.

Try to figure out what is required by looking at the existing code,
the book, and the test cases. If you are still unsure of how
something is supposed to work, please ask for help.
