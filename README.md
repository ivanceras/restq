# RestQ

[![Latest Version](https://img.shields.io/crates/v/restq.svg)](https://crates.io/crates/restq)

The simplest way to express data operations in a rest API.

Implemented using the combination
of appropriate HTTP methods, url and csv for the data format.


Example:
Querying a simple table `person` with filtering, grouping and paging.

```
GET /person?age=lt.42&(student=eq.true|gender=eq.'M')&group_by=sum(age),grade,gender&having=min(age)=gt.42&order_by=age.desc,height.asc&page=20&page_size=100
```

This can then be converted into a SQL query.
```sql
SELECT * FROM person
WHERE age < 42
    AND (student = true OR gender = 'M')
GROUP BY sum(age), grade, gender
HAVING min(age) > 42
ORDER BY age DESC, height ASC
LIMIT 100 OFFSET 1900 ROWS
```
The response body will contain a `csv` formatted data of the results from the query.


**RestQ Syntax/Grammar:**
```
create  = ["CREATE" | "PUT"], "/",  table, column_def_list, "\n", csv

select = "GET", "/", table, [join_table], column_list, [ "?", condition]

delete = "DELETE", table, [ "?", condition ]

update = ["UPDATE | "PATCH"] table, set_expr_list, [ "?", condition]

drop = ["DROP" | "DELETE"] "-", table

alter = ["ALTER" | "PATCH"] table, { drop_column | add_column | alter_column }

drop_column = "-", column

add_column = "+", column_def

alter_column = column, "=", column_def


column_def_list =  "{", { column_def }, "}"
        | "(", { column_def }, ")"

column_def = [ { column_attributes } ], column, [ "(" foreign ")" ], ":", data_type, [ "(" default_value ")" ]

column_attributes = primary | index | unique

primary = "*"

index = "@"

unique = "&"

data_type = "bool" | "s8" | "s16" | "s32" | "s64" | "u8" | "u16", etc

default_value  = value

value = number | string | bool ,..etc

column = string, ".", string
        | string

table = string

foreign = table

insert = table, column_list ,"\n", csv

column_list = "{", { column }, "}"


join_table = table, join_type, table

join_type = right_join | left_join | inner_join | full_join

right_join = "->"

left_join = "<-"

inner_join = "-><-"

full_join = "<-->"

condition = expr

expr =  column | value | binary_operation

binary_operation = expr, operator, expr

operator = "and" | "or" | "eq" | "gte" | "lte" ,..etc
```


## Data types
- `bool`                            : boolean
- `s8`                              : u8 that autoincrements
- `s16`                             : u16 that autoincrements
- `s32`                             : u32 that autoincrements, serial
- `s64`                             : u64 that autoincrements, bigserial
- `f32`                             : float 4 bytes
- `f64`                             : float 8 bytes
- `i8`,`i16`,`i32`,`i64`            : signed integer
- `u8`,`u16`,`u32`,`u64`            : unsigned intergers
- `text`                            : utf8 string
- `uuid`                            : plain uuid, randomly generated when null
- `uuid_rand`                       : randomly generated uuid
- `uuid_slug`                       : create a new uuid and generate a url friend base64 string out of it.
- `utc`                             : timestamp with time zone in utc,
- `local`                           : date in local timezone
- `url`                             : url types
- `json`                            : json
- `bytes`                           : binary data

## Creating a table and inserting records in one request.
```
PUT /+product{*product_id:s32,name:text,created_by(users):u32,created:utc,is_active:bool}
Content-Type: text/csv; charset=UTF-8

1,go pro,1,2019-10-31 11:59:59.872,,true
2,shovel,1,2019-11-01 07:30:00.462,,false
```

 - http method is `PUT`
 - url is a `restq` syntax.
 - body is `csv`

**The equivalent SQL:**
```sql
CREATE TABLE product (
 product_id serial NOT NULL PRIMARY,
 name character varying NOT NULL,
 created_by integer NOT NULL REFERENCES users(user_id),
 created timestamp with time zone NOT NULL DEFAULT now(),
 is_active boolean NOT NULL

INSERT INTO product(product_id, name, created_by, is_active)
VALUES(
    (1,'go pro',1,2019-10-31 11:59:59.872,DEFAULT,true)
    (2,'shovel',1,2019-11-01 07:30:00.462,DEFAULT,false)
);
```

## Show the table definition
```
HEAD /product

```

## Show all tables
```
HEAD /
```

## Querying the records

```
GET /product{product_id,name}?is_active=eq.true&order_by=created.desc
```

```sql
SELECT product_id,name FROM product WHERE is_active = true ORDER BY created DESC
```

## Inserting records
```
POST /product{product_id,name,created_by,created,is_active}
1,go pro,1,2019-10-31 11:59:59.872,,true
2,shovel,1,2019-11-01 07:30:00.462,,false
```

```sql
INSERT INTO product(product_id, name, created_by, is_active)
VALUES(
    (1,'go pro',1,2019-10-31 11:59:59.872,true)
    (2,'shovel',1,2019-11-01 07:30:00.462,false)
);
```

## Insert with query
```
POST /user{user_id,name,person_id(GET/person{id}?person.name=name)}
1,TOM JONES,,
```
```sql
INSERT INTO user(user_id, name, person_id)
VALUES(1, 'TOM JONES', (SELECT person.id FROM person WHERE person.name='TOM JONES'));
```

## Updating records

```
PATCH /product{description="I'm the new description now"}?product_id=1
```
```sql
UPDATE product SET description = 'I\'m the new description now' WHERE product_id = 1;
```

## Bulk updating records

2 versions of the same record is passed, first is the original, the next is the updated one
```
PATCH /product{*product_id,name}
1,go pro,1,go pro hero4
2,shovel,2,slightly used shovel
```
```sql
UPDATE product SET name = 'go pro hero4' WHERE id = 1;
UPDATE product SET name = 'slightly used shovel' WHERE id = 2'
```

## Delete

```
DELETE /product?product_id=1
```

```sql
DELETE FROM product WHERE product_id = '1'
```

## Delete multiple
```
DELETE /product{product_id}
1
2
3
```

```sql
DELETE FROM product WHERE product_id IN ('1','2','3')
```

## Delete multiple, by name(no primary keys).
```
DELETE /product{name,is_active}
Go Pro,true
Shovel,true
Chair,true
```
```sql
DELETE FROM product WHERE name = 'Go Pro' AND is_active = 'true';
DELETE FROM product WHERE name = 'Shovel' AND is_active = 'true';
DELETE FROM product WHERE name  = 'Chair' AND is_active = 'true';
```

## Delete all records of a table
```
DELETE /product
```

```sql
TRUNCATE product;
```

## Complex select (with joins)

```restq
GET /product<-users{product.*,users.user_name}?product_id=1&is_active=true&created=gt.2019-11-05T08:45:03.432
```

```sql
SELECT product.*, users.user_name
    FROM product
        LEFT JOIN users ON product.created_by = users.user_id
    WHERE product_id = 1
        AND is_active = true
        AND created > '2019-11-05T08:45:03.432'
```


## Join tables

 ### Supported join types
 - [X] INNER JOIN  `table1-><-table2`
 - OUTER JOIN
      - [X] LEFT JOIN  `table1<-table2`
      - [X] RIGHT JOIN  `table1->table2`
      - [X] FULL JOIN `table1<-->table2`

## Prior crate and inspiration
 - [inquerest](https://github.com/ivanceras/inquerest), in the works of porting to call this library.
 - [postgrest](https://github.com/PostgREST/postgrest), restq differs syntax to postgrest, with focus on intuitive filter clause

#### Please support this project:
 [![Become a patron](https://c5.patreon.com/external/logo/become_a_patron_button.png)](https://www.patreon.com/ivanceras)
