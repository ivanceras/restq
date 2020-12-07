# TODO
- [X] Convert restq to SQL
    - [ ] with the records as arguments to a prepared statement (parameterized query)
        - [ ] Blocked by, sqlparser don't support parameterize query yet.
- [ ] Implement Update statement parser
- [X] Implement Drop statement parser
- [ ] Implement Alter statment parser
-     - [X] Implement drop_column parser
-     - [X] Implement add_column parser
-     - [X] Implement rename column parser
- [ ] Support for money type using Bigdecimal crate
- [X] Publish to crate.io
        ## Blocked by:
        - [X] sqlparser-rs  (Problem: slow release cycle, busy main dev)
              - [X] -> Solution: release a fork crate name: sql-ast
        - [X] pom (expose method field in Parser)
              - [X] -> Solution: release a fork
- [ ] Implement `fmt::Display` on `Statement` AST such as `Select`
- [X] Rename `Table` to `TableName` and `Column` to `ColumnName`
- [X] Remove the user-friendly parsing of join such as `^-` and just use the arrows `<-`
    The strict version works find in the url and just use `percent_decode` to get the original url text
- [X] Remove the parenthesis grouping of columns and just use braces.

