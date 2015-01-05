#![feature(phase, macro_rules, old_orphan_check)]
#![allow(non_camel_case_types)]

#[phase(plugin)]
extern crate serde_macros;

extern crate serde;
extern crate "rustc-serialize" as rustc_serialize;
extern crate test;

use std::io::ByRefWriter;
use std::io::extensions::Bytes;
use std::io;
use std::num::FromPrimitive;
use test::Bencher;

use serde::de;
use serde::json::ser::escape_str;
use serde::json;
use serde::ser::Serialize;
use serde::ser;

use rustc_serialize::Encodable;

#[derive(Show, PartialEq, RustcEncodable, RustcDecodable)]
#[derive_serialize]
#[derive_deserialize]
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

#[derive(Copy, Show, PartialEq, FromPrimitive)]
enum HttpProtocol {
    HTTP_PROTOCOL_UNKNOWN,
    HTTP10,
    HTTP11,
}

impl<S: rustc_serialize::Encoder<E>, E> rustc_serialize::Encodable<S, E> for HttpProtocol {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: rustc_serialize::Decoder<E>, E> rustc_serialize::Decodable<D, E> for HttpProtocol {
    fn decode(d: &mut D) -> Result<HttpProtocol, E> {
        match FromPrimitive::from_uint(try!(d.read_uint())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from uint")),
        }
    }
}

impl<S: ser::Serializer<E>, E> ser::Serialize<S, E> for HttpProtocol {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

impl<D: de::Deserializer<E>, E> de::Deserialize<D, E> for HttpProtocol {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<HttpProtocol, E> {
        d.expect_from_primitive(token)
    }
}

#[derive(Copy, Show, PartialEq, FromPrimitive)]
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

impl<S: rustc_serialize::Encoder<E>, E> rustc_serialize::Encodable<S, E> for HttpMethod {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: rustc_serialize::Decoder<E>, E> rustc_serialize::Decodable<D, E> for HttpMethod {
    fn decode(d: &mut D) -> Result<HttpMethod, E> {
        match FromPrimitive::from_uint(try!(d.read_uint())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from uint")),
        }
    }
}

impl<S: ser::Serializer<E>, E> ser::Serialize<S, E> for HttpMethod {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

impl<D: de::Deserializer<E>, E> de::Deserialize<D, E> for HttpMethod {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<HttpMethod, E> {
        d.expect_from_primitive(token)
    }
}

#[derive(Copy, Show, PartialEq, FromPrimitive)]
enum CacheStatus {
    CACHESTATUS_UNKNOWN,
    Miss,
    Expired,
    Hit,
}

impl<S: rustc_serialize::Encoder<E>, E> rustc_serialize::Encodable<S, E> for CacheStatus {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: rustc_serialize::Decoder<E>, E> rustc_serialize::Decodable<D, E> for CacheStatus {
    fn decode(d: &mut D) -> Result<CacheStatus, E> {
        match FromPrimitive::from_uint(try!(d.read_uint())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from uint")),
        }
    }
}

impl<S: ser::Serializer<E>, E> ser::Serialize<S, E> for CacheStatus {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

impl<D: de::Deserializer<E>, E> de::Deserialize<D, E> for CacheStatus {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<CacheStatus, E> {
        d.expect_from_primitive(token)
    }
}

#[derive(Show, PartialEq, RustcEncodable, RustcDecodable)]
#[derive_serialize]
#[derive_deserialize]
struct Origin {
    ip: String,
    port: u32,
    hostname: String,
    protocol: OriginProtocol,
}

#[derive(Copy, Show, PartialEq, FromPrimitive)]
enum OriginProtocol {
    ORIGIN_PROTOCOL_UNKNOWN,
    HTTP,
    HTTPS,
}

impl<S: rustc_serialize::Encoder<E>, E> rustc_serialize::Encodable<S, E> for OriginProtocol {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: rustc_serialize::Decoder<E>, E> rustc_serialize::Decodable<D, E> for OriginProtocol {
    fn decode(d: &mut D) -> Result<OriginProtocol, E> {
        match FromPrimitive::from_uint(try!(d.read_uint())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from uint")),
        }
    }
}

impl<S: ser::Serializer<E>, E> ser::Serialize<S, E> for OriginProtocol {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

impl<D: de::Deserializer<E>, E> de::Deserialize<D, E> for OriginProtocol {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<OriginProtocol, E> {
        d.expect_from_primitive(token)
    }
}

#[derive(Copy, Show, PartialEq, FromPrimitive)]
enum ZonePlan {
    ZONEPLAN_UNKNOWN,
    FREE,
    PRO,
    BIZ,
    ENT,
}

impl<S: rustc_serialize::Encoder<E>, E> rustc_serialize::Encodable<S, E> for ZonePlan {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: rustc_serialize::Decoder<E>, E> rustc_serialize::Decodable<D, E> for ZonePlan {
    fn decode(d: &mut D) -> Result<ZonePlan, E> {
        match FromPrimitive::from_uint(try!(d.read_uint())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from uint")),
        }
    }
}

impl<S: ser::Serializer<E>, E> ser::Serialize<S, E> for ZonePlan {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

impl<D: de::Deserializer<E>, E> de::Deserialize<D, E> for ZonePlan {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<ZonePlan, E> {
        d.expect_from_primitive(token)
    }
}

#[derive(Copy, Show, PartialEq, FromPrimitive)]
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

impl<S: rustc_serialize::Encoder<E>, E> rustc_serialize::Encodable<S, E> for Country {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: rustc_serialize::Decoder<E>, E> rustc_serialize::Decodable<D, E> for Country {
    fn decode(d: &mut D) -> Result<Country, E> {
        match FromPrimitive::from_uint(try!(d.read_uint())) {
            Some(value) => Ok(value),
            None => Err(d.error("cannot convert from uint")),
        }
    }
}

impl<S: ser::Serializer<E>, E> ser::Serialize<S, E> for Country {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

impl<D: de::Deserializer<E>, E> de::Deserialize<D, E> for Country {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<Country, E> {
        d.expect_from_primitive(token)
    }
}

#[derive(Show, PartialEq, RustcEncodable, RustcDecodable)]
#[derive_serialize]
#[derive_deserialize]
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
    pub fn with_capacity(cap: uint) -> MyMemWriter0 {
        MyMemWriter0 {
            buf: Vec::with_capacity(cap)
        }
    }
}


impl Writer for MyMemWriter0 {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::IoResult<()> {
        self.buf.push_all(buf);
        Ok(())
    }
}

struct MyMemWriter1 {
    buf: Vec<u8>,
}

impl MyMemWriter1 {
    pub fn with_capacity(cap: uint) -> MyMemWriter1 {
        MyMemWriter1 {
            buf: Vec::with_capacity(cap)
        }
    }
}

// LLVM isn't yet able to lower `Vec::push_all` into a memcpy, so this helps
// MemWriter eke out that last bit of performance.
#[inline]
fn push_all_bytes(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // we would have failed if `reserve` overflowed.
        dst.set_len(dst_len + src_len);

        ::std::ptr::copy_nonoverlapping_memory(
            dst.as_mut_ptr().offset(dst_len as int),
            src.as_ptr(),
            src_len);
    }
}

impl Writer for MyMemWriter1 {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::IoResult<()> {
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

