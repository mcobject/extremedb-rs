# extremedb_sys

Low-level FFI bindings to McObject *e*X*treme*DB for Rust.

This package contains the low-level, unsafe Rust declarations of the 
*e*X*treme*DB C API functions, as well as structures and enumerations 
matching their C counterparts. Most applications will never call the functions
declared in this package directly.

A few environment variables must be set prior to building this package. For the
details, refer to the documentation.

Notice that an existing installation of *e*X*treme*DB 8.2 is required to build 
this package. An evaluation *e*X*treme*DB package can be requested at the
[McObject website](https://www.mcobject.com).
