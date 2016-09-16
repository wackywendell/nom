//! Byte level parsers and combinators
//!
#[allow(unused_variables)]

/// `tag!(&[T]: nom::AsBytes) => &[T] -> IResult<&[T], &[T]>`
/// declares a byte array as a suite to recognize
///
/// consumes the recognized characters
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::Done;
/// # fn main() {
///  named!(x, tag!("abcd"));
///  let r = x(&b"abcdefgh"[..]);
///  assert_eq!(r, Done(&b"efgh"[..], &b"abcd"[..]));
/// # }
/// ```
#[macro_export]
macro_rules! tag (
  ($i:expr, $tag: expr) => (
    {
      use $crate::{Compare,CompareResult};
      let res: $crate::IResult<_,_> = match ($i).compare($tag) {
        CompareResult::Ok => {
          let blen = $tag.len();
          $crate::IResult::Done(&$i[blen..], &$i[..blen])
        },
        CompareResult::Incomplete => {
          $crate::IResult::Incomplete($crate::Needed::Size($tag.len()))
        },
        CompareResult::Error => {
          $crate::IResult::Error(error_position!($crate::ErrorKind::Tag, $i))
        }
      };
      res
    }
  );
);

/// `is_not!(&[T:AsBytes]) => &[T] -> IResult<&[T], &[T]>`
/// returns the longest list of bytes that do not appear in the provided array
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::Done;
/// # fn main() {
///  named!( not_space, is_not!( " \t\r\n" ) );
///
///  let r = not_space(&b"abcdefgh\nijkl"[..]);
///  assert_eq!(r, Done(&b"\nijkl"[..], &b"abcdefgh"[..]));
///  # }
/// ```
#[macro_export]
macro_rules! is_not(
  ($input:expr, $arr:expr) => (
    {
      use $crate::InputLength;
      use $crate::IterIndices;
      use $crate::Find;

      let res: $crate::IResult<_,_> = match $input.position(|c| {
        c.find($arr)
      }) {
        Some(0) => $crate::IResult::Error(error_position!($crate::ErrorKind::IsNot,$input)),
        Some(n) => {
          let res = $crate::IResult::Done(&$input[n..], &$input[..n]);
          res
        },
        None    => {
          $crate::IResult::Done(&$input[$input.input_len()..], $input)
        }
      };
      res
    }
  );
);

/// `is_a!(&[T]) => &[T] -> IResult<&[T], &[T]>`
/// returns the longest list of bytes that appear in the provided array
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::Done;
/// # fn main() {
///  named!(abcd, is_a!( "abcd" ));
///
///  let r1 = abcd(&b"aaaaefgh"[..]);
///  assert_eq!(r1, Done(&b"efgh"[..], &b"aaaa"[..]));
///
///  let r2 = abcd(&b"dcbaefgh"[..]);
///  assert_eq!(r2, Done(&b"efgh"[..], &b"dcba"[..]));
/// # }
/// ```
#[macro_export]
macro_rules! is_a (
  ($input:expr, $arr:expr) => (
    {
      use $crate::InputLength;
      use $crate::IterIndices;
      use $crate::Find;

      let res: $crate::IResult<_,_> = match $input.position(|c| {
        !c.find($arr)
      }) {
        Some(0) => $crate::IResult::Error(error_position!($crate::ErrorKind::IsA,$input)),
        Some(n) => {
          let res: $crate::IResult<_,_> = $crate::IResult::Done(&$input[n..], &$input[..n]);
          res
        },
        None    => {
          $crate::IResult::Done(&$input[($input).input_len()..], $input)
        }
      };
      res
    }
  );
);

