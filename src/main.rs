use serde_json;
use std::{
    env, fmt,
    io::{self, Write},
    str::FromStr,
};

fn log(s: String) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    let header = format!("[{} {}]", module_path!(), line!());
    let s = format!("{} {}", header, s);
    io::Write::write_all(&mut stdout, s.as_bytes())
}

mod bencode {
    use std::{fmt, io, str::FromStr};
    #[derive(Debug, Clone, PartialEq, PartialOrd)]
    pub enum ErrorKind {
        Parse(String),
        SplitCount(usize),
    }
    impl fmt::Display for ErrorKind {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let msg = match self {
                Self::Parse(msg) => msg.clone(),
                Self::SplitCount(cnt) => format!("expected 2 elements, got {}", cnt),
            };
            fmt::write(f, format_args!("bencode error {}", msg))
        }
    }
    impl std::error::Error for ErrorKind {}
    trait FieldKey {
        type Value: FieldValue;
    }
    trait FieldValue {}

    impl<T> FieldValue for T where T: Sized {}

    pub struct StringFieldKey(String);

    impl FieldKey for StringFieldKey {
        type Value = String;
    }

    pub struct IntegerFieldKey(i32);

    impl FieldKey for IntegerFieldKey {
        type Value = i32;
    }

    impl FromStr for IntegerFieldKey {
        type Err = io::ErrorKind;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.chars().any(char::is_numeric) {
                Err(io::ErrorKind::InvalidInput)
            } else {
                let mut spl = s.split(':');
                let key = spl.nth(0usize);
                let value = spl.nth(0usize);
                match (key, value) {
                    (Some(key), Some(value)) => Ok(Self(value.parse::<i32>().unwrap())),
                    (_, _) => Err(io::ErrorKind::InvalidData),
                }
            }
        }
    }

    pub struct ArrayFieldKey<I>(I)
    where
        I: Iterator;

    impl<I> FieldKey for ArrayFieldKey<I>
    where
        I: Iterator,
    {
        type Value = I;
    }

    #[derive(Debug, Clone)]
    pub enum Kind<I, A, B, F>
    where
        A: Sized,
        I: Iterator<Item = A>,
        F: FnMut(A) -> B,
    {
        Str(String),
        Int(i32),
        Arr(I),
        Dic(std::iter::Map<I, F>),
    }
    pub struct Data<I, A, B, F>
    where
        A: Sized,
        I: Iterator<Item = A>,
        F: FnMut(A) -> B,
    {
        length: usize,
        kind: Kind<I, A, B, F>,
    }
    impl<I, A, B, F> FromStr for Data<I, A, B, F>
    where
        A: Sized,
        I: Iterator<Item = A>,
        F: FnMut(A) -> B,
    {
        type Err = io::Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut spl = s.split(':');
            match spl.clone().count() {
                2 => match spl.nth(0usize).map(|x| x.parse::<usize>()) {
                    Some(parsing) => match parsing {
                        Ok(length) => match spl.nth(0usize) {
                            Some(content) => Ok(Data {
                                length,
                                kind: Kind::Str(content.to_string()),
                            }),
                            None => {
                                let e_msg = "failed to get content";
                                let e = io::Error::new(io::ErrorKind::InvalidData, e_msg);
                                Err(e)
                            }
                        },
                        Err(e) => {
                            let e_msg = e.to_string();
                            let e = io::Error::new(io::ErrorKind::InvalidData, e_msg);
                            Err(e)
                        }
                    },
                    None => {
                        let e_msg = "coulndt parse length value";
                        let e = io::Error::new(io::ErrorKind::InvalidData, e_msg);
                        Err(e)
                    }
                },
                cnt => {
                    let e_msg = format!("expected splitcount 2, got {}", cnt);
                    let e = io::Error::new(io::ErrorKind::InvalidData, e_msg);
                    Err(e)
                }
            }
        }
    }
}

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    // If encoded_value starts with a digit, it's a number
    if encoded_value.chars().next().unwrap().is_digit(10) {
        // Example: "5:hello" -> "hello"
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<i64>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        return serde_json::Value::String(string.to_string());
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2].as_str();
        let decoded_value: bencode::Data<_, _, _, _> = bencode::Data::from_str(encoded_value)?;
        // println!("{}", decoded_value.to_string());
        Ok(())
    } else {
        let e_msg = format!("unknown command: {}", args[1]);
        let e = io::Error::new(io::ErrorKind::InvalidInput, e_msg);
        Err(e)
    }
}
