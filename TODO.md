# TODO
- [X] Convert restq to SQL
    - [ ]with the records as arguments to a prepared statement (parameterized query)
- [ ] Implement Update statement parser
- [ ] Implement Drop statement parser
- [ ] Implement Alter statment parser
-     - [ ] Implement drop_column parser
-     - [ ] Implement add_column parser
-     - [ ] Implement rename column parser
- [ ] Support for money type using Bigdecimal crate
- [ ] Rename restq => restq-core and restq-http => restq
- [ ] Publish to crate.io
        ## Blocked by:
        - [ ] sqlparser-rs
        - [ ] pom (expose method field in Parser)