/// `escaped!(&[T] -> IResult<&[T], &[T]>, T, &[T] -> IResult<&[T], &[T]>) => &[T] -> IResult<&[T], &[T]>`
/// matches a byte string with escaped characters.
///
/// The first argument matches the normal characters (it must not accept the control character), the second argument is the control character (like `\` in most languages),
/// the third argument matches the escaped characters
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::Done;
/// # use nom::alpha;
/// # fn main() {
///  named!(esc, escaped!(call!(alpha), '\\', is_a!("\"n\\")));
///  assert_eq!(esc(&b"abcd"[..]), Done(&b""[..], &b"abcd"[..]));
///  assert_eq!(esc(&b"ab\\\"cd"[..]), Done(&b""[..], &b"ab\\\"cd"[..]));
/// # }
/// ```
#[macro_export]
macro_rules! escaped (
  // Internal parser, do not use directly
  (__impl $i: expr, $normal:ident!(  $($args:tt)* ), $control_char: expr, $escapable:ident!(  $($args2:tt)* )) => (
    {
      use $crate::InputLength;
      let cl = || {
        use $crate::Offset;
        let mut index  = 0;

        while index < $i.len() {
          if let $crate::IResult::Done(i,_) = $normal!(&$i[index..], $($args)*) {
            if i.is_empty() {
              return $crate::IResult::Done(&$i[$i.input_len()..], $i)
            } else {
              index = $i.offset(i);
            }
          } else if $i[index] == $control_char as u8 {
            if index + 1 >= $i.len() {
              return $crate::IResult::Error(error_position!($crate::ErrorKind::Escaped,&$i[index..]));
            } else {
              match $escapable!(&$i[index+1..], $($args2)*) {
                $crate::IResult::Done(i,_) => {
                  if i.is_empty() {
                    return $crate::IResult::Done(&$i[$i.input_len()..], $i)
                  } else {
                    index = $i.offset(i);
                  }
                },
                $crate::IResult::Incomplete(i) => return $crate::IResult::Incomplete(i),
                $crate::IResult::Error(e)      => return $crate::IResult::Error(e)
              }
            }
          } else {
            if index == 0 {
              return $crate::IResult::Error(error_position!($crate::ErrorKind::Escaped,&$i[index..]))
            } else {
              return $crate::IResult::Done(&$i[index..], &$i[..index])
            }
          }
        }
        $crate::IResult::Done(&$i[index..], &$i[..index])
      };
      match cl() {
        $crate::IResult::Incomplete(x) => $crate::IResult::Incomplete(x),
        $crate::IResult::Done(i, o)    => $crate::IResult::Done(i, o),
        $crate::IResult::Error(e)      => {
          return $crate::IResult::Error(error_node_position!($crate::ErrorKind::Escaped, $i, e))
        }
      }
    }
  );
  // Internal parser, do not use directly
  (__impl_1 $i:expr, $submac1:ident!( $($args:tt)* ), $control_char: expr, $submac2:ident!( $($args2:tt)*) ) => (
    {
     escaped!(__impl $i, $submac1!($($args)*), $control_char,  $submac2!($($args2)*))
    }
  );
  // Internal parser, do not use directly
  (__impl_1 $i:expr, $submac1:ident!( $($args:tt)* ), $control_char: expr, $g:expr) => (
     escaped!(__impl $i, $submac1!($($args)*), $control_char, call!($g))
  );
  ($i:expr, $submac:ident!( $($args:tt)* ), $control_char: expr, $($rest:tt)+) => (
    {
      let input: &[u8] = $i;

      escaped!(__impl_1 input, $submac!($($args)*), $control_char, $($rest)*)
    }
  );

  ($i:expr, $f:expr, $control_char: expr, $($rest:tt)+) => (
    escaped!(__impl_1 $i, call!($f), $control_char, $($rest)*)
  );
);

/// `escaped_transform!(&[T] -> IResult<&[T], &[T]>, T, &[T] -> IResult<&[T], &[T]>) => &[T] -> IResult<&[T], Vec<T>>`
/// matches a byte string with escaped characters.
///
/// The first argument matches the normal characters (it must not match the control character), the second argument is the control character (like `\` in most languages),
/// the third argument matches the escaped characters and trnasforms them.
///
/// As an example, the chain `abc\tdef` could be `abc    def` (it also consumes the control character)
///
/// WARNING: if you do not use the `verbose-errors` feature, this combinator will currently fail to build
/// because of a type inference error
///
/// ```ignore
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::Done;
/// # use nom::alpha;
/// # use std::str::from_utf8;
/// # fn main() {
/// fn to_s(i:Vec<u8>) -> String {
///   String::from_utf8_lossy(&i).into_owned()
/// }

