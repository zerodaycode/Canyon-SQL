# Canyon Examples
**Usage examples for a program written with Canyon**

We will use the application **Triforce** to serve as a real example of Canyon's usage.



## Set up
First of all you need a installation of `PostgresSQL` running
on your computer. Go to the official Postgres page and download the lastest
version available for your operating system.

## SQL example script
In the root of this crate you will find an SQL script that will create a Postgres user with the correct permissions over the databases and the schemas, the database, the tables and the relations needed for providing you a complete
working example of `how to use Canyon`.

One thing to take in consideration it's the `Canyon's full mode`.
In this particular mode, Canyon will only works with the Rust structs 
annotated with `#[canyon_macros::canyon_entity]`. This is the preferable way to work with Canyon, cause it will take care about handling everything for you 
behind the scenes, including the create, drop and alter operations over your tables based on the source code written in Rust. More info on the official documentation - [ Not written yet :') ]

The point on this it's that you will encounter on the SQL example script
commented the part of the create the tables and the relations. 
This will be automatically perfomed by Canyon on the first run of your program, so you should only run the SQL script example to have a shortcircuit to create the database and the user. 

### secrets.toml
To tell Canyon about what credentials you want to use and to what database you want to work with, you **MUST** create a `secrets.toml` file on the root of this crate, following the next example:

```
username = zdc
password = your_password_for_the_user
db_name = triforce
```

### Postgres user
You need an user in order to work with the Postgres server. You can choose to use the default's Postgres user `postgres` to quickly set up an environment. This it's perfectly fine, due that we are not working in production, and we reduce development time. 
But if you feel like you want a complet `production-ready` example, just use the whole provided SQL example script.

### Database and tables
The SQL example script also will creates for you the database `triforce` and will create the tables and relations between them for you. 