    assert_eq!(wr.as_slice(), JSON_STR.as_bytes());
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
    let json = json::to_vec(&log);
    assert_eq!(json, JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log);
    b.bytes = json.len() as u64;

    b.iter(|| {
        let _ = json::to_vec(&log);
    });
}

#[test]
fn test_serializer_vec() {
    let log = Log::new();
    let wr = Vec::with_capacity(1024);
    let mut serializer = json::Serializer::new(wr);
    log.serialize(&mut serializer).unwrap();

    let json = serializer.unwrap();
    assert_eq!(json.as_slice(), JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer_vec(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log);
    b.bytes = json.len() as u64;

    let mut wr = Vec::with_capacity(1024);

    b.iter(|| {
        wr.clear();

        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
        let _json = serializer.unwrap();
    });
}

#[bench]
fn bench_serializer_slice(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log);
    b.bytes = json.len() as u64;

    let mut buf = [0; 1024];

    b.iter(|| {
        for item in buf.iter_mut(){ *item = 0; }
        let mut wr = std::io::BufWriter::new(&mut buf);

        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
        let _json = serializer.unwrap();
    });
}

#[test]
fn test_serializer_my_mem_writer0() {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);

    {
        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
    }

    assert_eq!(wr.buf.as_slice(), JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer_my_mem_writer0(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log);
    b.bytes = json.len() as u64;

    let mut wr = MyMemWriter0::with_capacity(1024);

    b.iter(|| {
        wr.buf.clear();

        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
        let _json = serializer.unwrap();
    });
}

#[test]
fn test_serializer_my_mem_writer1() {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);