///  named!(transform < String >,
///    map!(
///      escaped_transform!(call!(alpha), '\\',
///        alt!(
///            tag!("\\")       => { |_| &b"\\"[..] }
///          | tag!("\"")       => { |_| &b"\""[..] }
///          | tag!("n")        => { |_| &b"\n"[..] }
///        )
///      ), to_s
///    )
///  );
///  assert_eq!(transform(&b"ab\\\"cd"[..]), Done(&b""[..], String::from("ab\"cd")));
/// # }
/// ```
#[macro_export]
macro_rules! escaped_transform (
  // Internal parser, do not use directly
  (__impl $i: expr, $normal:ident!(  $($args:tt)* ), $control_char: expr, $transform:ident!(  $($args2:tt)* )) => (
    {
      use $crate::InputLength;
      let cl = || {
        use $crate::Offset;
        let mut index  = 0;
        let mut res = Vec::new();

        while index < $i.len() {
          if let $crate::IResult::Done(i,o) = $normal!(&$i[index..], $($args)*) {
            res.extend(o.iter().cloned());
            if i.is_empty() {
              return $crate::IResult::Done(&$i[$i.input_len()..], res);
            } else {
              index = $i.offset(i);
            }
          } else if $i[index] == $control_char as u8 {
            if index + 1 >= $i.len() {
              return $crate::IResult::Error(error_position!($crate::ErrorKind::EscapedTransform,&$i[index..]));
            } else {
              match $transform!(&$i[index+1..], $($args2)*) {
                $crate::IResult::Done(i,o) => {
                  res.extend(o.iter().cloned());
                  if i.is_empty() {
                    return $crate::IResult::Done(&$i[$i.input_len()..], res)
                  } else {
                    index = $i.offset(i);
                  }
                },
                $crate::IResult::Incomplete(i) => return $crate::IResult::Incomplete(i),
                $crate::IResult::Error(e)      => return $crate::IResult::Error(e)
              }
            }
          } else {
            if index == 0 {
              return $crate::IResult::Error(error_position!($crate::ErrorKind::EscapedTransform,&$i[index..]))
            } else {
              return $crate::IResult::Done(&$i[index..], res)
            }
          }
        }
        $crate::IResult::Done(&$i[index..], res)
      };
      match cl() {
        $crate::IResult::Incomplete(x) => $crate::IResult::Incomplete(x),
        $crate::IResult::Done(i, o)    => $crate::IResult::Done(i, o),
        $crate::IResult::Error(e)      => {
          return $crate::IResult::Error(error_node_position!($crate::ErrorKind::EscapedTransform, $i, e))
        }
      }
    }
  );
  // Internal parser, do not use directly
  (__impl_1 $i:expr, $submac1:ident!( $($args:tt)* ), $control_char: expr, $submac2:ident!( $($args2:tt)*) ) => (
    {
     escaped_transform!(__impl $i, $submac1!($($args)*), $control_char,  $submac2!($($args2)*))
    }
  );
  // Internal parser, do not use directly
  (__impl_1 $i:expr, $submac1:ident!( $($args:tt)* ), $control_char: expr, $g:expr) => (
     escaped_transform_impl!($i, $submac1!($($args)*), $control_char, call!($g))
  );
  ($i:expr, $submac:ident!( $($args:tt)* ), $control_char: expr, $($rest:tt)+) => (
    {
      let input: &[u8] = $i;

      escaped_transform!(__impl_1 input, $submac!($($args)*), $control_char, $($rest)*)
    }
  );

  ($i:expr, $f:expr, $control_char: expr, $($rest:tt)+) => (
    escaped_transform!(__impl_1 $i, call!($f), $control_char, $($rest)*)
  );
);

/// `take_while!(T -> bool) => &[T] -> IResult<&[T], &[T]>`
/// returns the longest list of bytes until the provided function fails.
///
/// The argument is either a function `T -> bool` or a macro returning a `bool`.
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::Done;
/// # use nom::is_alphanumeric;
/// # fn main() {
///  named!( alpha, take_while!( is_alphanumeric ) );
///
///  let r = alpha(&b"abcd\nefgh"[..]);
///  assert_eq!(r, Done(&b"\nefgh"[..], &b"abcd"[..]));
/// # }
/// ```
#[macro_export]
macro_rules! take_while (
  ($input:expr, $submac:ident!( $($args:tt)* )) => (
    {
      use $crate::IterIndices;
      let input = $input;

      match input.position(|c| !$submac!(c, $($args)*)) {
        Some(n) => {
          let res:$crate::IResult<_,_> = $crate::IResult::Done(&input[n..], &input[..n]);
          res
        },
        None    => {
          $crate::IResult::Done(&input[input.len()..], input)
        }
      }
    }
  );
  ($input:expr, $f:expr) => (
    take_while!($input, call!($f));
  );
);

