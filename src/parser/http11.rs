#![allow(unused_imports)]
#![allow(dead_code)]

use nom::{IResult};
use nom::IResult::*;
use nom::Err::*;

use nom::{digit,is_alphanumeric};

// Primitives
fn is_token_char(i: u8) -> bool {
  is_alphanumeric(i) ||
  b"!#$%&'*+-.^_`|~".contains(&i)
}
named!(pub token, filter!(is_token_char));

fn is_ws(i: u8) -> bool {
  i == ' ' as u8 && i == '\t' as u8
}
named!(pub repeated_ws, filter!(is_ws));
named!(pub obsolete_ws, chain!(repeated_ws?, || { &b" "[..] }));

named!(pub sp, tag!(" "));
named!(pub crlf, tag!("\r\n"));

fn is_vchar(i: u8) -> bool {
  i > 32 && i <= 126
}
fn is_vchar_or_ws(i: u8) -> bool {
  is_vchar(i) || is_ws(i)
}

named!(pub vchar_1, filter!(is_vchar));
named!(pub vchar_ws_1, filter!(is_vchar_or_ws));


#[derive(PartialEq,Debug)]
pub struct RequestLine<'a> {
    pub method: &'a [u8],
    pub uri: &'a [u8],
    pub version: [&'a [u8];2]
}

named!(pub http_version<[&[u8];2]>,
       chain!(
        tag!("HTTP/") ~
        major: digit ~
        tag!(".") ~
        minor: digit, || {
            [minor, major] // ToDo do we need it?
        }
       )
);

named!(pub request_line<RequestLine>,
       chain!(
        method: token ~
        sp ~
        uri: vchar_1 ~ // ToDo proper URI parsing?
        sp ~
        version: http_version ~
        crlf, || {
            RequestLine {
              method: method,
              uri: uri,
              version: version
            }
        }
       )
);

#[derive(PartialEq,Debug)]
pub struct RequestHeader<'a> {
    pub name: &'a [u8],
    pub value: &'a [u8]
}

named!(pub message_header<RequestHeader>,
       chain!(
         name: token ~
         tag!(":") ~
         sp ~ // ToDo make it optional
         value: vchar_1 ~ // ToDo handle folding?
         crlf, || {
           RequestHeader {
            name: name,
            value: value
           }
         }
       )
);

#[derive(PartialEq,Debug)]
pub struct RequestHead<'a> {
    pub request_line: RequestLine<'a>,
    pub headers: Vec<RequestHeader<'a>>
}

named!(pub request_head<RequestHead>,
       chain!(
        rl: request_line ~
        hs: many0!(message_header) ~
        crlf, || {
          RequestHead {
            request_line: rl,
            headers: hs
          }
        }
       )
);


#[cfg(test)]
mod tests {
  use super::*;
  use nom::*;
  use nom::IResult::*;

  #[test]
  fn request_line_test() {
      let input = b"GET /index.html HTTP/1.1\r\n";
      let result = request_line(input);
      let expected = RequestLine {
        method: b"GET",
        uri: b"/index.html",
        version: [b"1", b"1"]
      };

      assert_eq!(result, Done(&[][..], expected));
  }

  #[test]
  fn header_test() {
      let input = b"Accept: */*\r\n";
      let result = message_header(input);
      let expected = RequestHeader {
        name: b"Accept",
        value: b"*/*"
      };

      assert_eq!(result, Done(&b""[..], expected))
  }

  #[test]
  fn request_head_test() {
      let input =
          b"GET /index.html HTTP/1.1\r\n\
            Host: localhost:8888\r\n\
            User-Agent: curl/7.43.0\r\n\
            Accept: */*\r\n\
            \r\n";
      let result = request_head(input);
      let expected = RequestHead {
        request_line: RequestLine {
        method: b"GET",
        uri: b"/index.html",
        version: [b"1", b"1"]
      },
        headers: vec!(
          RequestHeader {
            name: b"Host",
            value: b"localhost:8888"
          },
          RequestHeader {
            name: b"User-Agent",
            value: b"curl/7.43.0"
          },
          RequestHeader {
            name: b"Accept",
            value: b"*/*"
          }
        )
      };

      assert_eq!(result, Done(&b""[..], expected))
  }
}