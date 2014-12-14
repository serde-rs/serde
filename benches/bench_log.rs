#![feature(phase, macro_rules)]
#![allow(non_camel_case_types)]

#[phase(plugin)]
extern crate serde_macros;

extern crate serde;
extern crate serialize;
extern crate test;

use std::io;
use std::io::ByRefWriter;
use test::Bencher;

use serde::de;
use serde::json::ser::escape_str;
use serde::json;
use serde::ser::Serialize;
use serde::ser;

use serialize::Encodable;

#[deriving(Show, PartialEq, Encodable, Decodable)]
#[deriving_serialize]
#[deriving_deserialize]
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

#[deriving(Copy, Show, PartialEq, FromPrimitive)]
enum HttpProtocol {
    HTTP_PROTOCOL_UNKNOWN,
    HTTP10,
    HTTP11,
}

impl<S: serialize::Encoder<E>, E> serialize::Encodable<S, E> for HttpProtocol {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: ::serialize::Decoder<E>, E> serialize::Decodable<D, E> for HttpProtocol {
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

#[deriving(Copy, Show, PartialEq, FromPrimitive)]
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

impl<S: serialize::Encoder<E>, E> serialize::Encodable<S, E> for HttpMethod {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: ::serialize::Decoder<E>, E> serialize::Decodable<D, E> for HttpMethod {
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

#[deriving(Copy, Show, PartialEq, FromPrimitive)]
enum CacheStatus {
    CACHESTATUS_UNKNOWN,
    Miss,
    Expired,
    Hit,
}

impl<S: serialize::Encoder<E>, E> serialize::Encodable<S, E> for CacheStatus {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: ::serialize::Decoder<E>, E> serialize::Decodable<D, E> for CacheStatus {
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

#[deriving(Show, PartialEq, Encodable, Decodable)]
#[deriving_serialize]
#[deriving_deserialize]
struct Origin {
    ip: String,
    port: u32,
    hostname: String,
    protocol: OriginProtocol,
}

#[deriving(Copy, Show, PartialEq, FromPrimitive)]
enum OriginProtocol {
    ORIGIN_PROTOCOL_UNKNOWN,
    HTTP,
    HTTPS,
}

impl<S: serialize::Encoder<E>, E> serialize::Encodable<S, E> for OriginProtocol {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: ::serialize::Decoder<E>, E> serialize::Decodable<D, E> for OriginProtocol {
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

#[deriving(Copy, Show, PartialEq, FromPrimitive)]
enum ZonePlan {
    ZONEPLAN_UNKNOWN,
    FREE,
    PRO,
    BIZ,
    ENT,
}

impl<S: serialize::Encoder<E>, E> serialize::Encodable<S, E> for ZonePlan {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: ::serialize::Decoder<E>, E> serialize::Decodable<D, E> for ZonePlan {
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

#[deriving(Copy, Show, PartialEq, FromPrimitive)]
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

impl<S: serialize::Encoder<E>, E> serialize::Encodable<S, E> for Country {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        (*self as uint).encode(s)
    }
}

impl<D: ::serialize::Decoder<E>, E> serialize::Decodable<D, E> for Country {
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

#[deriving(Show, PartialEq, Encodable, Decodable)]
#[deriving_serialize]
#[deriving_deserialize]
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
)

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
)

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
    use serialize::Encodable;

    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);

    {
        let mut encoder = serialize::json::Encoder::new(&mut wr as &mut Writer);
        log.encode(&mut encoder).unwrap();
    }

    assert_eq!(wr.as_slice(), JSON_STR.as_bytes());
}

#[bench]
fn bench_encoder(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);

    {
        let mut encoder = serialize::json::Encoder::new(&mut wr as &mut Writer);
        log.encode(&mut encoder).unwrap();
    }

    b.bytes = wr.len() as u64;

    b.iter(|| {
        wr.clear();

        let mut encoder = serialize::json::Encoder::new(&mut wr as &mut Writer);
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

fn manual_no_escape<W: Writer>(wr: &mut W, log: &Log) {
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

fn manual_escape<W: Writer>(wr: &mut W, log: &Log) {
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
fn test_manual_vec_no_escape() {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_no_escape(&mut wr, &log);

    let json = String::from_utf8(wr).unwrap();
    assert_eq!(JSON_STR, json.as_slice());
}

#[bench]
fn bench_manual_vec_no_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_no_escape(&mut wr, &log);
    b.bytes = wr.len() as u64;

    b.iter(|| {
        wr.clear();
        manual_no_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_vec_escape() {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_escape(&mut wr, &log);

    let json = String::from_utf8(wr).unwrap();
    assert_eq!(JSON_STR, json.as_slice());
}

#[bench]
fn bench_manual_vec_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = Vec::with_capacity(1024);
    manual_escape(&mut wr, &log);
    b.bytes = wr.len() as u64;

    b.iter(|| {
        wr.clear();

        manual_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_my_mem_writer0_no_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1000);
    manual_no_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(JSON_STR, json.as_slice());
}

#[bench]
fn bench_manual_my_mem_writer0_no_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    manual_no_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_no_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_my_mem_writer0_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    manual_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(JSON_STR, json.as_slice());
}

#[bench]
fn bench_manual_my_mem_writer0_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter0::with_capacity(1024);
    manual_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_my_mem_writer1_no_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_no_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(JSON_STR, json.as_slice());
}

#[bench]
fn bench_manual_my_mem_writer1_no_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_no_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_no_escape(&mut wr, &log);
    });
}

#[test]
fn test_manual_my_mem_writer1_escape() {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_escape(&mut wr, &log);

    let json = String::from_utf8(wr.buf).unwrap();
    assert_eq!(JSON_STR, json.as_slice());
}

#[bench]
fn bench_manual_my_mem_writer1_escape(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_escape(&mut wr, &log);
    b.bytes = wr.buf.len() as u64;

    b.iter(|| {
        wr.buf.clear();

        manual_escape(&mut wr, &log);
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
    let json = serialize::json::from_str(JSON_STR).unwrap();
    let mut decoder = serialize::json::Decoder::new(json);
    let log: Log = serialize::Decodable::decode(&mut decoder).unwrap();
    assert_eq!(log, Log::new());
}

#[bench]
fn bench_decoder(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        let json = serialize::json::from_str(JSON_STR).unwrap();
        let mut decoder = serialize::json::Decoder::new(json);
        let _log: Log = serialize::Decodable::decode(&mut decoder).unwrap();
    });
}

#[test]
fn test_deserializer() {
    let log: Log = json::from_str(JSON_STR).unwrap();
    assert_eq!(log, Log::new());
}

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

    //b.iter(|| {
        for _ in range(0i, 10000) {
        let _log: Log = json::from_str(s).unwrap();
        }
    //});
}