/// `take_while1!(&[T] -> bool) => &[T] -> IResult<&[T], &[T]>`
/// returns the longest (non empty) list of bytes until the provided function fails.
///
/// The argument is either a function `&[T] -> bool` or a macro returning a `bool
#[macro_export]
macro_rules! take_while1 (
  ($input:expr, $submac:ident!( $($args:tt)* )) => (
    {
      let input = $input;

      use $crate::InputLength;
      use $crate::IterIndices;
      if input.input_len() == 0 {
        $crate::IResult::Error(error_position!($crate::ErrorKind::TakeWhile1,input))
      } else {
        match input.position(|c| !$submac!(c, $($args)*)) {
          Some(0) => $crate::IResult::Error(error_position!($crate::ErrorKind::TakeWhile1,input)),
          Some(n) => {
            $crate::IResult::Done(&input[n..], &input[..n])
          },
          None    => {
            $crate::IResult::Done(&input[input.len()..], input)
          }
        }
      }
    }
  );
  ($input:expr, $f:expr) => (
    take_while1!($input, call!($f));
  );
);

/// `take_till!(T -> bool) => &[T] -> IResult<&[T], &[T]>`
/// returns the longest list of bytes until the provided function succeeds
///
/// The argument is either a function `&[T] -> bool` or a macro returning a `bool
#[macro_export]
macro_rules! take_till (
  ($input:expr, $submac:ident!( $($args:tt)* )) => (
    {
      let input = $input;

      use $crate::InputLength;
      use $crate::IterIndices;
      match input.position(|c| $submac!(c, $($args)*)) {
        Some(n) => $crate::IResult::Done(&input[n..], &input[..n]),
        None    => $crate::IResult::Done(&input[input.input_len()..], input)
      }
    }
  );
  ($input:expr, $f:expr) => (
    take_till!($input, call!($f));
  );
);

/// `take!(nb) => &[T] -> IResult<&[T], &[T]>`
/// generates a parser consuming the specified number of bytes
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::Done;
/// # fn main() {
///  // Desmond parser
///  named!(take5, take!( 5 ) );
///
///  let a = b"abcdefgh";
///
///  assert_eq!(take5(&a[..]), Done(&b"fgh"[..], &b"abcde"[..]));
/// # }
/// ```
#[macro_export]
macro_rules! take (
  ($i:expr, $count:expr) => (
    {
      use $crate::IterIndices;
      let input = $i;

      let cnt = $count as usize;

      let res: $crate::IResult<_,_> = match input.index(cnt) {
        None        => $crate::IResult::Incomplete($crate::Needed::Size(cnt)),
        Some(index) => $crate::IResult::Done(&input[index..], &input[..index])
      };
      res
    }
  );
);

/// `take!(nb) => &[T] -> IResult<&[T], &str>`
/// same as take! but returning a &str
#[macro_export]
macro_rules! take_str (
 ( $i:expr, $size:expr ) => (
    {
      let input: &[u8] = $i;

      map_res!(input, take!($size), ::std::str::from_utf8)
    }
  );
);

/// `take_until_and_consume!(tag) => &[T] -> IResult<&[T], &[T]>`
/// generates a parser consuming bytes until the specified byte sequence is found, and consumes it
#[macro_export]
macro_rules! take_until_and_consume(
  ($i:expr, $inp:expr) => (
    {
      let input: &[u8] = $i;

      #[inline(always)]
      fn as_bytes<T: $crate::AsBytes>(b: &T) -> &[u8] {
        b.as_bytes()
      }

      let expected   = $inp;
      let bytes      = as_bytes(&expected);
      take_until_and_consume_bytes!(input, bytes)
    }
  );
);

#[doc(hidden)]
#[macro_export]
macro_rules! take_until_and_consume_bytes (
  ($i:expr, $bytes:expr) => (
    {
      let res: $crate::IResult<_,_> = if $bytes.len() > $i.len() {
        $crate::IResult::Incomplete($crate::Needed::Size($bytes.len()))
      } else {
        let mut index  = 0;
        let mut parsed = false;

        for idx in 0..$i.len() {
          if idx + $bytes.len() > $i.len() {
            index = idx;
            break;
          }
          if &$i[idx..idx + $bytes.len()] == $bytes {
            parsed = true;
            index = idx;
            break;
          }
        }

        if parsed {
          $crate::IResult::Done(&$i[(index + $bytes.len())..], &$i[0..index])
        } else {
          $crate::IResult::Error(error_position!($crate::ErrorKind::TakeUntilAndConsume,$i))
        }
      };
      res
    }
  );
);

