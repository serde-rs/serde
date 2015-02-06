#![feature(plugin, io)]
#![allow(non_camel_case_types)]

#[plugin]
extern crate serde2_macros;

extern crate serde2;
extern crate "rustc-serialize" as rustc_serialize;
extern crate test;

use std::old_io;
use std::old_io::ByRefWriter;
use std::num::FromPrimitive;
use test::Bencher;

use serde2::json::ser::escape_str;
use serde2::json;
use serde2::ser::{Serialize, Serializer};
use serde2::ser;
use serde2::de::{Deserialize, Deserializer};
use serde2::de;

use rustc_serialize::Encodable;

enum HttpField {
    Protocol,
    Status,
    HostStatus,
    UpStatus,
    Method,
    ContentType,
    UserAgent,
    Referer,
    RequestUri,
}

impl de::Deserialize for HttpField {
    fn deserialize<
        S: Deserializer,
    >(state: &mut S) -> Result<HttpField, S::Error> {
        struct Visitor;

        impl de::Visitor for Visitor {
            type Value = HttpField;

            fn visit_str<
                E: de::Error,
            >(&mut self, value: &str) -> Result<HttpField, E> {
                let x = match value {
                    "protocol" => HttpField::Protocol,
                    "status" => HttpField::Status,
                    "host_status" => HttpField::HostStatus,
                    "up_status" => HttpField::UpStatus,
                    "method" => HttpField::Method,
                    "content_type" => HttpField::ContentType,
                    "user_agent" => HttpField::UserAgent,
                    "referer" => HttpField::Referer,
                    "request_uri" => HttpField::RequestUri,
                    _ => panic!(),
                };
                Ok(x)
            }
        }

        state.visit(&mut Visitor)
    }
}

#[derive(Debug, PartialEq, RustcEncodable, RustcDecodable)]
#[derive_serialize]
//#[derive_deserialize]
struct Http {
    protocol: HttpProtocol,
    status: u32,
    host_status: u32,
    up_status: u32,
    method: HttpMethod,
    content_type: String,
    user_agent: String,
    referer: String,
    request_uri: String,
}

impl de::Deserialize for Http {
    fn deserialize<
        S: Deserializer,
    >(state: &mut S) -> Result<Http, S::Error> {
        struct Visitor;

        impl de::Visitor for Visitor {
            type Value = Http;

            fn visit_map<
                V: de::MapVisitor,
            >(&mut self, mut visitor: V) -> Result<Http, V::Error> {
                let mut protocol = None;
                let mut status = None;
                let mut host_status = None;
                let mut up_status = None;
                let mut method = None;
                let mut content_type = None;
                let mut user_agent = None;
                let mut referer = None;
                let mut request_uri = None;

                while let Some(key) = try!(visitor.visit_key()) {
                    match key {
                        HttpField::Protocol => { protocol = Some(try!(visitor.visit_value())); }
                        HttpField::Status => { status = Some(try!(visitor.visit_value())); }
                        HttpField::HostStatus => { host_status = Some(try!(visitor.visit_value())); }
                        HttpField::UpStatus => { up_status = Some(try!(visitor.visit_value())); }
                        HttpField::Method => { method = Some(try!(visitor.visit_value())); }
                        HttpField::ContentType => { content_type = Some(try!(visitor.visit_value())); }
                        HttpField::UserAgent => { user_agent = Some(try!(visitor.visit_value())); }
                        HttpField::Referer => { referer = Some(try!(visitor.visit_value())); }
                        HttpField::RequestUri => { request_uri = Some(try!(visitor.visit_value())); }
                    }
                }

                Ok(Http {
                    protocol: protocol.unwrap(),
                    status: status.unwrap(),
                    host_status: host_status.unwrap(),
                    up_status: up_status.unwrap(),
                    method: method.unwrap(),
                    content_type: content_type.unwrap(),
                    user_agent: user_agent.unwrap(),
                    referer: referer.unwrap(),
                    request_uri: request_uri.unwrap(),
                })
            }
        }

        state.visit(&mut Visitor)
    }
}

