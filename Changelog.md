# Changelog

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