/// `take_until!(tag) => &[T] -> IResult<&[T], &[T]>`
/// consumes data until it finds the specified tag
#[macro_export]
macro_rules! take_until(
  ($i:expr, $inp:expr) => (
    {
      let input: &[u8] = $i;

      #[inline(always)]
      fn as_bytes<T: $crate::AsBytes>(b: &T) -> &[u8] {
        b.as_bytes()
      }

      let expected   = $inp;
      let bytes      = as_bytes(&expected);
      take_until_bytes!(input, bytes)
    }
  );
);

#[doc(hidden)]
#[macro_export]
macro_rules! take_until_bytes(
  ($i:expr, $bytes:expr) => (
    {
      let res: $crate::IResult<_,_> = if $bytes.len() > $i.len() {
        $crate::IResult::Incomplete($crate::Needed::Size($bytes.len()))
      } else {
        let mut index  = 0;
        let mut parsed = false;

        for idx in 0..$i.len() {
          if idx + $bytes.len() > $i.len() {
            index = idx;
            break;
          }
          if &$i[idx..idx+$bytes.len()] == $bytes {
            parsed = true;
            index  = idx;
            break;
          }
        }

        if parsed {
          $crate::IResult::Done(&$i[index..], &$i[0..index])
        } else {
          $crate::IResult::Error(error_position!($crate::ErrorKind::TakeUntil,$i))
        }
      };
      res
    }
  );
);

/// `take_until_either_and_consume!(tag) => &[T] -> IResult<&[T], &[T]>`
/// consumes data until it finds any of the specified characters, and consume it
#[macro_export]
macro_rules! take_until_either_and_consume(
  ($i:expr, $inp:expr) => (
    {
      let input: &[u8] = $i;

      #[inline(always)]
      fn as_bytes<T: $crate::AsBytes>(b: &T) -> &[u8] {
        b.as_bytes()
      }

      let expected   = $inp;
      let bytes      = as_bytes(&expected);
      take_until_either_and_consume_bytes!(input, bytes)
    }
  );
);

#[doc(hidden)]
#[macro_export]
macro_rules! take_until_either_and_consume_bytes(
  ($i:expr, $bytes:expr) => (
    {
      let res: $crate::IResult<_,_> = if 1 > $i.len() {
        $crate::IResult::Incomplete($crate::Needed::Size(1))
      } else {
        let mut index  = 0;
        let mut parsed = false;

        for idx in 0..$i.len() {
          if idx + 1 > $i.len() {
            index = idx;
            break;
          }
          for &t in $bytes.iter() {
            if $i[idx] == t {
              parsed = true;
              index = idx;
              break;
            }
          }
          if parsed { break; }
        }

        if parsed {
          $crate::IResult::Done(&$i[(index+1)..], &$i[0..index])
        } else {
          $crate::IResult::Error($crate::Err::error_position!($crate::ErrorKind::TakeUntilEitherAndConsume,$i))
        }
      };
      res
    }
  );
);

/// `take_until_either!(tag) => &[T] -> IResult<&[T], &[T]>`
#[macro_export]
macro_rules! take_until_either(
  ($i:expr, $inp:expr) => (
    {
      let input: &[u8] = $i;

      #[inline(always)]
      fn as_bytes<T: $crate::AsBytes>(b: &T) -> &[u8] {
        b.as_bytes()
      }

      let expected   = $inp;
      let bytes      = as_bytes(&expected);
      take_until_either_bytes!(input, bytes)
    }
  );
);

#[doc(hidden)]
#[macro_export]
macro_rules! take_until_either_bytes(
  ($i:expr, $bytes:expr) => (
    {
      let res: $crate::IResult<_,_> = if 1 > $i.len() {
        $crate::IResult::Incomplete($crate::Needed::Size(1))
      } else {
        let mut index  = 0;
        let mut parsed = false;

        for idx in 0..$i.len() {
          if idx + 1 > $i.len() {
            index = idx;
            break;
          }
          for &t in $bytes.iter() {
            if $i[idx] == t {
              parsed = true;
              index = idx;
              break;
            }
          }
          if parsed { break; }
        }

        if parsed {
          $crate::IResult::Done(&$i[index..], &$i[0..index])
        } else {
          $crate::IResult::Error(error_position!($crate::ErrorKind::TakeUntilEither,$i))
        }
      };
      res
    }
  );
);