#[derive(Copy, Debug, PartialEq, FromPrimitive)]
enum HttpProtocol {
    HTTP_PROTOCOL_UNKNOWN,
    HTTP10,
    HTTP11,
}

impl rustc_serialize::Encodable for HttpProtocol {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as u8).encode(s)
    }
}

impl rustc_serialize::Decodable for HttpProtocol {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<HttpProtocol, D::Error> {
        match FromPrimitive::from_u8(try!(d.read_u8())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from u8")),
        }
    }
}

impl ser::Serialize for HttpProtocol {
    #[inline]
    fn visit<
        V: ser::Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        visitor.visit_u8(*self as u8)
    }
}

impl de::Deserialize for HttpProtocol {
    #[inline]
    fn deserialize<
        S: Deserializer,
    >(state: &mut S) -> Result<HttpProtocol, S::Error> {
        state.visit(&mut de::PrimitiveVisitor)
    }
}

#[derive(Copy, Debug, PartialEq, FromPrimitive)]
enum HttpMethod {
    METHOD_UNKNOWN,
    GET,
    POST,
    DELETE,
    PUT,
    HEAD,
    PURGE,
    OPTIONS,
    PROPFIND,
    MKCOL,
    PATCH,
}

impl rustc_serialize::Encodable for HttpMethod {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as u8).encode(s)
    }
}

impl rustc_serialize::Decodable for HttpMethod {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<HttpMethod, D::Error> {
        match FromPrimitive::from_u8(try!(d.read_u8())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from u8")),
        }
    }
}

impl ser::Serialize for HttpMethod {
    #[inline]
    fn visit<
        V: ser::Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        visitor.visit_u8(*self as u8)
    }
}

impl de::Deserialize for HttpMethod {
    #[inline]
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<HttpMethod, S::Error> {
        state.visit(&mut de::PrimitiveVisitor)
    }
}

#[derive(Copy, Debug, PartialEq, FromPrimitive)]
enum CacheStatus {
    CACHESTATUS_UNKNOWN,
    Miss,
    Expired,
    Hit,
}

impl rustc_serialize::Encodable for CacheStatus {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as u8).encode(s)
    }
}

impl rustc_serialize::Decodable for CacheStatus {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<CacheStatus, D::Error> {
        match FromPrimitive::from_u8(try!(d.read_u8())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from uint")),
        }
    }
}

impl ser::Serialize for CacheStatus {
    #[inline]
    fn visit<
        V: ser::Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        visitor.visit_u8(*self as u8)
    }
}

impl de::Deserialize for CacheStatus {
    #[inline]
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<CacheStatus, S::Error> {
        state.visit(&mut de::PrimitiveVisitor)
    }
}

enum OriginField {
    Ip,
    Port,
    Hostname,
    Protocol,
}

impl de::Deserialize for OriginField {
    fn deserialize<
        S: Deserializer,
    >(state: &mut S) -> Result<OriginField, S::Error> {
        struct Visitor;

        impl de::Visitor for Visitor {
            type Value = OriginField;

            fn visit_str<
                E: de::Error,
            >(&mut self, value: &str) -> Result<OriginField, E> {
                let x = match value {
                    "ip" => OriginField::Ip,
                    "port" => OriginField::Port,
                    "hostname" => OriginField::Hostname,
                    "protocol" => OriginField::Protocol,
                    _ => panic!(),
                };
                Ok(x)
            }
        }

        state.visit(&mut Visitor)
    }
}

#[derive(Debug, PartialEq, RustcEncodable, RustcDecodable)]
#[derive_serialize]
//#[derive_deserialize]
struct Origin {
    ip: String,
    port: u32,
    hostname: String,
    protocol: OriginProtocol,
}

