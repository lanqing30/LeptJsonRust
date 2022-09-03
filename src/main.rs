
// Section 1: basic headers
use std::cell::RefCell;

#[derive(Copy, Clone)]
enum LeptType { 
    LEPT_NULL, 
    LEPT_FALSE, 
    LEPT_TRUE, 
    LEPT_NUMBER, 
    LEPT_STRING, 
    LEPT_ARRAY, 
    LEPT_OBJECT 
}

enum Status {
    LEPT_PARSE_OK,
    LEFT_PARSE_ERR
    // LEPT_PARSE_EXPECT_VALUE,
    // LEPT_PARSE_INVALID_VALUE,
    // LEPT_PARSE_ROOT_NOT_SINGULAR,
    // LEPT_PARSE_MISS_QUOTATION_MARK,
    // LEPT_PARSE_INVALID_STRING_ESCAPE,
    // LEPT_PARSE_INVALID_STRING_CHAR,
    // LEPT_PARSE_INVALID_UNICODE_SURROGATE,
    // LEPT_PARSE_MISS_COMMA_OR_SQUARE_BRACKET,
    // LEPT_PARSE_MISS_KEY,
    // LEPT_PARSE_MISS_COLON
}

use std::io::LineWriter;
use std::ptr::NonNull;
use std::rc::{Rc};
use std::vec;
struct LeptNode {
    key: String,
    val: LeptValue,
    next: Link,
}
type Link = Option<Rc<RefCell<LeptNode>>>;
struct LeptValue {
    n: f64,
    a: Vec<LeptValue>,
    str: String,
    o: Link,
    // using vector to mimic array 
    // using links list to mimic the key-val pairs
    tag : LeptType
}


impl Default for LeptValue {
    fn default() -> LeptValue {
        LeptValue {
            tag: LeptType::LEPT_FALSE,
            n: 0.0,
            a: vec![],
            str: String::new(),
            o: None
        }
    }
}

// Section 2: parse the string to leptvalue.

struct LeptContext{
    chars:Vec<char>,
    ptr:usize,
    stack:Vec<char>
}

impl LeptContext {
    fn hasnext(&self) -> bool {self.ptr < self.chars.len()}
    fn peek(&self) -> char { self.chars[self.ptr] }
    fn peek1(&self) -> char { self.chars[self.ptr + 1] }
    fn peek2(&self) -> char { self.chars[self.ptr + 2] }
    fn peek3(&self) -> char {self.chars[self.ptr + 3]}
    fn forward(&mut self) {self.ptr += 1;}
    fn expect(&mut self, c: char) {
        if self.peek() != c {
            panic!("Expect {}, but got{}", c, self.peek());
        } else {
            self.forward();
        }
    }
    fn putc(&mut self, c: char) {
        self.stack.push(c);
    }

}

fn ISDIGIT(c: char) -> bool {
    if (c == '1' || c == '0' || c == '2' || c == '3' || 
        c == '4' || c == '5' || c == '6' || c == '7' || 
        c == '8' || c == '9') {
        return true;
    }
    else {
        return false;
    }
}

fn lept_parse_whitespace(c:& mut LeptContext)->Status {
    while c.hasnext() && (c.peek() == ' ' || c.peek() == '\t' || c.peek() == '\n' || c.peek() == '\r') { c.forward(); }
    Status::LEPT_PARSE_OK
}

fn lept_parse_string(c:& mut LeptContext, v:&mut LeptValue)->Status {
    c.expect('\"');
    let mut buf:String = String::new();
    while (c.peek() != '\"') {
        buf.push(c.peek());
        c.forward();
    }
    c.expect('\"');
    v.str = buf;
    v.tag = LeptType::LEPT_STRING;
    return Status::LEPT_PARSE_OK;
}

// LeptContext* c, LeptValue* v
fn lept_parse_null(c:& mut LeptContext, v:&mut LeptValue)->Status {
    c.expect('n');
    if c.peek() != 'u' || c.peek1() != 'l' && c.peek2() != 'l' {
        return Status::LEFT_PARSE_ERR;
    }
    c.forward();
    c.forward();
    c.forward();
    v.tag = LeptType::LEPT_NULL;
    return Status::LEPT_PARSE_OK;
}

fn lept_parse_array(c:& mut LeptContext, v:&mut LeptValue) -> Status {
    c.expect('[');
    while true {
        let mut buf = LeptValue { ..Default::default() };
        lept_parse_value(c, &mut buf);
        v.a.push(buf);
        match c.peek() {
            ',' => c.forward(),
            ']' => break,
            _ => return Status::LEFT_PARSE_ERR
        }
    }
    c.expect(']');
    v.tag = LeptType::LEPT_ARRAY;
    return Status::LEPT_PARSE_OK;
}


