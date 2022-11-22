use crate::{types::Type, eval};

#[test]
fn test_u8vector() {
    let t = eval("#u8(1 3 6 7 8 0)").unwrap();
    println!("{}",t);
    if let Type::ByteVectors(v) = t{
        assert_eq!(v.len(), 6);
    }
   
}

#[test]
fn test_u8vector2() {
    let t = eval("(#u8(1 3 6 7 8 0))").unwrap();
    println!("{}",t);
    if let Type::ByteVectors(v) = t{
        assert_eq!(v.len(), 6);
    }
   
}