# RestQ
is a compact data format/language suitable for use in a rest api.



**RestQ Syntax:**
```
create  = table,"{", { column_def }, "}", "\n", csv_data
column_def = [column_attributes], column, [ "(" foreign ")" ], ":", data_type, [ "(" default_value ")" ]
column_attributes = "*" | "@" | "&"
data_type = "bool" | "s8" | "s16" | "s32" | "s64" | "u8" | "u16", etc
default_value  = value
value = number | string | bool ,etc
column = string
table = string
foreign = table
insert = table, "{", { column }, "}","\n", csv_data
select = table, [join_table], "{", { column } "}", "?", [condition]
join_table = table, join_type, table
join_type = "->" | "<-" | "-><-" | "<-->"
condition = expr
expr =  column | value | binary_operation
binary_operation = expr, operator, expr
operator = "and" | "or" | "eq" | "gte" | "lte", etc
```

### Column attributes:
- `*` denotes a primary column
- `@` denotes to add the column to the table index
- `&` denotes to add the column to the unique keys

### Join types:
  - left join
```
product<-users
```
 - right join
```
product->users
```
 - inner join
```
product-><-users
```
 - full join
```
product<-->users
```

## Data types:
- `s8`                         : u8 that autoincrements
- `s16`                        : u16 that autoincrements
- `s32`                        : u32 that autoincrements, serial
- `s64`                        : u64 that autoincrements, bigserial
- `f32`                        : float 4 bytes
- `f64`                        : float 8 bytes
- `bool`                       : boolean
- `i8`,`i16`,`i32`,`i64`       : signed integer
- `u8`,`u16`,`u32`,`u64`       : unsigned intergers
- `text`                       : utf8 string
- `uuid`                       : plain uuid, randomly generated when null
- `uui_rand`                   : randomly generated uuid
- `slug`                       : create a new uuid and generate a url friend base64 string out of it.
- `utc`                        : timestamp with time zone in utc,
- `local`                      : date in local timezone
- `url`                        : url types
- `money`                      : money in pg, numeric(10,2)/numeric(19,4) for high precision in pg as alternative, decimal(19,4) in mysql,  string in sqlite, for storing monetary values

## Creating a table and inserting records in one request.
```
PUT /product{*product_id:s32,name:text,created_by(users):u32,created:utc,is_active:bool}

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

```

```sql
INSERT INTO product(product_id, name, created_by, is_active)
VALUES(
    (1,'go pro',1,2019-10-31 11:59:59.872,true)
    (2,'shovel',1,2019-11-01 07:30:00.462,false)
);
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
PATH /product{*product_id,name}
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
GET product<-users{product.*,users.user_name}?product_id=1&is_active=true&created=gt.2019-11-05T08:45:03.432
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
 - [X] INNER JOIN
 - OUTER JOIN
      - [X] LEFT JOIN
      - [X] RIGHT JOIN
      - [X] FULL JOIN