    {
        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
    }

    assert_eq!(wr.buf.as_slice(), JSON_STR.as_bytes());
}

#[bench]
fn bench_serializer_my_mem_writer1(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log);
    b.bytes = json.len() as u64;

    let mut wr = MyMemWriter1::with_capacity(1024);

    b.iter(|| {
        wr.buf.clear();

        let mut serializer = json::Serializer::new(wr.by_ref());
        log.serialize(&mut serializer).unwrap();
        let _json = serializer.unwrap();
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
    (write!(wr, "{}", log.zone_plan as uint)).unwrap();

    wr.write_str(",\"http\":{\"protocol\":").unwrap();
    (write!(wr, "{}", log.http.protocol as uint)).unwrap();
    wr.write_str(",\"status\":").unwrap();
    (write!(wr, "{}", log.http.status)).unwrap();
    wr.write_str(",\"host_status\":").unwrap();
    (write!(wr, "{}", log.http.host_status)).unwrap();
    wr.write_str(",\"up_status\":").unwrap();
    (write!(wr, "{}", log.http.up_status)).unwrap();
    wr.write_str(",\"method\":").unwrap();
    (write!(wr, "{}", log.http.method as uint)).unwrap();
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
    (write!(wr, "{}", log.origin.protocol as uint)).unwrap();

    wr.write_str("},\"country\":").unwrap();
    (write!(wr, "{}", log.country as uint)).unwrap();
    wr.write_str(",\"cache_status\":").unwrap();
    (write!(wr, "{}", log.cache_status as uint)).unwrap();
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
    (write!(wr, "{}", log.zone_plan as int)).unwrap();

    wr.write_str(",").unwrap();
    escape_str(wr, "http").unwrap();
    wr.write_str(":{").unwrap();
    escape_str(wr, "protocol").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.http.protocol as uint)).unwrap();
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
    (write!(wr, "{}", log.http.method as uint)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "content_type").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, log.http.content_type.as_slice()).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "user_agent").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, log.http.user_agent.as_slice()).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "referer").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, log.http.referer.as_slice()).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "request_uri").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, log.http.request_uri.as_slice()).unwrap();

    wr.write_str("},").unwrap();
    escape_str(wr, "origin").unwrap();
    wr.write_str(":{").unwrap();

    escape_str(wr, "ip").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, log.origin.ip.as_slice()).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "port").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.origin.port)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "hostname").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, log.origin.hostname.as_slice()).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "protocol").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.origin.protocol as uint)).unwrap();

    wr.write_str("},").unwrap();
    escape_str(wr, "country").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.country as uint)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "cache_status").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.cache_status as uint)).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "server_ip").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, log.server_ip.as_slice()).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "server_name").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, log.server_name.as_slice()).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "remote_ip").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, log.remote_ip.as_slice()).unwrap();
    wr.write_str(",").unwrap();
    escape_str(wr, "bytes_dlv").unwrap();
    wr.write_str(":").unwrap();
    (write!(wr, "{}", log.bytes_dlv)).unwrap();

    wr.write_str(",").unwrap();
    escape_str(wr, "ray_id").unwrap();
    wr.write_str(":").unwrap();
    escape_str(wr, log.ray_id.as_slice()).unwrap();
    wr.write_str("}").unwrap();
}

