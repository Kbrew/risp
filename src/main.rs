#![allow(dead_code)]

enum SExp {
	Cons(Box<SExp>, Box<SExp>),
	Nil,
	Symbol(String),
	String(String),
	Integer(isize),
}


#[derive(Debug)]
enum ReadError{
	EarlyEOF{ loc : FileLocation,	msg : String },
	WrongChar{ loc : FileLocation,	msg : String },
	ParenMismatch{ loc : FileLocation,	msg : String },
	NotImplemented,
}

#[derive(Debug, Clone)]
struct FileLocation{
	file : String,
	line : usize,
	col : usize,
}

struct SExpParser<'a>{
	loc : FileLocation,
	iter : &'a mut Iterator<Item=char>,
	next_char : Option<char>,
}

impl<'a> SExpParser<'a> {
	fn eat_white_space(& mut self) {
		while let Some(c) = self.peek(){
			if c.is_whitespace() {
				self.advance();
			} else {
				break;
			}
		}
	}
	
	fn read_sexp(& mut self) -> Result<SExp, ReadError> {
		let c = try!(self.peek().ok_or(self.error_eof()));
		
		if c.is_open_paren() {
			self.read_list()
		} else if c == '-' || c.is_digit(10) {
			self.read_number()
		} else if c == '"' {
			self.read_string()
		} else if !c.is_whitespace() {
			self.read_symbol()
		} else {
			Err(self.error_wrong_char(c, "Any"))
		}
	}

	fn read_list(& mut self) -> Result<SExp, ReadError> {
		let c1 = try!(self.peek().ok_or(self.error_eof()));

		if c1.is_open_paren() {
			self.advance();
		} else {
			return Err(self.error_wrong_char(c1, "({[" ));
		}
		
		let items = try!(self.read_list_items());
		
		let c2 = try!(self.peek().ok_or(self.error_eof()));
		
		if c1.is_matching_paren(c2) {
			return Ok(items);
		} else {
			return Err(self.error_paren_mismatch(c1, c2));
		}
	}
	
	fn read_list_items(& mut self) -> Result<SExp, ReadError> {
		self.eat_white_space();
		
		let c = try!(self.peek().ok_or(self.error_eof()));
		
		if c.is_close_paren() {
			Ok(SExp::Nil)
		} else {
			let head = try!(self.read_sexp());
			let tail = try!(self.read_list_items());
			
			Ok(SExp::Cons(Box::new(head), Box::new(tail)))
		}
	}
	
	fn read_symbol(& mut self) -> Result<SExp, ReadError> {
		let mut sym_string = String::new();
		
		while let Some(c) = self.peek() {
			if c.is_delimiter() {
				break;
			}
			sym_string.push(c);
			self.advance();
		}
		
		Ok(SExp::Symbol(sym_string))
	}
	
	fn read_escaped_string_char(& mut self) -> Result<char, ReadError> {
		let c = try!(self.next().ok_or(self.error_eof()));
		
		if c == '\\' {
			let c = try!(self.next().ok_or(self.error_eof()));
			Ok(
				match c {
					'n' => '\n',
					't' => '\t',
					'r' => '\r',
					 _  => c,
				}
			)
		} else {
			Ok(c)
		}
	}
	
	fn read_string(& mut self) -> Result<SExp, ReadError> {
		let c = try!(self.peek().ok_or(self.error_eof()));
		
		if c != '"' {
			return Err(self.error_wrong_char(c, "\""));
		} else {
			self.advance();
		}
		
		let mut str_val = String::new();
		
		loop {
			let c = try!(self.peek().ok_or(self.error_eof()));
			if c == '"' {
				self.advance();
				break;
			} else {
				let c = try!(self.read_escaped_string_char());
				str_val.push(c);
			}
		}
		
		Ok(SExp::String(str_val))
	}
	
	fn read_number(& mut self) -> Result<SExp, ReadError>{
		Err(self.error_not_implemented())
	}

	
	fn advance(& mut self){
		let _ = self.next();
	}
	

	fn error_eof(&self) -> ReadError {
		ReadError::EarlyEOF{
			loc: self.loc.clone(),
			msg: "Unexpected End of File".to_string(),
		}
	}
	
	fn error_wrong_char(&self, c : char, expected : &str) -> ReadError {
		ReadError::WrongChar{
			loc: self.loc.clone(),
			msg: format!("Unexpect character '{}' expected one of '{}'", c, expected),
		}
	}
	
	fn error_paren_mismatch(&self, c1 : char, c2 : char) -> ReadError {
		ReadError::ParenMismatch{
			loc: self.loc.clone(),
			msg: format!("List delimiters don't match:  '{}' and '{}'", c1, c2),
		}
	}
	
	fn error_not_implemented(&self) -> ReadError {
		ReadError::NotImplemented
	}
}

impl<'a> Iterator for SExpParser<'a>{
	type Item = char;
	fn next(&mut self) -> Option<char>{
		let c = self.next_char;
		self.next_char = self.iter.next();
		
		if let Some(c) = c {
			if c == '\n' {
				self.loc.line += 1;
				self.loc.col = 0;
			} else {
				self.loc.col += 1;
			}
		}
		return c;
	}
}

trait Peek{
	type Item;
	fn peek(&self) -> Option<Self::Item>;
}

impl<'a> Peek for SExpParser<'a>{
	type Item = char;
	fn peek(&self) -> Option<Self::Item>{
		return self.next_char;
	}
}

trait CharExt{
	fn is_open_paren(self) -> bool;
	fn is_close_paren(self) -> bool;
	fn is_matching_paren(self, c2 : Self) -> bool;
	fn is_delimiter(self) -> bool;
}

impl CharExt for char{
	fn is_open_paren(self) -> bool { self == '(' || self == '[' || self == '{' }
	fn is_close_paren(self) -> bool { self == ')' || self == ']' || self == '}' }
	fn is_matching_paren(self, other : Self) -> bool {
		match self {
		'(' => other == ')',
		'[' => other == ']',
		'{' => other == '}',
		 _  => false,
		}
	}

	fn is_delimiter(self) -> bool {
		return 
			self.is_whitespace() || 
			self.is_open_paren() || 
			self.is_close_paren() ||
			self == '"';
	}
}


fn main() {
	println!("Hello, world!");
}