impl Deserialize for Origin {
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<Origin, S::Error> {
        struct Visitor;

        impl de::Visitor for Visitor {
            type Value = Origin;

            fn visit_map<
                V: de::MapVisitor,
            >(&mut self, mut visitor: V) -> Result<Origin, V::Error> {
                let mut ip = None;
                let mut port = None;
                let mut hostname = None;
                let mut protocol = None;

                while let Some(key) = try!(visitor.visit_key()) {
                    match key {
                        OriginField::Ip => { ip = Some(try!(visitor.visit_value())); }
                        OriginField::Port => { port = Some(try!(visitor.visit_value())); }
                        OriginField::Hostname => { hostname = Some(try!(visitor.visit_value())); }
                        OriginField::Protocol => { protocol = Some(try!(visitor.visit_value())); }
                    }
                }

                Ok(Origin {
                    ip: ip.unwrap(),
                    port: port.unwrap(),
                    hostname: hostname.unwrap(),
                    protocol: protocol.unwrap(),
                })
            }
        }

        state.visit(&mut Visitor)
    }
}

#[derive(Copy, Debug, PartialEq, FromPrimitive)]
enum OriginProtocol {
    ORIGIN_PROTOCOL_UNKNOWN,
    HTTP,
    HTTPS,
}

impl rustc_serialize::Encodable for OriginProtocol {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as u8).encode(s)
    }
}

impl rustc_serialize::Decodable for OriginProtocol {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<OriginProtocol, D::Error> {
        match FromPrimitive::from_u8(try!(d.read_u8())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from u8")),
        }
    }
}

impl ser::Serialize for OriginProtocol {
    #[inline]
    fn visit<
        V: ser::Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        visitor.visit_u8(*self as u8)
    }
}

impl de::Deserialize for OriginProtocol {
    #[inline]
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<OriginProtocol, S::Error> {
        state.visit(&mut de::PrimitiveVisitor)
    }
}

#[derive(Copy, Debug, PartialEq, FromPrimitive)]
enum ZonePlan {
    ZONEPLAN_UNKNOWN,
    FREE,
    PRO,
    BIZ,
    ENT,
}

impl rustc_serialize::Encodable for ZonePlan {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as u8).encode(s)
    }
}

impl rustc_serialize::Decodable for ZonePlan {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<ZonePlan, D::Error> {
        match FromPrimitive::from_u8(try!(d.read_u8())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from u8")),
        }
    }
}

impl ser::Serialize for ZonePlan {
    #[inline]
    fn visit<
        V: ser::Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        visitor.visit_u8(*self as u8)
    }
}

impl de::Deserialize for ZonePlan {
    #[inline]
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<ZonePlan, S::Error> {
        state.visit(&mut de::PrimitiveVisitor)
    }
}

