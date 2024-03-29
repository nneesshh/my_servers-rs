use httparse;

struct RequestProtocolIndices {
    path: (usize, usize),
}

struct HeaderIndices {
    name: (usize, usize),
    value: (usize, usize),
}

pub struct Request {
    method: String,
    proto: RequestProtocolIndices,
    headers: Vec<HeaderIndices>,
    body: (usize, usize),
    buffer: Vec<u8>,
}

pub struct Header<'a> {
    pub name: &'a str,
    pub value: &'a [u8],
}

impl Request {
    #[inline(always)]
    pub fn split_body(&mut self) -> Vec<u8> {
        let body = self.buffer.drain(self.body.0..).collect();
        let (start, _) = self.body;
        self.body = (start, start);
        body
    }

    #[inline(always)]
    pub fn method(&self) -> &str {
        self.method.as_str()
    }

    #[inline(always)]
    pub fn path(&self) -> &str {
        ::std::str::from_utf8(&self.buffer[self.proto.path.0..self.proto.path.1]).unwrap()
    }

    #[inline(always)]
    pub fn headers<'a>(&'a self) -> HeaderIter<'a> {
        HeaderIter(&self.buffer, self.headers.iter())
    }
}

pub struct HeaderIter<'a>(&'a [u8], ::std::slice::Iter<'a, HeaderIndices>);

impl<'a> Iterator for HeaderIter<'a> {
    type Item = Header<'a>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        use std::str;
        self.1.next().map(
            |&HeaderIndices {
                 ref name,
                 ref value,
             }| {
                Header {
                    name: str::from_utf8(&self.0[name.0..name.1]).unwrap(),
                    value: &self.0[value.0..value.1],
                }
            },
        )
    }
}

pub enum ParseResult {
    Complete(Request),
    Partial,
}

#[inline(always)]
fn slice_indices(buffer: &[u8], value: &[u8]) -> (usize, usize) {
    let from = buffer.as_ptr() as usize;
    let to = value.as_ptr() as usize;
    assert!(to >= from);

    let start = to - from;
    assert!(start + value.len() <= buffer.len());

    (start, start + value.len())
}

#[inline(always)]
pub fn try_parse_request(buffer: &[u8]) -> Result<ParseResult, httparse::Error> {
    let result = {
        let mut header_buffer = [httparse::EMPTY_HEADER; 32];
        let mut request = httparse::Request::new(&mut header_buffer);
        let request_opt = match request.parse(&*buffer)? {
            httparse::Status::Partial => None,
            httparse::Status::Complete(n) => {
                //
                Some((request, n))
            }
        };

        request_opt
            .map(|(r, n)| {
                let proto = RequestProtocolIndices {
                    path: slice_indices(&*buffer, r.path.unwrap().as_bytes()),
                };

                let method = r.method.unwrap();
                (r, method, proto, n)
            })
            .map(|(r, method, proto, n)| {
                let headers = r
                    .headers
                    .iter()
                    .map(
                        |&httparse::Header {
                             ref name,
                             ref value,
                         }| {
                            HeaderIndices {
                                name: slice_indices(&*buffer, name.as_bytes()),
                                value: slice_indices(&*buffer, value),
                            }
                        },
                    )
                    .collect::<Vec<_>>();
                (method, proto, headers, n)
            })
    };

    if let Some((method, proto, headers, n)) = result {
        return Ok(ParseResult::Complete(Request {
            method: method.to_owned(),
            proto: proto,
            headers: headers,
            body: slice_indices(&*buffer, &buffer[n..]),
            buffer: Vec::from(buffer),
        }));
    }

    return Ok(ParseResult::Partial);
}
