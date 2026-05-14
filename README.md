# scheme4r

### feature 
- Scheme for rust 
- Impl R7RS
- No keyword can be customized to overwrite
- Proper tail-recursive evaluation for core tail positions

### Basic types
- Numbers
- Booleans
- Pairs and lists 
- Symbols
- Characters
- Strings 
- Vectors 
- Bytevectors
- Dicts
- Procedures
- Records 
- Ports

### more feature
- macro (deinfe-syntax syntax-rules)
- Tail-safe control flow through `apply`, `call-with-values`, `eval`, `call/cc`, `dynamic-wind`, and `with-exception-handler`

### use
- Support cmd and api , can embeddable
**api**
```
use scheme4r::Scheme;

let scheme = Scheme::standard();
let v = scheme.eval("(+ 1 2 3)")?;
println!("{}",v);
```
console
```
6
```

shortcut
```
use scheme4r::eval;

let v = eval("(+ 1 2 3)")?;
println!("{}", v);
```

host function
```
use std::collections::HashMap;

use scheme4r::{BuiltinFn, Scheme, SchemeError, Value, eval::Engine};

fn double(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    match args {
        [Value::Number(v)] => Ok(Value::Number(v * 2)),
        _ => unreachable!(),
    }
}

let mut builtins = HashMap::new();
builtins.insert("double".to_string(), double as BuiltinFn);

let scheme = Scheme::new(builtins);
let v = scheme.eval("(double 21)")?;
println!("{}", v);
```
### learn docs
- [r7rs.org](https://small.r7rs.org/)
- [r7rs-overview.pdf](https://small.r7rs.org/attachment/overview.pdf)
- [r7rs.pdf](https://small.r7rs.org/attachment/r7rs.pdf)
