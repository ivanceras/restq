# Changelog

# 0.6.1
- Remove the dependency to `js-sys` since we can use `Utc::now` from `chrono` using the `wasmbind` feature.

# 0.6.0
 - Upgrade base64 0.8 -> 0.13
 - **breaking** Now uses base64::URL_SAFE for encoding/decoding bytes value.
 - move list_fail into utils
 - **breaking** Remove the alternate syntax for columns projection using parenthesis, instead just use braces
 - **breaking** Remove the alternate syntax for join type using caret for the arrow
 - Add multi_values parser for expression

# 0.5.0
 - Rename `Table` to `TableName` and `Column` to `ColumnName` to signify that it only contains the name and not their definition
 - Improve multi-statement to ignore multiple whitespace and empty new lines

# 0.4.0
- Include the serial type in conversion to primitive types
- Add support for multi_statement restq
- modify the serial type to use custom serial type in the generated create sql statement
- expose parse_header from stmt_data parser
- Implement all variants of Statement
- add helper function for column_def determing the generated default property
- Implement creating a date now() in wasm using js date interface
- Fix edge case conversion of dates when it is a blank string
- Modify DataTypeDef default value to accomodate also Function as default is not limited to DataValue, it can also be functions
- Add Implement Default for Direction
- parse bulk_update
- Add into_data_values, casting the simple value into a more specific data_type
- Remove the column_def in csv_rows, plain_data and stmt_data since casting is not needed
- Use the simple Value in CsvRows and StmtData, so as not to keep casting the values to and fro in the higher level usage
- Implement conversion of BulkUpdate
- Implement bulk update
- implement conversion of Ilike operator

## 0.3.3
- Add helper methods to Select AST
- Implement fmt::Display for Select AST
- Improve implementation of data conversion
- Fix conversion of serial types

## 0.3.2
- Fix issue: query only has table and paging, the paging was incorrectly parsed as filter.
- Add parsing for common date format
- reexport uuid
- implement `fmt::Display` for TableDef, DataValue, Table, ColumnDef, Column, DataTypeDef etc ColumnAttribte
- Add Bytes and Json DataType
- expose PlainData in the crate

## 0.3.1
 - Add an expose `parse_select_chars` for usage in `inquerest` crate

## 0.3.0
- Rename `CsvData` to a more appropriate name `StmtData`
- Add PlainData which parses only data with table definition and the csv rows
- modularize `csv_rows` into its own module

## 0.2.1
- Add parser for bulk update, which is just the same with bulk delete
- Remove complex string types such as email, domain, ipaddr since they can be stored as string
- Modularize functions and expose the a function to extract only the restq from a request
