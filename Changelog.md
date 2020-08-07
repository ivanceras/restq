# Changelog

## 0.3.0
- Rename `CsvData` to a more appropriate name `StmtData`
- Add PlainData which parses only data with table definition and the csv rows
- modularize `csv_rows` into its own module

## 0.2.1
- Add parser for bulk update, which is just the same with bulk delete
- Remove complex string types such as email, domain, ipaddr since they can be stored as string
- Modularize functions and expose the a function to extract only the restq from a request