/// `length_bytes!(&[T] -> IResult<&[T], nb>) => &[T] -> IResult<&[T], &[T]>
/// gets a number from the first parser, then extracts that many bytes from the
/// remaining stream
#[macro_export]
macro_rules! length_bytes(
  ($i:expr, $submac:ident!( $($args:tt)* )) => (
    {
      let input: &[u8] = $i;

      match  $submac!(input, $($args)*) {
        $crate::IResult::Error(a)      => $crate::IResult::Error(a),
        $crate::IResult::Incomplete(i) => $crate::IResult::Incomplete(i),
        $crate::IResult::Done(i1,nb)   => {
          let nb = nb as usize;
          if i1.len() < nb {
            use $crate::Offset;
            $crate::IResult::Incomplete($crate::Needed::Size($i.offset(i1) + nb))
          } else {
            $crate::IResult::Done(&i1[nb..], &i1[..nb])
          }
        }
      }
    }
  );
  ($i:expr, $f:expr) => (
    length_bytes!($i, call!($f))
  )
);

#[cfg(test)]
mod tests {
  use internal::Needed;
  use internal::IResult::*;
  use util::ErrorKind;
  use nom::{alpha, digit, hex_digit, oct_digit, alphanumeric, space, multispace};

  #[test]
  fn is_a() {
    named!(a_or_b, is_a!(&b"ab"[..]));

    let a = &b"abcd"[..];
    assert_eq!(a_or_b(a), Done(&b"cd"[..], &b"ab"[..]));

    let b = &b"bcde"[..];
    assert_eq!(a_or_b(b), Done(&b"cde"[..], &b"b"[..]));

    let c = &b"cdef"[..];
    assert_eq!(a_or_b(c), Error(error_position!(ErrorKind::IsA,c)));

    let d = &b"bacdef"[..];
    assert_eq!(a_or_b(d), Done(&b"cdef"[..], &b"ba"[..]));
  }

  #[test]
  fn is_not() {
    named!(a_or_b, is_not!(&b"ab"[..]));

    let a = &b"cdab"[..];
    assert_eq!(a_or_b(a), Done(&b"ab"[..], &b"cd"[..]));

    let b = &b"cbde"[..];
    assert_eq!(a_or_b(b), Done(&b"bde"[..], &b"c"[..]));

    let c = &b"abab"[..];
    assert_eq!(a_or_b(c), Error(error_position!(ErrorKind::IsNot,c)));

    let d = &b"cdefba"[..];
    assert_eq!(a_or_b(d), Done(&b"ba"[..], &b"cdef"[..]));

    let e = &b"e"[..];
    assert_eq!(a_or_b(e), Done(&b""[..], &b"e"[..]));

    let f = &b"fghi"[..];
    assert_eq!(a_or_b(f), Done(&b""[..], &b"fghi"[..]));
  }

  #[test]
  fn escaping() {
    named!(esc, escaped!(call!(alpha), '\\', is_a!("\"n\\")));
    assert_eq!(esc(&b"abcd"[..]), Done(&b""[..], &b"abcd"[..]));
    assert_eq!(esc(&b"ab\\\"cd"[..]), Done(&b""[..], &b"ab\\\"cd"[..]));
    assert_eq!(esc(&b"\\\"abcd"[..]), Done(&b""[..], &b"\\\"abcd"[..]));
    assert_eq!(esc(&b"\\n"[..]), Done(&b""[..], &b"\\n"[..]));
    assert_eq!(esc(&b"ab\\\"12"[..]), Done(&b"12"[..], &b"ab\\\""[..]));
    assert_eq!(esc(&b"AB\\"[..]), Error(error_node_position!(ErrorKind::Escaped, &b"AB\\"[..],
      error_position!(ErrorKind::Escaped, &b"\\"[..]))));
    assert_eq!(esc(&b"AB\\A"[..]), Error(error_node_position!(ErrorKind::Escaped, &b"AB\\A"[..],
      error_position!(ErrorKind::IsA, &b"A"[..]))));
  }

  fn to_s(i:Vec<u8>) -> String {
    String::from_utf8_lossy(&i).into_owned()
  }