#[test]
fn test_manual_serialize_vec_no_escape() {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_serialize_no_escape(&mut wr, &log);

    let json = String::from_utf8(wr).unwrap();
    assert_eq!(JSON_STR, json.as_slice());
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
    assert_eq!(JSON_STR, json.as_slice());
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
    assert_eq!(JSON_STR, json.as_slice());
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
    assert_eq!(JSON_STR, json.as_slice());
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
    assert_eq!(JSON_STR, json.as_slice());
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
    assert_eq!(JSON_STR, json.as_slice());
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

fn direct<W: Writer>(wr: &mut W, log: &Log) {
    use serde::ser::Serializer;

    let mut serializer = json::Serializer::new(wr.by_ref());

    serializer.serialize_struct_start("Log", 12).unwrap();

    serializer.serialize_struct_elt("timestamp", &log.timestamp).unwrap();
    serializer.serialize_struct_elt("zone_id", &log.zone_id).unwrap();
    serializer.serialize_struct_elt("zone_plan", &(log.zone_plan as uint)).unwrap();
    serializer.serialize_struct_elt("http", &log.http).unwrap();
    serializer.serialize_struct_elt("origin", &log.origin).unwrap();
    serializer.serialize_struct_elt("country", &(log.country as uint)).unwrap();
    serializer.serialize_struct_elt("cache_status", &(log.cache_status as uint)).unwrap();
    serializer.serialize_struct_elt("server_ip", &log.server_ip.as_slice()).unwrap();
    serializer.serialize_struct_elt("server_name", &log.server_name.as_slice()).unwrap();
    serializer.serialize_struct_elt("remote_ip", &log.remote_ip.as_slice()).unwrap();
    serializer.serialize_struct_elt("bytes_dlv", &log.bytes_dlv).unwrap();
    serializer.serialize_struct_elt("ray_id", &log.ray_id.as_slice()).unwrap();

    serializer.serialize_struct_end().unwrap();
}

#[test]
fn test_direct_vec() {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    direct(&mut wr, &log);

    let json = String::from_utf8(wr).unwrap();
    assert_eq!(JSON_STR, json.as_slice());
}

#[bench]
fn bench_direct_vec(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    direct(&mut wr, &log);
    b.bytes = wr.len() as u64;

    b.iter(|| {
        let mut wr = Vec::with_capacity(1024);
        direct(&mut wr, &log);
    });
}

#[test]
fn test_direct_my_mem_writer0() {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    direct(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(JSON_STR, json.as_slice());
}

#[bench]
fn bench_direct_my_mem_writer0(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    direct(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        direct(&mut wr, &log);
    });
}

#[test]
fn test_direct_my_mem_writer1() {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    direct(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(JSON_STR, json.as_slice());
}

#[bench]
fn bench_direct_my_mem_writer1(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    direct(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        direct(&mut wr, &log);
    });
}

#[test]
fn test_decoder() {
    use rustc_serialize::json::Json;

    let json = Json::from_str(JSON_STR).unwrap();
    let mut decoder = rustc_serialize::json::Decoder::new(json);
    let log: Log = rustc_serialize::Decodable::decode(&mut decoder).unwrap();
    assert_eq!(log, Log::new());
}

#[bench]
fn bench_decoder(b: &mut Bencher) {
    use rustc_serialize::json::Json;

    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        let json = Json::from_str(JSON_STR).unwrap();
        let mut decoder = rustc_serialize::json::Decoder::new(json);
        let _log: Log = rustc_serialize::Decodable::decode(&mut decoder).unwrap();
    });
}

#[test]
fn test_deserializer() {
    let log: Log = json::from_str(JSON_STR).unwrap();
    assert_eq!(log, Log::new());
}

//////////////////////////////////////////////////////////////////////////////

#[inline]
fn manual_reader_ignore<R: Reader>(rdr: &mut R, buf: &mut [u8], key: &[u8]) {
    let buf = buf.slice_mut(0, key.len());
    rdr.read(buf).unwrap();
    assert_eq!(buf, key);
}

#[inline]
fn manual_reader_field<R: Reader>(rdr: &mut R, buf: &mut [u8], key: &[u8]) {
    let b = rdr.read_byte().unwrap();
    assert_eq!(b, b'"');

    manual_reader_ignore(rdr, buf, key);

    let b = rdr.read_byte().unwrap();
    assert_eq!(b, b'"');

    let b = rdr.read_byte().unwrap();
    assert_eq!(b, b':');
}

#[inline]
fn manual_reader_int<R: Reader>(rdr: &mut R, buf: &mut [u8], key: &[u8]) -> i64 {
    manual_reader_field(rdr, buf, key);

    let mut res = 0;

    loop {
        let byte = rdr.read_byte().unwrap();
        match byte {
            b'0' ... b'9' => {
                res *= 10;
                res += (byte as i64) - (b'0' as i64);
            }
            _ => { break; }
        }
    }

    res
}