fn lept_parse_object(c:& mut LeptContext, v:&mut LeptValue) -> Status {

    c.expect('{');
    lept_parse_whitespace(c);
    if (c.peek() == '}') {
        // just empty object
        c.forward();
        v.tag = LeptType::LEPT_OBJECT;
        return Status::LEPT_PARSE_OK;
    }
    
    while true {
        // 1.parse string using api.
        let mut tmp_str = LeptValue {..Default::default()};
        let mut tmp_val = LeptValue {..Default::default()};
        lept_parse_value(c, & mut tmp_str);
        c.expect(':');
        // 2.lept_parse_value
        lept_parse_value(c, & mut tmp_val);
        // 3. put the above infos into results
        let new_node = Rc::new(RefCell::new(LeptNode {
            key: tmp_str.str,
            val: tmp_val,
            next: v.o.take()
        }));
        v.o = Some(new_node);
        if (c.peek() == '}') {
            c.forward();
            break;
        } else if (c.peek() == ',') {
            c.forward();
        }
    }
    v.tag = LeptType::LEPT_OBJECT;
    return Status::LEPT_PARSE_OK;
}


// LeptContext* c, LeptValue* v
fn lept_parse_true(c:& mut LeptContext, v:&mut LeptValue)->Status {
    c.expect('t');
    if c.peek() != 'r' || c.peek1() != 'u' && c.peek2() != 'e' {
        return Status::LEFT_PARSE_ERR;
    }
    c.forward();
    c.forward();
    c.forward();
    v.tag = LeptType::LEPT_TRUE;
    return Status::LEPT_PARSE_OK;
}

// LeptContext* c, LeptValue* v
fn lept_parse_false(c:& mut LeptContext, v:&mut LeptValue)->Status {
    c.expect('f');
    if c.peek() != 'a' || c.peek1() != 'l' && c.peek2() != 's' && c.peek3() != 'e' {
        return Status::LEFT_PARSE_ERR;
    }
    c.forward();
    c.forward();
    c.forward();
    c.forward();
    v.tag = LeptType::LEPT_FALSE;
    return Status::LEPT_PARSE_OK;
}

fn lept_parse_literal(c:& mut LeptContext, v:&mut LeptValue, literal:&str, tag:LeptType) -> Status {
    let literal_chars:Vec<char> = literal.chars().collect();
    c.expect(literal_chars[0]);

    for i in 1..literal_chars.len() {
        if c.peek() == literal_chars[i] {
            c.forward();
        } else {
            return Status::LEFT_PARSE_ERR;
        }
    }
    // TODO: Is it ok if we do not backward
    v.tag = tag;
    Status::LEPT_PARSE_OK
}

// TODO: currently only support positve integers and no leading zeros.
fn lept_parse_number(c:& mut LeptContext, v:&mut LeptValue) -> Status{
    let mut res:f64 = 0.0;
    while ISDIGIT(c.peek()) {
        let x = c.peek();
        res = res * 10.0 + (x as u32 as f64) - ('0' as u32 as f64);
        c.forward();
    }
    v.n = res;
    v.tag = LeptType::LEPT_NUMBER;
    return Status::LEPT_PARSE_OK;
}

fn lept_parse_value(c:& mut LeptContext, v:&mut LeptValue)->Status {
    lept_parse_whitespace(c);
    match c.peek() {
        'n' => lept_parse_null(c, v),
        'f' => lept_parse_false(c, v),
        't' => lept_parse_true(c, v),
        '\"' => lept_parse_string(c, v),
        '[' => lept_parse_array(c, v),
        '{' => lept_parse_object(c, v),
        _ => lept_parse_number(c, v)
    };

    return lept_parse_whitespace(c);
}

fn lept_parse(v:&mut LeptValue, json:&str) -> Status {
    let mut c = LeptContext {chars: json.chars().collect(), ptr: 0, stack:vec![]};
    v.tag = LeptType::LEPT_NULL;
    return lept_parse_value(&mut c, v);
}

// Section 3: Convert To String API

fn lept_array_stringfy(v:&LeptValue) -> String {
    let dummy = match v.tag {
        LeptType::LEPT_ARRAY => {},
        _ => panic!("NOT ARRY")
    };
    let mut vector:Vec<String> = Vec::new();
    let mut header = String::from("[");
    let mut footer = String::from("]");
    let sz = v.a.len();
    for i in 0..sz {
        let elem = &v.a[i];
        vector.push(lept_stringfy(elem));
    }
    let mut middle = vector.join(",");
    return [header, middle, footer].join("");
}