  #[cfg(feature = "verbose-errors")]
  #[test]
  fn escape_transform() {
    use std::str;

    named!(esc<String>, map!(escaped_transform!(alpha, '\\',
      alt!(
          tag!("\\")       => { |_| &b"\\"[..] }
        | tag!("\"")       => { |_| &b"\""[..] }
        | tag!("n")        => { |_| &b"\n"[..] }
      )), to_s)
    );

    assert_eq!(esc(&b"abcd"[..]), Done(&b""[..], String::from("abcd")));
    assert_eq!(esc(&b"ab\\\"cd"[..]), Done(&b""[..], String::from("ab\"cd")));
    assert_eq!(esc(&b"\\\"abcd"[..]), Done(&b""[..], String::from("\"abcd")));
    assert_eq!(esc(&b"\\n"[..]), Done(&b""[..], String::from("\n")));
    assert_eq!(esc(&b"ab\\\"12"[..]), Done(&b"12"[..], String::from("ab\"")));
    assert_eq!(esc(&b"AB\\"[..]), Error(error_node_position!(ErrorKind::EscapedTransform, &b"AB\\"[..], error_position!(ErrorKind::EscapedTransform, &b"\\"[..]))));
    assert_eq!(esc(&b"AB\\A"[..]), Error(error_node_position!(ErrorKind::EscapedTransform, &b"AB\\A"[..],
      error_position!(ErrorKind::Alt, &b"A"[..]))));

    let e = "è";
    let a = "à";
    println!("è: {:?} | à: {:?}", str::as_bytes(e), str::as_bytes(a));
    named!(esc2< String >, map!(escaped_transform!(call!(alpha), '&',
      alt!(
          tag!("egrave;") => { |_| str::as_bytes("è") }
        | tag!("agrave;") => { |_| str::as_bytes("à") }
      )), to_s)
    );
    assert_eq!(esc2(&b"ab&egrave;DEF"[..]), Done(&b""[..], String::from("abèDEF")));
    assert_eq!(esc2(&b"ab&egrave;D&agrave;EF"[..]), Done(&b""[..], String::from("abèDàEF")));
  }

  #[test]
  fn issue_84() {
    let r0 = is_a!(&b"aaaaefgh"[..], "abcd");
    assert_eq!(r0, Done(&b"efgh"[..], &b"aaaa"[..]));
    let r1 = is_a!(&b"aaaa"[..], "abcd");
    assert_eq!(r1, Done(&b""[..], &b"aaaa"[..]));
    let r2 = is_a!(&b"1"[..], "123456789");
    assert_eq!(r2, Done(&b""[..], &b"1"[..]));
  }

  #[test]
  fn take_str_test() {
    let a = b"omnomnom";

    assert_eq!(take_str!(&a[..], 5), Done(&b"nom"[..], "omnom"));
    assert_eq!(take_str!(&a[..], 9), Incomplete(Needed::Size(9)));
  }

  #[test]
  fn take_until_test() {
    named!(x, take_until_and_consume!("efgh"));
    let r = x(&b"abcdabcdefghijkl"[..]);
    assert_eq!(r, Done(&b"ijkl"[..], &b"abcdabcd"[..]));

    println!("Done 1\n");

    let r2 = x(&b"abcdabcdefgh"[..]);
    assert_eq!(r2, Done(&b""[..], &b"abcdabcd"[..]));

    println!("Done 2\n");
    let r3 = x(&b"abcefg"[..]);
    assert_eq!(r3,  Error(error_position!(ErrorKind::TakeUntilAndConsume, &b"abcefg"[..])));

    assert_eq!(
      x(&b"ab"[..]),
      Incomplete(Needed::Size(4))
    );
  }

  #[test]
  fn take_until_either_incomplete() {
    named!(x, take_until_either!("!."));
    assert_eq!(
      x(&b"123"[..]),
      Error(error_position!(ErrorKind::TakeUntilEither, &b"123"[..]))
    );
  }

  #[test]
  fn take_until_incomplete() {
    named!(y, take_until!("end"));
    assert_eq!(
      y(&b"nd"[..]),
      Incomplete(Needed::Size(3))
    );
    assert_eq!(
      y(&b"123"[..]),
      Error(error_position!(ErrorKind::TakeUntil, &b"123"[..]))
    );
  }