#[inline]
fn manual_reader_string<R: Reader>(rdr: &mut R, buf: &mut [u8], key: &[u8]) -> String {
    manual_reader_field(rdr, buf, key);
    manual_reader_ignore(rdr, buf, b"\"");

    let mut idx = 0;

    loop {
        let byte = rdr.read_byte().unwrap();
        match byte {
            b'"' => { break; }
            byte => { buf[idx] = byte; }
        };

        idx += 1;
    }

    let b = rdr.read_byte().unwrap();
    assert!(b == b',' || b == b']' || b == b'}');

    String::from_utf8(buf.slice_to(idx).to_vec()).unwrap()
}

#[inline]
fn manual_reader_deserialize<R: Reader>(rdr: &mut R) -> Log {
    let mut buf = [0; 128];

    manual_reader_ignore(rdr, &mut buf, b"{");
    let timestamp = manual_reader_int(rdr, &mut buf, b"timestamp");
    let zone_id = manual_reader_int(rdr, &mut buf, b"zone_id");
    let zone_plan = manual_reader_int(rdr, &mut buf, b"zone_plan");

    manual_reader_field(rdr, &mut buf, b"http");
    manual_reader_ignore(rdr, &mut buf, b"{");

    let protocol = manual_reader_int(rdr, &mut buf, b"protocol");
    let status = manual_reader_int(rdr, &mut buf, b"status");
    let host_status = manual_reader_int(rdr, &mut buf, b"host_status");
    let up_status = manual_reader_int(rdr, &mut buf, b"up_status");
    let method = manual_reader_int(rdr, &mut buf, b"method");
    let content_type = manual_reader_string(rdr, &mut buf, b"content_type");
    let user_agent = manual_reader_string(rdr, &mut buf, b"user_agent");
    let referer = manual_reader_string(rdr, &mut buf, b"referer");
    let request_uri = manual_reader_string(rdr, &mut buf, b"request_uri");

    let http = Http {
        protocol: FromPrimitive::from_i64(protocol).unwrap(),
        status: FromPrimitive::from_i64(status).unwrap(),
        host_status: FromPrimitive::from_i64(host_status).unwrap(),
        up_status: FromPrimitive::from_i64(up_status).unwrap(),
        method: FromPrimitive::from_i64(method).unwrap(),
        content_type: content_type,
        user_agent: user_agent,
        referer: referer,
        request_uri: request_uri,
    };

    manual_reader_ignore(rdr, &mut buf, b",");
    manual_reader_field(rdr, &mut buf, b"origin");
    manual_reader_ignore(rdr, &mut buf, b"{");

    let ip = manual_reader_string(rdr, &mut buf, b"ip");
    let port = manual_reader_int(rdr, &mut buf, b"port");
    let hostname = manual_reader_string(rdr, &mut buf, b"hostname");
    let protocol = manual_reader_int(rdr, &mut buf, b"protocol");

    let origin = Origin {
        ip: ip,
        port: FromPrimitive::from_i64(port).unwrap(),
        hostname: hostname,
        protocol: FromPrimitive::from_i64(protocol).unwrap(),
    };

    manual_reader_ignore(rdr, &mut buf, b",");
    let country = manual_reader_int(rdr, &mut buf, b"country");
    let cache_status = manual_reader_int(rdr, &mut buf, b"cache_status");
    let server_ip = manual_reader_string(rdr, &mut buf, b"server_ip");
    let server_name = manual_reader_string(rdr, &mut buf, b"server_name");
    let remote_ip = manual_reader_string(rdr, &mut buf, b"remote_ip");
    let bytes_dlv = manual_reader_int(rdr, &mut buf, b"bytes_dlv");
    let ray_id = manual_reader_string(rdr, &mut buf, b"ray_id");

    Log {
        timestamp: timestamp,
        zone_id: FromPrimitive::from_i64(zone_id).unwrap(),
        zone_plan: FromPrimitive::from_i64(zone_plan).unwrap(),
        http: http,
        origin: origin,
        country: FromPrimitive::from_i64(country).unwrap(),
        cache_status: FromPrimitive::from_i64(cache_status).unwrap(),
        server_ip: server_ip,
        server_name: server_name,
        remote_ip: remote_ip,
        bytes_dlv: FromPrimitive::from_i64(bytes_dlv).unwrap(),
        ray_id: ray_id,
    }
}

//////////////////////////////////////////////////////////////////////////////

#[inline]
fn manual_iter_ignore<R: Iterator<Item=u8>>(mut rdr: R, buf: &mut [u8], key: &[u8]) {
    let buf = buf.slice_mut(0, key.len());

    for idx in range(0, key.len()) {
        buf[idx] = rdr.next().unwrap();
    }
    assert_eq!(buf, key);
}