#[derive(Copy, Debug, PartialEq, FromPrimitive)]
enum Country {
	UNKNOWN,
	A1,
	A2,
	O1,
	AD,
	AE,
	AF,
	AG,
	AI,
	AL,
	AM,
	AO,
	AP,
	AQ,
	AR,
	AS,
	AT,
	AU,
	AW,
	AX,
	AZ,
	BA,
	BB,
	BD,
	BE,
	BF,
	BG,
	BH,
	BI,
	BJ,
	BL,
	BM,
	BN,
	BO,
	BQ,
	BR,
	BS,
	BT,
	BV,
	BW,
	BY,
	BZ,
	CA,
	CC,
	CD,
	CF,
	CG,
	CH,
	CI,
	CK,
	CL,
	CM,
	CN,
	CO,
	CR,
	CU,
	CV,
	CW,
	CX,
	CY,
	CZ,
	DE,
	DJ,
	DK,
	DM,
	DO,
	DZ,
	EC,
	EE,
	EG,
	EH,
	ER,
	ES,
	ET,
	EU,
	FI,
	FJ,
	FK,
	FM,
	FO,
	FR,
	GA,
	GB,
	GD,
	GE,
	GF,
	GG,
	GH,
	GI,
	GL,
	GM,
	GN,
	GP,
	GQ,
	GR,
	GS,
	GT,
	GU,
	GW,
	GY,
	HK,
	HM,
	HN,
	HR,
	HT,
	HU,
	ID,
	IE,
	IL,
	IM,
	IN,
	IO,
	IQ,
	IR,
	IS,
	IT,
	JE,
	JM,
	JO,
	JP,
	KE,
	KG,
	KH,
	KI,
	KM,
	KN,
	KP,
	KR,
	KW,
	KY,
	KZ,
	LA,
	LB,
	LC,
	LI,
	LK,
	LR,
	LS,
	LT,
	LU,
	LV,
	LY,
	MA,
	MC,
	MD,
	ME,
	MF,
	MG,
	MH,
	MK,
	ML,
	MM,
	MN,
	MO,
	MP,
	MQ,
	MR,
	MS,
	MT,
	MU,
	MV,
	MW,
	MX,
	MY,
	MZ,
	NA,
	NC,
	NE,
	NF,
	NG,
	NI,
	NL,
	NO,
	NP,
	NR,
	NU,
	NZ,
	OM,
	PA,
	PE,
	PF,
	PG,
	PH,
	PK,
	PL,
	PM,
	PN,
	PR,
	PS,
	PT,
	PW,
	PY,
	QA,
	RE,
	RO,
	RS,
	RU,
	RW,
	SA,
	SB,
	SC,
	SD,
	SE,
	SG,
	SH,
	SI,
	SJ,
	SK,
	SL,
	SM,
	SN,
	SO,
	SR,
	SS,
	ST,
	SV,
	SX,
	SY,
	SZ,
	TC,
	TD,
	TF,
	TG,
	TH,
	TJ,
	TK,
	TL,
	TM,
	TN,
	TO,
	TR,
	TT,
	TV,
	TW,
	TZ,
	UA,
	UG,
	UM,
	US,
	UY,
	UZ,
	VA,
	VC,
	VE,
	VG,
	VI,
	VN,
	VU,
	WF,
	WS,
	XX,
	YE,
	YT,
	ZA,
	ZM,
	ZW,
}

impl rustc_serialize::Encodable for Country {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        (*self as u8).encode(s)
    }
}

impl rustc_serialize::Decodable for Country {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Country, D::Error> {
        match FromPrimitive::from_u8(try!(d.read_u8())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from u8")),
        }
    }
}

impl ser::Serialize for Country {
    #[inline]
    fn visit<
        V: ser::Visitor,
    >(&self, visitor: &mut V) -> Result<V::Value, V::Error> {
        visitor.visit_u8(*self as u8)
    }
}

impl de::Deserialize for Country {
    #[inline]
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<Country, S::Error> {
        state.visit(&mut de::PrimitiveVisitor)
    }
}

enum LogField {
    Timestamp,
    ZoneId,
    ZonePlan,
    Http,
    Origin,
    Country,
    CacheStatus,
    ServerIp,
    ServerName,
    RemoteIp,
    BytesDlv,
    RayId,
}

impl de::Deserialize for LogField {
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<LogField, S::Error> {
        struct Visitor;

        impl de::Visitor for Visitor {
            type Value = LogField;

            fn visit_str<
                E: de::Error,
            >(&mut self, value: &str) -> Result<LogField, E> {
                let x = match value {
                    "timestamp" => LogField::Timestamp,
                    "zone_id" => LogField::ZoneId,
                    "zone_plan" => LogField::ZonePlan,
                    "http" => LogField::Http,
                    "origin" => LogField::Origin,
                    "country" => LogField::Country,
                    "cache_status" => LogField::CacheStatus,
                    "server_ip" => LogField::ServerIp,
                    "server_name" => LogField::ServerName,
                    "remote_ip" => LogField::RemoteIp,
                    "bytes_dlv" => LogField::BytesDlv,
                    "ray_id" => LogField::RayId,
                    _ => panic!(),
                };
                Ok(x)
            }
        }

        state.visit(&mut Visitor)
    }
}

