# CANYON-SQL
**A full written in `Rust` ORM for `POSTRESQL` based databases.**

`ORM` stands for object-relational mapping, where objects are used to connect the programming language on to the database systems, with the facility to work SQL and object-oriented programming concepts.


## Early stages advice
The library it's still on a `early stage` state. 
Any contrib via `fork` + `PR` it's really apreciated, specially if you like concepts like backend development, relational - mapping, low-level code, performance optimizations and, of course, `RUST`.


# Availiable query operations:
    - Find all
    - Find by ID


## Basic example of usage

- Since the `v 0.3.0`, the unique Rust code requisite in order have access to the database associated functions that will query your database, it's to derive
the `#[derive(Debug, CanyonCRUD, CanyonMapper)]` elements just above your struct, and `CANYON` will take care about creating all the necessary
stuff to correctly map your database results into Rust objects.

- The unique external thing, it's that you will need a pre-constructed database schema that had the same columns that your struct has. The table name it's for now, irrelevant, due to the necessity of pass it as a String on every call.
(Both things will be solved on future releases, specially soon the fact of have to write the table name). 

Warning: Be aware of correctly map your columns with the attributes of your struct.

Take this example:

`my_model.rs` file
```
use canyon_sql::tokio;
use canyon_sql::canyon_macros::*;

#[derive(Debug, CanyonCRUD, CanyonMapper)]
pub struct Foo {
    field: String,
    name: String,
    just_an_i32: i32
}
```

And now, on your main file, just instanciate a new object of your custom type Foo.
You will have availiable (thanks to the `CrudOperations` trait) any crud operation as an associated function of your type.

NOTE: Remember to `await` the result of any trait's method. 
NOTE: For human-readable result, use the `.as_response::<Foo>()` method, where Foo is the type of your struct.

WARNING: You will need an asyncronous runtime in order to use the ORM. This is because the implementation it's based on the `tokio::postgres` library, not in the default one, in order to get an asyncronous client.

By the way, the easiest option availiable it's just add `tokio` to your `Cargo.toml` file, and mark your main function with the `#[tokio::main]` attribute and make it asyncronous, like the example below:
s


`main.rs` file
```
use canyon_sql::canyon_macros::*;


#[tokio::main]
async fn main() {
    
    // The classic **find all** query.
    let all_foo = Foo::find_all("foo_table_name", &[])  // Where "foo_table_name" it's the name of your table on your database

        .await
        .as_response::<Foo>();
    
    println!("BAZ result: {:?}", all_foo);  
    // Iterate over all the Foo elements on the Vec<Foo>, showing the value of its attrs
    for result in all_foo {
        println!("ITEM: field = {:?}, name = {:?}", result.field, result.name);
    }

    
    // The "non-less classic" **find by ID** implementation
    println!("BAZ on find_by_id: {:?}", 
        Foo::find_by_id(foo_table, 1)
            .await
            .as_response::<Foo>()[0]
    );
}
```

Note: For now, on the `find_by_id` associated function, the result has to be accessed by slice the content of the Vec<Foo>, even if only exists one result. 

This limitation it's due to the DatabaseResult<T> struct. This limitation will be solved soon.

After getting your element on index 0, you can access it's properties by use the dot notation.


## Output of the main code
```
> Executing task: C:\Users\Alex\.cargo\bin\cargo.exe run --package tester_canyon_sql --bin tester_canyon_sql --all-features <

   Compiling canyon_sql v0.1.0 (D:\MSi 2020-2021\Code\Rust\CANYON\canyon_sql)
   Compiling tester_canyon_sql v0.1.0 (D:\MSi 2020-2021\Code\Rust\CANYON\tester_canyon_sql)
    Finished dev [unoptimized + debuginfo] target(s) in 3.77s
     Running `target\debug\tester_canyon_sql.exe`

BAZ result: [Foo { field: "field_field", name: "nombre_field_prueba" }, Foo { field: "field_de_Canyon", name: "Canyon_SQL" }]

ITEM: field = "field_field", name = "nombre_field_prueba"
ITEM: field = "field_de_Canyon", name = "Canyon_SQL"     

BAZ on find_by_id: Foo { field: "field_field", name: "nombre_field_prueba" }
```
