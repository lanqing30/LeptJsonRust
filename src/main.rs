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

use std::ptr::NonNull;
use std::rc::{Rc};
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



impl LeptValue {
    // https://stackoverflow.com/questions/55331919/borrowed-refcell-does-not-last-long-enough-when-iterating-over-a-list
    fn GetNode(&self, key:String) -> Link {
        let mut p = match self.o {
            None => None,
            Some(ref n) => Some(Rc::clone(n)),
        };

        loop {
            let node = match p {
                None => break,
                Some(ref n) => Rc::clone(n), // Clone the Rc
            };

            if (node.borrow().key == key) {return Some(node);}
            p = match node.borrow().next {
                None => None,
                Some(ref next) => Some(Rc::clone(next)), //clone the Rc
            };
        }

        return None;
    }
}


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
    unsafe { test_pass_counter += 1;}
}

fn test6() {
    let mut value = LeptValue { ..Default::default() };
    lept_parse(&mut value, " { \"123\" : [ {\" dick \" : 1234 }] } ");
    match value.tag {
        LeptType::LEPT_OBJECT => println!("passed"),
        _ => panic!("err")
    }
    unsafe { test_pass_counter += 1;}
}


fn test7() {
    let mut value = LeptValue { ..Default::default() };
    lept_parse(&mut value, " { \"123\" : \" bullshit \" } ");
    match value.tag {
        LeptType::LEPT_OBJECT => println!("passed"),
        _ => panic!("err")
    }

    let fetch = value.GetNode(String::from("123"));
    match fetch {
        None => println!("None detected"),
        Some(ref n) => {
            println!("Some value");
            // we will get the content of the Node, we know it is a string
            // let content = &n.borrow().val;
            // println!("content is {}", content.str);
            n.borrow_mut().val.str = String::from("234");
            let content = &n.borrow().val;
            println!("content is {}", content.str);
        }
    }
    unsafe { test_pass_counter += 1;}
}

// convert this to String
fn Stringfy(v:&LeptValue) {

}

// operate key[val]
// operator array[index]





fn main() {
    println!("LeptJson Rust version, Start!");
    test1();
    test2();
    test3();
    test4();
    test5();
    test6();
    test7();
    unsafe { println!("{} test case passed!", test_pass_counter); }
}