#[derive(Debug, PartialEq, RustcEncodable, RustcDecodable)]
#[derive_serialize]
//#[derive_deserialize]
struct Log {
    timestamp: i64,
    zone_id: u32,
    zone_plan: ZonePlan,
    http: Http,
    origin: Origin,
    country: Country,
    cache_status: CacheStatus,
    server_ip: String,
    server_name: String,
    remote_ip: String,
    bytes_dlv: u64,
    ray_id: String,
}

impl Deserialize for Log {
    fn deserialize<
        S: de::Deserializer,
    >(state: &mut S) -> Result<Log, S::Error> {
        struct Visitor;

        impl de::Visitor for Visitor {
            type Value = Log;

            fn visit_map<
                V: de::MapVisitor,
            >(&mut self, mut visitor: V) -> Result<Log, V::Error> {
                let mut timestamp = None;
                let mut zone_id = None;
                let mut zone_plan = None;
                let mut http = None;
                let mut origin = None;
                let mut country = None;
                let mut cache_status = None;
                let mut server_ip = None;
                let mut server_name = None;
                let mut remote_ip = None;
                let mut bytes_dlv = None;
                let mut ray_id = None;

                while let Some(key) = try!(visitor.visit_key()) {
                    match key {
                        LogField::Timestamp => { timestamp = Some(try!(visitor.visit_value())); }
                        LogField::ZoneId => { zone_id = Some(try!(visitor.visit_value())); }
                        LogField::ZonePlan => { zone_plan = Some(try!(visitor.visit_value())); }
                        LogField::Http => { http = Some(try!(visitor.visit_value())); }
                        LogField::Origin => { origin = Some(try!(visitor.visit_value())); }
                        LogField::Country => { country = Some(try!(visitor.visit_value())); }
                        LogField::CacheStatus => { cache_status = Some(try!(visitor.visit_value())); }
                        LogField::ServerIp => { server_ip = Some(try!(visitor.visit_value())); }
                        LogField::ServerName => { server_name = Some(try!(visitor.visit_value())); }
                        LogField::RemoteIp => { remote_ip = Some(try!(visitor.visit_value())); }
                        LogField::BytesDlv => { bytes_dlv = Some(try!(visitor.visit_value())); }
                        LogField::RayId => { ray_id = Some(try!(visitor.visit_value())); }
                    }
                }

                Ok(Log {
                    timestamp: timestamp.unwrap(),
                    zone_id: zone_id.unwrap(),
                    zone_plan: zone_plan.unwrap(),
                    http: http.unwrap(),
                    origin: origin.unwrap(),
                    country: country.unwrap(),
                    cache_status: cache_status.unwrap(),
                    server_ip: server_ip.unwrap(),
                    server_name: server_name.unwrap(),
                    remote_ip: remote_ip.unwrap(),
                    bytes_dlv: bytes_dlv.unwrap(),
                    ray_id: ray_id.unwrap(),
                })
            }
        }

        state.visit(&mut Visitor)
    }
}

impl Log {
    fn new() -> Log {
        Log {
            timestamp: 2837513946597,
            zone_id: 123456,
            zone_plan: ZonePlan::FREE,
            http: Http {
                protocol: HttpProtocol::HTTP11,
                status: 200,
                host_status: 503,
                up_status: 520,
                method: HttpMethod::GET,
                content_type: "text/html".to_string(),
                user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36".to_string(),
                referer: "https://www.cloudflare.com/".to_string(),
                request_uri: "/cdn-cgi/trace".to_string(),
            },
            origin: Origin {
                ip: "1.2.3.4".to_string(),
                port: 8000,
                hostname: "www.example.com".to_string(),
                protocol: OriginProtocol::HTTPS,
            },
            country: Country::US,
            cache_status: CacheStatus::Hit,
            server_ip: "192.168.1.1".to_string(),
            server_name: "metal.cloudflare.com".to_string(),
            remote_ip: "10.1.2.3".to_string(),
            bytes_dlv: 123456,
            ray_id: "10c73629cce30078-LAX".to_string(),
        }
    }
}

