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
    LEPT_PARSE_EXPECT_VALUE,
    LEPT_PARSE_INVALID_VALUE,
    LEPT_PARSE_ROOT_NOT_SINGULAR
}

struct LeptValue {
    tag:LeptType
}
struct LeptContext{
    chars:Vec<char>,
    ptr:usize
}

impl LeptContext {
    fn peek(&self) -> char { self.chars[self.ptr] }
    fn peek1(&self) -> char { self.chars[self.ptr + 1] }
    fn peek2(&self) -> char { self.chars[self.ptr + 2] }
    fn forward(&mut self) {self.ptr += 1;}
    fn expect(&mut self, c: char) {
        if self.peek() != c {
            panic!("Expect {}, but got{}", c, self.peek());
        } else {
            self.forward();
        }
    }
}



fn lept_parse_whitespace(c:& mut LeptContext) {
    while c.peek() == ' ' || c.peek() == '\t' || c.peek() == '\n' || c.peek() == '\r' { c.forward(); }
}

// LeptContext* c, LeptValue* v
fn lept_parse_null(c:& mut LeptContext, v:&mut LeptValue)->Status {
    c.expect('n');
    if c.peek() != 'u' || c.peek1() != 'l' && c.peek2() != 'l' {
        return Status::LEPT_PARSE_INVALID_VALUE;
    }
    c.forward();
    c.forward();
    c.forward();
    v.tag = LeptType::LEPT_NULL;
    return Status::LEPT_PARSE_OK;
}

fn lept_parse_value(c:& mut LeptContext, v:&mut LeptValue)->Status {
    match c.peek() {
        '\n' => lept_parse_null(c, v),
        '\0' => Status::LEPT_PARSE_EXPECT_VALUE,
        _ => Status::LEPT_PARSE_INVALID_VALUE
    }
}

fn lept_get_type(v:& LeptValue) -> LeptType { v.tag } 

fn lept_parse(v:&mut LeptValue, json:&str) -> Status {
    let mut c = LeptContext {chars: json.chars().collect(), ptr: 0};
    v.tag = LeptType::LEPT_NULL;
    lept_parse_whitespace(&mut c);
    return lept_parse_value(&mut c, v);
}

static mut test_pass_counter:i32 = 0;

fn test1() {
    let mut value = LeptValue{tag: LeptType::LEPT_FALSE};
    lept_parse(&mut value, "  null");
    match value.tag {
        LeptType::LEPT_NULL => println!("passed"),
        _ => panic!("err")
    }
    unsafe { test_pass_counter += 1;}
}


fn main() {
    println!("LeptJson Rust version, Start!");
    test1();
    unsafe { println!("{} test case passed!", test_pass_counter); }

}