#[inline]
fn manual_iter_field<R: Iterator<Item=u8>>(mut rdr: R, buf: &mut [u8], key: &[u8]) {
    let b = rdr.next().unwrap();
    assert_eq!(b, b'"');

    manual_iter_ignore(rdr.by_ref(), buf, key);

    let b = rdr.next().unwrap();
    assert_eq!(b, b'"');

    let b = rdr.next().unwrap();
    assert_eq!(b, b':');
}

#[inline]
fn manual_iter_int<R: Iterator<Item=u8>>(mut rdr: R, buf: &mut [u8], key: &[u8]) -> i64 {
    manual_iter_field(rdr.by_ref(), buf, key);

    let mut res = 0;

    loop {
        let byte = rdr.next().unwrap();
        match byte {
            b'0' ... b'9' => {
                res *= 10;
                res += (byte as i64) - (b'0' as i64);
            }
            _ => { break; }
        }
    }

    res
}

#[inline]
fn manual_iter_string<R: Iterator<Item=u8>>(mut rdr: R, buf: &mut [u8], key: &[u8]) -> String {
    manual_iter_field(rdr.by_ref(), buf, key);
    manual_iter_ignore(rdr.by_ref(), buf, b"\"");

    let mut idx = 0;

    loop {
        let byte = rdr.next().unwrap();
        match byte {
            b'"' => { break; }
            byte => { buf[idx] = byte; }
        };

        idx += 1;
    }

    let b = rdr.next().unwrap();
    assert!(b == b',' || b == b']' || b == b'}');

    String::from_utf8(buf.slice_to(idx).to_vec()).unwrap()
}

#[inline]
fn manual_iter_deserialize<R: Iterator<Item=u8>>(mut rdr: R) -> Log {
    let mut buf = [0u8; 128];

    manual_iter_ignore(rdr.by_ref(), &mut buf, b"{");
    let timestamp = manual_iter_int(rdr.by_ref(), &mut buf, b"timestamp");
    let zone_id = manual_iter_int(rdr.by_ref(), &mut buf, b"zone_id");
    let zone_plan = manual_iter_int(rdr.by_ref(), &mut buf, b"zone_plan");

    manual_iter_field(rdr.by_ref(), &mut buf, b"http");
    manual_iter_ignore(rdr.by_ref(), &mut buf, b"{");

    let protocol = manual_iter_int(rdr.by_ref(), &mut buf, b"protocol");
    let status = manual_iter_int(rdr.by_ref(), &mut buf, b"status");
    let host_status = manual_iter_int(rdr.by_ref(), &mut buf, b"host_status");
    let up_status = manual_iter_int(rdr.by_ref(), &mut buf, b"up_status");
    let method = manual_iter_int(rdr.by_ref(), &mut buf, b"method");
    let content_type = manual_iter_string(rdr.by_ref(), &mut buf, b"content_type");
    let user_agent = manual_iter_string(rdr.by_ref(), &mut buf, b"user_agent");
    let referer = manual_iter_string(rdr.by_ref(), &mut buf, b"referer");
    let request_uri = manual_iter_string(rdr.by_ref(), &mut buf, b"request_uri");

    let http = Http {
        protocol: FromPrimitive::from_i64(protocol).unwrap(),
        status: FromPrimitive::from_i64(status).unwrap(),
        host_status: FromPrimitive::from_i64(host_status).unwrap(),
        up_status: FromPrimitive::from_i64(up_status).unwrap(),
        method: FromPrimitive::from_i64(method).unwrap(),
        content_type: content_type,
        user_agent: user_agent,
        referer: referer,
        request_uri: request_uri,
    };

    manual_iter_ignore(rdr.by_ref(), &mut buf, b",");
    manual_iter_field(rdr.by_ref(), &mut buf, b"origin");
    manual_iter_ignore(rdr.by_ref(), &mut buf, b"{");

    let ip = manual_iter_string(rdr.by_ref(), &mut buf, b"ip");
    let port = manual_iter_int(rdr.by_ref(), &mut buf, b"port");
    let hostname = manual_iter_string(rdr.by_ref(), &mut buf, b"hostname");
    let protocol = manual_iter_int(rdr.by_ref(), &mut buf, b"protocol");

    let origin = Origin {
        ip: ip,
        port: FromPrimitive::from_i64(port).unwrap(),
        hostname: hostname,
        protocol: FromPrimitive::from_i64(protocol).unwrap(),
    };

    manual_iter_ignore(rdr.by_ref(), &mut buf, b",");
    let country = manual_iter_int(rdr.by_ref(), &mut buf, b"country");
    let cache_status = manual_iter_int(rdr.by_ref(), &mut buf, b"cache_status");
    let server_ip = manual_iter_string(rdr.by_ref(), &mut buf, b"server_ip");
    let server_name = manual_iter_string(rdr.by_ref(), &mut buf, b"server_name");
    let remote_ip = manual_iter_string(rdr.by_ref(), &mut buf, b"remote_ip");
    let bytes_dlv = manual_iter_int(rdr.by_ref(), &mut buf, b"bytes_dlv");
    let ray_id = manual_iter_string(rdr.by_ref(), &mut buf, b"ray_id");

    Log {
        timestamp: timestamp,
        zone_id: FromPrimitive::from_i64(zone_id).unwrap(),
        zone_plan: FromPrimitive::from_i64(zone_plan).unwrap(),
        http: http,
        origin: origin,
        country: FromPrimitive::from_i64(country).unwrap(),
        cache_status: FromPrimitive::from_i64(cache_status).unwrap(),
        server_ip: server_ip,
        server_name: server_name,
        remote_ip: remote_ip,
        bytes_dlv: FromPrimitive::from_i64(bytes_dlv).unwrap(),
        ray_id: ray_id,
    }
}

