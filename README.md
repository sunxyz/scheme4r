# scheme4r

### feature 
- Scheme for rust 
- Impl R7RS
- No keyword can be customized to overwrite

### Basic types
- Numbers
- Booleans
- Pairs and lists 
- Symbols
- Characters
- Strings 
- Vectors 
- Bytevectors
- Procedures
- Records 
- Ports

### more feature
- macro (deinfe-syntax syntax-rules)

### use
- Support cmd and api , can embeddable
**api**
```
use scheme::eval;
let v = eval("(+ 1 2 3)");
println!("{}",v);
```
console
```
6
```
### learn docs
- [r7rs.org](https://small.r7rs.org/)
- [r7rs-overview.pdf](https://small.r7rs.org/attachment/overview.pdf)
- [r7rs.pdf](https://small.r7rs.org/attachment/r7rs.pdf)