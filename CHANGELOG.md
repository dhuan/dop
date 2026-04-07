# Changelog

## Unreleased yet

* FIXED
  * Avoid panic when trying to use unsupported type NULL with TOML
  * Avoid panic when trying to read oversized array index 

## v0.2.0

* ADDED
  * --begin: allows defining Lua variables before processing starts
  * --print-var: Replace default output printing a Lua variable instead.

## v0.1.1

* FIXED
  * "Execute once" does not work when input is an empty object
  * Querying with -q while passing an invalid query string panics
  * JSON with number keys are discarded
  * Attempting to retrieve out of range item from array crashes
  * set/unset(...) with invalid key/path crashes