  #[test]
  fn recognize() {
    named!(x, recognize!(delimited!(tag!("<!--"), take!(5), tag!("-->"))));
    let r = x(&b"<!-- abc --> aaa"[..]);
    assert_eq!(r, Done(&b" aaa"[..], &b"<!-- abc -->"[..]));

    let empty = &b""[..];

    named!(ya, recognize!(alpha));
    let ra = ya(&b"abc"[..]);
    assert_eq!(ra, Done(empty, &b"abc"[..]));

    named!(yd, recognize!(digit));
    let rd = yd(&b"123"[..]);
    assert_eq!(rd, Done(empty, &b"123"[..]));

    named!(yhd, recognize!(hex_digit));
    let rhd = yhd(&b"123abcDEF"[..]);
    assert_eq!(rhd, Done(empty, &b"123abcDEF"[..]));

    named!(yod, recognize!(oct_digit));
    let rod = yod(&b"1234567"[..]);
    assert_eq!(rod, Done(empty, &b"1234567"[..]));

    named!(yan, recognize!(alphanumeric));
    let ran = yan(&b"123abc"[..]);
    assert_eq!(ran, Done(empty, &b"123abc"[..]));

    named!(ys, recognize!(space));
    let rs = ys(&b" \t"[..]);
    assert_eq!(rs, Done(empty, &b" \t"[..]));

    named!(yms, recognize!(multispace));
    let rms = yms(&b" \t\r\n"[..]);
    assert_eq!(rms, Done(empty, &b" \t\r\n"[..]));
  }

  #[test]
  fn take_while() {
    use nom::is_alphabetic;
    named!(f, take_while!(is_alphabetic));
    let a = b"";
    let b = b"abcd";
    let c = b"abcd123";
    let d = b"123";

    assert_eq!(f(&a[..]), Done(&a[..], &a[..]));
    assert_eq!(f(&b[..]), Done(&a[..], &b[..]));
    assert_eq!(f(&c[..]), Done(&d[..], &b[..]));
    assert_eq!(f(&d[..]), Done(&d[..], &a[..]));
  }

  #[test]
  fn take_while1() {
    use nom::is_alphabetic;
    named!(f, take_while1!(is_alphabetic));
    let a = b"";
    let b = b"abcd";
    let c = b"abcd123";
    let d = b"123";

    assert_eq!(f(&a[..]), Error(error_position!(ErrorKind::TakeWhile1, &b""[..])));
    assert_eq!(f(&b[..]), Done(&a[..], &b[..]));
    assert_eq!(f(&c[..]), Done(&b"123"[..], &b[..]));
    assert_eq!(f(&d[..]), Error(error_position!(ErrorKind::TakeWhile1, &d[..])));
  }

  #[cfg(feature = "nightly")]
  use test::Bencher;

  #[cfg(feature = "nightly")]
  #[bench]
  fn take_while_bench(b: &mut Bencher) {
    use nom::is_alphabetic;
    named!(f, take_while!(is_alphabetic));
    b.iter(|| {
      f(&b"abcdefghijklABCDEejfrfrjgro12aa"[..])
    });
  }

  #[test]
  fn recognize_take_while() {
    use nom::is_alphanumeric;
    named!(x, take_while!(is_alphanumeric));
    named!(y, recognize!(x));
    assert_eq!(x(&b"ab"[..]), Done(&[][..], &b"ab"[..]));
    println!("X: {:?}", x(&b"ab"[..]));
    assert_eq!(y(&b"ab"[..]), Done(&[][..], &b"ab"[..]));
  }

  #[test]
  fn length_bytes() {
    use nom::le_u8;
    named!(x, length_bytes!(le_u8));
    assert_eq!(x(b"\x02..>>"), Done(&b">>"[..], &b".."[..]));
    assert_eq!(x(b"\x02.."), Done(&[][..], &b".."[..]));
    assert_eq!(x(b"\x02."), Incomplete(Needed::Size(3)));
    assert_eq!(x(b"\x02"), Incomplete(Needed::Size(3)));

    named!(y, chain!(tag!("magic") ~ b: length_bytes!(le_u8), || b));
    assert_eq!(y(b"magic\x02..>>"), Done(&b">>"[..], &b".."[..]));
    assert_eq!(y(b"magic\x02.."), Done(&[][..], &b".."[..]));
    assert_eq!(y(b"magic\x02."), Incomplete(Needed::Size(8)));
    assert_eq!(y(b"magic\x02"), Incomplete(Needed::Size(8)));
  }
}