macro_rules! likely(
    ($val:expr) => {
        {
            extern {
                #[link_name = "llvm.expect.i8"]
                fn expect(val: u8, expected_val: u8) -> u8;
            }
            let x: bool = $val;
            unsafe { expect(x as u8, 1) != 0 }
        }
    }
);

macro_rules! unlikely(
    ($val:expr) => {
        {
            extern {
                #[link_name = "llvm.expect.i8"]
                fn expect(val: u8, expected_val: u8) -> u8;
            }
            let x: bool = $val;
            unsafe { expect(x as u8, 0) != 0 }
        }
    }
);

struct MyMemWriter0 {
    buf: Vec<u8>,
}

impl MyMemWriter0 {
    pub fn with_capacity(cap: usize) -> MyMemWriter0 {
        MyMemWriter0 {
            buf: Vec::with_capacity(cap)
        }
    }
}


impl Writer for MyMemWriter0 {
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> old_io::IoResult<()> {
        self.buf.push_all(buf);
        Ok(())
    }
}

struct MyMemWriter1 {
    buf: Vec<u8>,
}

impl MyMemWriter1 {
    pub fn with_capacity(cap: usize) -> MyMemWriter1 {
        MyMemWriter1 {
            buf: Vec::with_capacity(cap)
        }
    }
}

// LLVM isn't yet able to lower `Vec::push_all` into a memcpy, so this helps
// MemWriter eke out that last bit of performance.
//#[inline(always)]
fn push_all_bytes(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // we would have failed if `reserve` overflowed.
        dst.set_len(dst_len + src_len);

        ::std::ptr::copy_nonoverlapping_memory(
            dst.as_mut_ptr().offset(dst_len as isize),
            src.as_ptr(),
            src_len);
    }
}

impl Writer for MyMemWriter1 {
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> old_io::IoResult<()> {
        push_all_bytes(&mut self.buf, buf);
        Ok(())
    }
}

const JSON_STR: &'static str = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":1,"http":{"protocol":2,"status":200,"host_status":503,"up_status":520,"method":1,"content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":2},"country":238,"cache_status":3,"server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

#[test]
fn test_encoder() {
    use rustc_serialize::Encodable;

    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);

    {
        let mut encoder = rustc_serialize::json::Encoder::new(&mut wr);
        log.encode(&mut encoder).unwrap();
    }

    assert_eq!(&wr[], JSON_STR.as_bytes());
}

#[bench]
fn bench_encoder(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);

    {
        let mut encoder = rustc_serialize::json::Encoder::new(&mut wr);
        log.encode(&mut encoder).unwrap();
    }

    b.bytes = wr.len() as u64;

    b.iter(|| {
        wr.clear();

        let mut encoder = rustc_serialize::json::Encoder::new(&mut wr);
        log.encode(&mut encoder).unwrap();
    });
}

#[test]
fn test_serializer() {
    let log = Log::new();
    let json = json::to_vec(&log).unwrap();
    assert_eq!(json, JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log).unwrap();
    b.bytes = json.len() as u64;

    b.iter(|| {
        let _ = json::to_vec(&log);
    });
}

#[test]
fn test_serializer_vec() {
    let log = Log::new();
    let wr = Vec::with_capacity(1024);
    let mut serializer = json::Writer::new(wr);
    serializer.visit(&log).unwrap();

    let json = serializer.into_inner();
    assert_eq!(&json[], JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer_vec(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log).unwrap();
    b.bytes = json.len() as u64;

    let mut wr = Vec::with_capacity(1024);

    b.iter(|| {
        wr.clear();

        let mut serializer = json::Writer::new(wr.by_ref());
        serializer.visit(&log).unwrap();
        let _json = serializer.into_inner();
    });
}

#[bench]
fn bench_serializer_slice(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log).unwrap();
    b.bytes = json.len() as u64;

    let mut buf = [0; 1024];

    b.iter(|| {
        for item in buf.iter_mut(){ *item = 0; }
        let mut wr = std::old_io::BufWriter::new(&mut buf);

        let mut serializer = json::Writer::new(wr.by_ref());
        serializer.visit(&log).unwrap();
        let _json = serializer.into_inner();
    });
}

