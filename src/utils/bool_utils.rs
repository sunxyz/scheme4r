use crate::types::{Type::{self,*}, refs::RefOps};



pub fn is_true(v: &Type) -> bool {
    let v = v.clone();
    match v {
        Booleans(b) => {
            return b;
        }
        Numbers(n) => {
            return n != 0;
        }
        Strings(s) => {
            let s:&str = &s.ref_read();
            if s != "" || s != "nil" || s != "0" || s == "#t" {
                return true;
            }
        }
        Characters(c) => {
            if c != '\0' {
                return true;
            }
        }
        Nil => {}
        _ => {
            return true;
        }
    }
    return false;
}