//////////////////////////////////////////////////////////////////////////////

#[bench]
fn bench_deserializer(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        let _log: Log = json::from_str(JSON_STR).unwrap();
    });
}

#[bench]
fn bench_deserializers(b: &mut Bencher) {
    let s = r#"{"timestamp":25469139677502,"zone_id":123456,"zone_plan":1,"http":{"protocol":2,"status":200,"host_status":503,"up_status":520,"method":1,"content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":2},"country":238,"cache_status":3,"server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    b.bytes = s.len() as u64;

    for _ in range(0i, 10000) {
        let _log: Log = json::from_str(s).unwrap();
    }
}

//////////////////////////////////////////////////////////////////////////////

#[test]
fn test_reader_manual_deserializer() {
    let mut rdr = JSON_STR.as_bytes();
    let log = manual_reader_deserialize(&mut rdr);

    assert_eq!(log, Log::new());
}

#[bench]
fn bench_reader_manual_reader_deserializer(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        let mut rdr = JSON_STR.as_bytes();
        let _ = manual_reader_deserialize(&mut rdr);
    });
}

#[bench]
fn bench_reader_manual_reader_deserializers(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    for _ in range(0i, 100000) {
        let mut rdr = JSON_STR.as_bytes();
        let _ = manual_reader_deserialize(&mut rdr);
    }
}

//////////////////////////////////////////////////////////////////////////////

#[test]
fn test_iter_manual_iter_deserializer() {
    let log = manual_iter_deserialize(JSON_STR.bytes());

    assert_eq!(log, Log::new());
}

#[bench]
fn bench_iter_manual_iter_deserializer(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        let _ = manual_iter_deserialize(JSON_STR.bytes());
    });
}

#[bench]
fn bench_iter_manual_iter_deserializers(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    for _ in range(0i, 10000) {
        let _ = manual_iter_deserialize(JSON_STR.bytes());
    }
}

//////////////////////////////////////////////////////////////////////////////

#[test]
fn test_iter_manual_reader_as_iter_deserializer() {
    let mut rdr = JSON_STR.as_bytes();
    let iter = Bytes::new(&mut rdr)
        .map(|x| x.unwrap());

    let log = manual_iter_deserialize(iter);

    assert_eq!(log, Log::new());
}

#[bench]
fn bench_iter_manual_reader_as_iter_deserializer(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        let mut rdr = JSON_STR.as_bytes();
        let iter = Bytes::new(&mut rdr)
            .map(|x| x.unwrap());

        let _ = manual_iter_deserialize(iter);
    });
}

#[bench]
fn bench_iter_manual_reader_as_iter_deserializers(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    for _ in range(0i, 10000) {
        let mut rdr = JSON_STR.as_bytes();
        let iter = Bytes::new(&mut rdr)
            .map(|x| x.unwrap());

        let _ = manual_iter_deserialize(iter);
    }
}