#[test]
fn test_serializer_my_mem_writer0() {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);

    {
        let mut serializer = json::Writer::new(wr.by_ref());
        serializer.visit(&log).unwrap();
        let _json = serializer.into_inner();
    }

    assert_eq!(&wr.buf[], JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer_my_mem_writer0(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log).unwrap();
    b.bytes = json.len() as u64;

    let mut wr = MyMemWriter0::with_capacity(1024);

    b.iter(|| {
        wr.buf.clear();

        let mut serializer = json::Writer::new(wr.by_ref());
        serializer.visit(&log).unwrap();
        let _json = serializer.into_inner();
    });
}

#[test]
fn test_serializer_my_mem_writer1() {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);

    {
        let mut serializer = json::Writer::new(wr.by_ref());
        serializer.visit(&log).unwrap();
        let _json = serializer.into_inner();
    }

    assert_eq!(&wr.buf[], JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer_my_mem_writer1(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log).unwrap();
    b.bytes = json.len() as u64;

    let mut wr = MyMemWriter1::with_capacity(1024);

    b.iter(|| {
        wr.buf.clear();

        let mut serializer = json::Writer::new(wr.by_ref());
        serializer.visit(&log).unwrap();
        let _json = serializer.into_inner();
    });
}

#[bench]
fn bench_copy(b: &mut Bencher) {
    let json = JSON_STR.as_bytes().to_vec();
    b.bytes = json.len() as u64;

    b.iter(|| {
        let _json = JSON_STR.as_bytes().to_vec();
    });
}

fn manual_serialize_no_escape<W: Writer>(wr: &mut W, log: &Log) {
    wr.write_str("{\"timestamp\":").unwrap();
    (write!(wr, "{}", log.timestamp)).unwrap();
    wr.write_str(",\"zone_id\":").unwrap();
    (write!(wr, "{}", log.zone_id)).unwrap();
    wr.write_str(",\"zone_plan\":").unwrap();
    (write!(wr, "{}", log.zone_plan as usize)).unwrap();

    wr.write_str(",\"http\":{\"protocol\":").unwrap();
    (write!(wr, "{}", log.http.protocol as usize)).unwrap();
    wr.write_str(",\"status\":").unwrap();
    (write!(wr, "{}", log.http.status)).unwrap();
    wr.write_str(",\"host_status\":").unwrap();
    (write!(wr, "{}", log.http.host_status)).unwrap();
    wr.write_str(",\"up_status\":").unwrap();
    (write!(wr, "{}", log.http.up_status)).unwrap();
    wr.write_str(",\"method\":").unwrap();
    (write!(wr, "{}", log.http.method as usize)).unwrap();
    wr.write_str(",\"content_type\":").unwrap();
    (write!(wr, "\"{}\"", log.http.content_type)).unwrap();
    wr.write_str(",\"user_agent\":").unwrap();
    (write!(wr, "\"{}\"", log.http.user_agent)).unwrap();
    wr.write_str(",\"referer\":").unwrap();
    (write!(wr, "\"{}\"", log.http.referer)).unwrap();
    wr.write_str(",\"request_uri\":").unwrap();
    (write!(wr, "\"{}\"", log.http.request_uri)).unwrap();

    wr.write_str("},\"origin\":{").unwrap();

    wr.write_str("\"ip\":").unwrap();
    (write!(wr, "\"{}\"", log.origin.ip)).unwrap();
    wr.write_str(",\"port\":").unwrap();
    (write!(wr, "{}", log.origin.port)).unwrap();
    wr.write_str(",\"hostname\":").unwrap();
    (write!(wr, "\"{}\"", log.origin.hostname)).unwrap();

    wr.write_str(",\"protocol\":").unwrap();
    (write!(wr, "{}", log.origin.protocol as usize)).unwrap();

    wr.write_str("},\"country\":").unwrap();
    (write!(wr, "{}", log.country as usize)).unwrap();
    wr.write_str(",\"cache_status\":").unwrap();
    (write!(wr, "{}", log.cache_status as usize)).unwrap();
    wr.write_str(",\"server_ip\":").unwrap();
    (write!(wr, "\"{}\"", log.server_ip)).unwrap();
    wr.write_str(",\"server_name\":").unwrap();
    (write!(wr, "\"{}\"", log.server_name)).unwrap();
    wr.write_str(",\"remote_ip\":").unwrap();
    (write!(wr, "\"{}\"", log.remote_ip)).unwrap();
    wr.write_str(",\"bytes_dlv\":").unwrap();
    (write!(wr, "{}", log.bytes_dlv)).unwrap();

    wr.write_str(",\"ray_id\":").unwrap();
    (write!(wr, "\"{}\"", log.ray_id)).unwrap();
    wr.write_str("}").unwrap();
}