fn lept_object_stringfy(v:&LeptValue) -> String {
    let dummy = match v.tag {
        LeptType::LEPT_OBJECT => {},
        _ => panic!("NOT object")
    };
    let mut vector:Vec<String> = Vec::new();
    let header = String::from("{");
    let fotter = String::from("}");

    // we have to iterative 
    let mut head = match v.o {
        None => None,
        Some(ref n) => Some(Rc::clone(n)),
    };

    while true {
        let node = match head {
            None => break,
            Some(ref n) => Rc::clone(n), // Clone the Rc
        };
        // do something on node.
        let key = &node.borrow().key;
        let val = &node.borrow().val;
        let pair = format!("\"{}\":{}", key, lept_stringfy(val));
        vector.push(pair);
        head = match node.borrow().next {
            None => None,
            Some(ref next) => Some(Rc::clone(next)), //clone the Rc
        };
    }
    
    let middle:String = vector.join(",");
    return [header, middle, fotter].join("");

}



fn lept_stringfy(v:& LeptValue) -> String {
    match v.tag {
        LeptType::LEPT_ARRAY => lept_array_stringfy(v),
        LeptType::LEPT_FALSE => String::from("false"),
        LeptType::LEPT_NULL => String::from("null"),
        LeptType::LEPT_NUMBER => v.n.to_string(),
        LeptType::LEPT_OBJECT => lept_object_stringfy(v),
        LeptType::LEPT_TRUE => String::from("true"),
        LeptType::LEPT_STRING => String::from("string")
    }
}

// Section 4: Get Type API

// Section 5: Array Designed API

// Section 5: Object Designed API => core of this implementations.

// insert(k, v), assume it is never in the array, just entail this
fn InsertOrUpdate(v:&mut LeptValue, key: String, val:&LeptValue) {
    // check it is a object.
    let dummy = match v.tag {
        LeptType::LEPT_OBJECT => {},
        _ => panic!("NOT object")
    };

    // 

}

// remove(k)


// Unit Test.

static mut test_pass_counter:i32 = 0;
fn test1() {
    let mut value = LeptValue { ..Default::default() };
    lept_parse(&mut value, "  null");
    match value.tag {
        LeptType::LEPT_NULL => println!("passed"),
        _ => panic!("err")
    }
    unsafe { test_pass_counter += 1;}
}

fn test2() {
    let mut value = LeptValue { ..Default::default() };
    lept_parse(&mut value, "  false");
    match value.tag {
        LeptType::LEPT_FALSE => println!("passed"),
        _ => panic!("err")
    }
    unsafe { test_pass_counter += 1;}
}

fn test3() {
    let mut value = LeptValue { ..Default::default() };
    lept_parse(&mut value, "  01234   ");
    match value.tag {
        LeptType::LEPT_NUMBER => println!("passed"),
        _ => panic!("err")
    }
    unsafe { test_pass_counter += 1;}
}

fn test4() {
    let mut value = LeptValue { ..Default::default() };
    lept_parse(&mut value, " \" shit  \" ");
    match value.tag {
        LeptType::LEPT_STRING => println!("passed"),
        _ => panic!("err")
    }
    unsafe { test_pass_counter += 1;}
}

fn test5() {
    let mut value = LeptValue { ..Default::default() };
    lept_parse(&mut value, "  [1231, [null, false], 344] ");
    match value.tag {
        LeptType::LEPT_ARRAY => println!("passed"),
        _ => panic!("err")
    }

    println!("{}", lept_array_stringfy(&value));

    unsafe { test_pass_counter += 1;}
}

fn test6() {
    let mut value = LeptValue { ..Default::default() };
    lept_parse(&mut value, " { \"123\" : [ {\" dick \" : 1234 }] } ");
    match value.tag {
        LeptType::LEPT_OBJECT => println!("passed"),
        _ => panic!("err")
    }
    println!("{}", lept_object_stringfy(&value));
    unsafe { test_pass_counter += 1;}
}



fn main() {
    println!("LeptJson Rust version, Start!");
    test1();
    test2();
    test3();
    test4();
    test5();
    test6();
    unsafe { println!("{} test case passed!", test_pass_counter); }
}



// interface: 
// parse

/*
parse_from_file
parse_from_string

LeptValue* v = parse_from_xxx();
leptvalue convert to string (make it beautify)


v->get type

if type == array:
    vec[value*] = get_all_childeren()
    get(i)
    set(i, v*)
    append(value)
    remove(i)
    

if type == obj:
    vec[value*] = get_all_key()
    v* = get(key)
    contains(key)
    insert(key: v*)
    set(key, v*)
    remove(key)

others:
    not opened.

*/