fn manual_serialize_escape<W: Writer>(wr: &mut W, log: &Log) {
    wr.write_str("{").unwrap();
    escape_str(wr, "timestamp").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.timestamp)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "zone_id").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.zone_id)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "zone_plan").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.zone_plan as isize)).unwrap();

    wr.write_str(",").unwrap();
    escape_str(wr, "http").unwrap();
    wr.write_str(":{").unwrap();
    escape_str(wr, "protocol").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.http.protocol as usize)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "status").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.http.status)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "host_status").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.http.host_status)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "up_status").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.http.up_status)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "method").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.http.method as usize)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "content_type").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, &log.http.content_type).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "user_agent").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, &log.http.user_agent).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "referer").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, &log.http.referer).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "request_uri").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, &log.http.request_uri).unwrap();

    wr.write_str("},").unwrap();
    escape_str(wr, "origin").unwrap();
    wr.write_str(":{").unwrap();

    escape_str(wr, "ip").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, &log.origin.ip).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "port").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.origin.port)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "hostname").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, &log.origin.hostname).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "protocol").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.origin.protocol as usize)).unwrap();

    wr.write_str("},").unwrap();
    escape_str(wr, "country").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.country as usize)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "cache_status").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.cache_status as usize)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "server_ip").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, &log.server_ip).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "server_name").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, &log.server_name).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "remote_ip").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, &log.remote_ip).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "bytes_dlv").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.bytes_dlv)).unwrap();

    wr.write_str(",").unwrap();
    escape_str(wr, "ray_id").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, &log.ray_id).unwrap();
    wr.write_str("}").unwrap();
}

#[test]
fn test_manual_serialize_vec_no_escape() {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);

    let json = String::from_utf8(wr).unwrap();
    assert_eq!(JSON_STR, &json[]);
}

#[bench]
fn bench_manual_serialize_vec_no_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);
    b.bytes = wr.len() as u64;

    b.iter(|| {
        wr.clear();
        manual_serialize_no_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_serialize_vec_escape() {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);

    let json = String::from_utf8(wr).unwrap();
    assert_eq!(JSON_STR, &json[]);
}

#[bench]
fn bench_manual_serialize_vec_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);
    b.bytes = wr.len() as u64;

    b.iter(|| {
        wr.clear();

        manual_serialize_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_serialize_my_mem_writer0_no_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1000);
    manual_serialize_no_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(JSON_STR, &json[]);
}

#[bench]
fn bench_manual_serialize_my_mem_writer0_no_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_serialize_no_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_serialize_my_mem_writer0_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(JSON_STR, &json[]);
}

#[bench]
fn bench_manual_serialize_my_mem_writer0_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_serialize_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_serialize_my_mem_writer1_no_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(JSON_STR, &json[]);
}

#[bench]
fn bench_manual_serialize_my_mem_writer1_no_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_serialize_no_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_serialize_my_mem_writer1_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(JSON_STR, &json[]);
}

#[bench]
fn bench_manual_serialize_my_mem_writer1_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_serialize_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_serialize_escape(&mut wr, &log);
    });
}

#[bench]
fn bench_deserializer(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        let _log: Log = json::from_str(JSON_STR).unwrap();
    });
}
