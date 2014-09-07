#![allow(non_camel_case_types)]
#![feature(phase, macro_rules)]

extern crate serde2;
extern crate serialize;
extern crate test;
extern crate time;

#[phase(plugin)]
extern crate serde2_macros;

//use std::io;
//use std::io::MemWriter;
use test::Bencher;

use serde2::json;
//use serde2::de;
use serde2::Serialize;
use serde2::ser;

#[deriving(Encodable, Decodable)]
#[deriving_serializable]
//#[deriving_deserializable]
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

#[deriving(Show, Encodable, Decodable, FromPrimitive)]
enum HttpProtocol {
    HTTP_PROTOCOL_UNKNOWN,
    HTTP10,
    HTTP11,
}

impl<S: ser::VisitorState<R>, R> ser::Serialize<S, R> for HttpProtocol {
    #[inline]
    fn serialize(&self, s: &mut S) -> R {
        s.visit_uint(*self as uint)
    }
}

/*
impl<D: de::Deserializer<E>, E> de::Deserializable<D, E> for HttpProtocol {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<HttpProtocol, E> {
        d.expect_from_primitive(token)
    }
}
*/

#[deriving(Show, Encodable, Decodable, FromPrimitive)]
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

impl<S: ser::VisitorState<R>, R> ser::Serialize<S, R> for HttpMethod {
    #[inline]
    fn serialize(&self, s: &mut S) -> R {
        s.visit_uint(*self as uint)
    }
}

/*
impl<D: de::Deserializer<E>, E> de::Deserializable<D, E> for HttpMethod {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<HttpMethod, E> {
        d.expect_from_primitive(token)
    }
}
*/

#[deriving(Show, Encodable, Decodable, FromPrimitive)]
enum CacheStatus {
    CACHESTATUS_UNKNOWN,
    Miss,
    Expired,
    Hit,
}

impl<S: ser::VisitorState<R>, R> ser::Serialize<S, R> for CacheStatus {
    #[inline]
    fn serialize(&self, s: &mut S) -> R {
        s.visit_uint(*self as uint)
    }
}

/*
impl<D: de::Deserializer<E>, E> de::Deserializable<D, E> for CacheStatus {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<CacheStatus, E> {
        d.expect_from_primitive(token)
    }
}
*/

#[deriving(Encodable, Decodable)]
#[deriving_serializable]
//#[deriving_deserializable]
struct Origin {
    ip: String,
    port: u32,
    hostname: String,
    protocol: OriginProtocol,
}

#[deriving(Show, Encodable, Decodable, FromPrimitive)]
enum OriginProtocol {
    ORIGIN_PROTOCOL_UNKNOWN,
    HTTP,
    HTTPS,
}

impl<S: ser::VisitorState<R>, R> ser::Serialize<S, R> for OriginProtocol {
    #[inline]
    fn serialize(&self, s: &mut S) -> R {
        s.visit_uint(*self as uint)
    }
}

/*
impl<D: de::Deserializer<E>, E> de::Deserializable<D, E> for OriginProtocol {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<OriginProtocol, E> {
        d.expect_from_primitive(token)
    }
}
*/

#[deriving(Show, Encodable, Decodable, FromPrimitive)]
enum ZonePlan {
    ZONEPLAN_UNKNOWN,
    FREE,
    PRO,
    BIZ,
    ENT,
}

impl<S: ser::VisitorState<R>, R> ser::Serialize<S, R> for ZonePlan {
    #[inline]
    fn serialize(&self, s: &mut S) -> R {
        s.visit_uint(*self as uint)
    }
}

/*
impl<D: de::Deserializer<E>, E> de::Deserializable<D, E> for ZonePlan {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<ZonePlan, E> {
        d.expect_from_primitive(token)
    }
}
*/

#[deriving(Show, Encodable, Decodable, FromPrimitive)]
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

impl<S: ser::VisitorState<R>, R> ser::Serialize<S, R> for Country {
    #[inline]
    fn serialize(&self, s: &mut S) -> R {
        s.visit_uint(*self as uint)
    }
}

/*
impl<D: de::Deserializer<E>, E> de::Deserializable<D, E> for Country {
    #[inline]
    fn deserialize_token(d: &mut D, token: de::Token) -> Result<Country, E> {
        d.expect_from_primitive(token)
    }
}
*/

#[deriving(Encodable, Decodable)]
//#[deriving_serializable]
//#[deriving_deserializable]
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
            timestamp: time::precise_time_ns() as i64,
            zone_id: 123456,
            zone_plan: FREE,
            http: Http {
                protocol: HTTP11,
                status: 200,
                host_status: 503,
                up_status: 520,
                method: GET,
                content_type: "text/html".to_string(),
                user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36".to_string(),
                referer: "https://www.cloudflare.com/".to_string(),
                request_uri: "/cdn-cgi/trace".to_string(),
            },
            origin: Origin {
                ip: "1.2.3.4".to_string(),
                port: 8000,
                hostname: "www.example.com".to_string(),
                protocol: HTTPS,
            },
            country: US,
            cache_status: Hit,
            server_ip: "192.168.1.1".to_string(),
            server_name: "metal.cloudflare.com".to_string(),
            remote_ip: "10.1.2.3".to_string(),
            bytes_dlv: 123456,
            ray_id: "10c73629cce30078-LAX".to_string(),
        }
    }
}


impl <S: ::serde2::VisitorState<R>, R> ::serde2::Serialize<S, R> for Log {
    #[inline]
    fn serialize(&self, s: &mut S) -> R {
        struct Log3033<'a> {
            value: &'a Log,
            state: u8,
        }

        impl<'a, S: ::serde2::VisitorState<R>, R> ::serde2::Visitor<S, R> for Log3033<'a> {
            #[inline]
            fn visit(&mut self, s: &mut S) -> Option<R> {
                match self.state {
                    0 => {
                        self.state += 1;
                        Some(s.visit_map_elt(true, "timestamp",
                                             &self.value.timestamp))
                    }
                    1 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "zone_id", &self.value.zone_id))
                    }
                    2 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "zone_plan",
                                             &self.value.zone_plan))
                    }
                    3 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "http", &self.value.http))
                    }
                    4 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "origin", &self.value.origin))
                    }
                    5 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "country", &self.value.country))
                    }
                    6 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "cache_status",
                                             &self.value.cache_status))
                    }
                    7 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "server_ip",
                                             &self.value.server_ip))
                    }
                    8 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "server_name",
                                             &self.value.server_name))
                    }
                    9 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "remote_ip",
                                             &self.value.remote_ip))
                    }
                    10 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "bytes_dlv",
                                             &self.value.bytes_dlv))
                    }
                    11 => {
                        self.state += 1;
                        Some(s.visit_map_elt(false, "ray_id", &self.value.ray_id))
                    }
                    _ => None,
                }
            }
        }

        s.visit_named_map("Log", Log3033{value: self, state: 0,})
    }
}


#[bench]
fn bench_encoder(b: &mut Bencher) {
    let log = Log::new();
    let json = serialize::json::encode(&log);
    b.bytes = json.len() as u64;

    b.iter(|| {
        //for _ in range(0u, 1000) {
        let _json = serialize::json::encode(&log);
        //}
    });
}

#[bench]
fn bench_serializer(b: &mut Bencher) {
    let log = Log::new();
    let json = json::to_vec(&log).unwrap();
    b.bytes = json.len() as u64;

    b.iter(|| {
        //for _ in range(0u, 1000) {
        let _json = json::to_vec(&log).unwrap();
        //}
    });
}

/*
#[bench]
fn bench_copy(b: &mut Bencher) {
    let s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let json = Vec::from_slice(s.as_bytes());
    b.bytes = json.len() as u64;

    b.iter(|| {
        let _json = Vec::from_slice(s.as_bytes());
    });
}

fn manual_no_escape<W: Writer>(mut wr: W, log: &Log) {
    wr.write_str("{\"timestamp\":").unwrap();
    (write!(wr, "{}", log.timestamp)).unwrap();
    wr.write_str(",\"zone_id\":").unwrap();
    (write!(wr, "{}", log.zone_id)).unwrap();
    wr.write_str(",\"zone_plan\":").unwrap();
    (write!(wr, "{}", log.zone_plan as int)).unwrap();

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
    json::escape_str(&mut wr, log.http.content_type.as_slice()).unwrap();
    wr.write_str(",\"user_agent\":").unwrap();
    json::escape_str(&mut wr, log.http.user_agent.as_slice()).unwrap();
    wr.write_str(",\"referer\":").unwrap();
    json::escape_str(&mut wr, log.http.referer.as_slice()).unwrap();
    wr.write_str(",\"request_uri\":").unwrap();
    json::escape_str(&mut wr, log.http.request_uri.as_slice()).unwrap();

    wr.write_str("},\"origin\":{\"port\":").unwrap();
    (write!(wr, "{}", log.origin.port)).unwrap();
    wr.write_str(",\"hostname\":").unwrap();
    json::escape_str(&mut wr, log.origin.hostname.as_slice()).unwrap();
    wr.write_str(",\"protocol\":").unwrap();
    (write!(wr, "{}", log.origin.protocol as uint)).unwrap();

    wr.write_str("},\"country\":").unwrap();
    (write!(wr, "{}", log.country as uint)).unwrap();
    wr.write_str(",\"cache_status\":").unwrap();
    (write!(wr, "{}", log.cache_status as uint)).unwrap();
    wr.write_str(",\"server_ip\":").unwrap();
    json::escape_str(&mut wr, log.server_ip.as_slice()).unwrap();
    wr.write_str(",\"server_name\":").unwrap();
    json::escape_str(&mut wr, log.server_name.as_slice()).unwrap();
    wr.write_str(",\"remote_ip\":").unwrap();
    json::escape_str(&mut wr, log.remote_ip.as_slice()).unwrap();
    wr.write_str(",\"bytes_dlv\":").unwrap();
    (write!(wr, "{}", log.bytes_dlv)).unwrap();

    wr.write_str(",\"ray_id\":").unwrap();
    json::escape_str(&mut wr, log.ray_id.as_slice()).unwrap();
    wr.write_str("}").unwrap();
}

fn manual_escape<W: Writer>(mut wr: W, log: &Log) {
    wr.write_str("{\"").unwrap();
    json::escape_str(&mut wr, "timestamp").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.timestamp)).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "zone_id").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.zone_id)).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "zone_plan").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.zone_plan as int)).unwrap();

    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "http").unwrap();
    wr.write_str("\":{\"").unwrap();
    json::escape_str(&mut wr, "protocol").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.http.protocol as uint)).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "status").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.http.status)).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "host_status").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.http.host_status)).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "up_status").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.http.up_status)).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "method").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.http.method as uint)).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "content_type").unwrap();
    wr.write_str("\":").unwrap();
    json::escape_str(&mut wr, log.http.content_type.as_slice()).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "user_agent").unwrap();
    wr.write_str("\":").unwrap();
    json::escape_str(&mut wr, log.http.user_agent.as_slice()).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "referer").unwrap();
    wr.write_str("\":").unwrap();
    json::escape_str(&mut wr, log.http.referer.as_slice()).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "request_uri").unwrap();
    wr.write_str("\":").unwrap();
    json::escape_str(&mut wr, log.http.request_uri.as_slice()).unwrap();

    wr.write_str("},\"").unwrap();
    json::escape_str(&mut wr, "origin").unwrap();
    wr.write_str("\":{\"").unwrap();
    json::escape_str(&mut wr, "port").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.origin.port)).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "hostname").unwrap();
    wr.write_str("\":").unwrap();
    json::escape_str(&mut wr, log.origin.hostname.as_slice()).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "protocol").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.origin.protocol as uint)).unwrap();

    wr.write_str("},\"").unwrap();
    json::escape_str(&mut wr, "country").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.country as uint)).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "cache_status").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.cache_status as uint)).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "server_ip").unwrap();
    wr.write_str("\":").unwrap();
    json::escape_str(&mut wr, log.server_ip.as_slice()).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "server_name").unwrap();
    wr.write_str("\":").unwrap();
    json::escape_str(&mut wr, log.server_name.as_slice()).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "remote_ip").unwrap();
    wr.write_str("\":").unwrap();
    json::escape_str(&mut wr, log.remote_ip.as_slice()).unwrap();
    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "bytes_dlv").unwrap();
    wr.write_str("\":").unwrap();
    (write!(wr, "{}", log.bytes_dlv)).unwrap();

    wr.write_str(",\"").unwrap();
    json::escape_str(&mut wr, "ray_id").unwrap();
    wr.write_str("\":").unwrap();
    json::escape_str(&mut wr, log.ray_id.as_slice()).unwrap();
    wr.write_str("}").unwrap();
}

#[bench]
fn bench_manual_mem_writer_no_escape(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MemWriter::with_capacity(1024);
    manual_no_escape(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MemWriter::with_capacity(1024);
        manual_no_escape(wr.by_ref(), &log);

        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

#[bench]
fn bench_manual_mem_writer_escape(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MemWriter::with_capacity(1024);
    manual_escape(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MemWriter::with_capacity(1024);
        manual_escape(wr.by_ref(), &log);
        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

#[bench]
fn bench_manual_my_mem_writer0_no_escape(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MyMemWriter0::with_capacity(1000);
    manual_no_escape(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MyMemWriter0::with_capacity(1024);
        manual_no_escape(wr.by_ref(), &log);

        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

#[bench]
fn bench_manual_my_mem_writer0_escape(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MemWriter::with_capacity(1024);
    manual_escape(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MyMemWriter0::with_capacity(1024);
        manual_escape(wr.by_ref(), &log);
        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

#[bench]
fn bench_manual_my_mem_writer1_no_escape(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MyMemWriter1::with_capacity(1000);
    manual_no_escape(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MyMemWriter1::with_capacity(1024);
        manual_no_escape(wr.by_ref(), &log);

        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

#[bench]
fn bench_manual_my_mem_writer1_escape(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MyMemWriter1::with_capacity(1024);
    manual_escape(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MyMemWriter1::with_capacity(1024);
        manual_escape(wr.by_ref(), &log);
        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

#[bench]
fn bench_manual_my_mem_writer2_no_escape(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MyMemWriter2::with_capacity(1000);
    manual_no_escape(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MyMemWriter2::with_capacity(1024);
        manual_no_escape(wr.by_ref(), &log);

        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

#[bench]
fn bench_manual_my_mem_writer2_escape(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MyMemWriter2::with_capacity(1024);
    manual_escape(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MyMemWriter2::with_capacity(1024);
        manual_escape(wr.by_ref(), &log);
        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}


#[bench]
fn bench_manual_my_mem_writer3_no_escape(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MyMemWriter3::with_capacity(1000);
    manual_no_escape(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MyMemWriter3::with_capacity(1024);
        manual_no_escape(wr.by_ref(), &log);

        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

#[bench]
fn bench_manual_my_mem_writer3_escape(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MyMemWriter3::with_capacity(1024);
    manual_escape(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MyMemWriter3::with_capacity(1024);
        manual_escape(wr.by_ref(), &log);
        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

fn direct<W: Writer>(wr: W, log: &Log) {
    use ser::VisitorState;

    let mut serializer = json::Serializer::new(wr);
    serializer.visit_struct_start("Log", 12).unwrap();

    serializer.visit_struct_elt("timestamp", &log.timestamp).unwrap();
    serializer.visit_struct_elt("zone_id", &log.zone_id).unwrap();
    serializer.serialize_struct_elt("zone_plan", &(log.zone_plan as uint)).unwrap();

    serializer.serialize_struct_start("Http", 9).unwrap();
    serializer.serialize_struct_elt("protocol", &(log.http.protocol as uint)).unwrap();
    serializer.serialize_struct_elt("status", &log.http.status).unwrap();
    serializer.serialize_struct_elt("host_status", &log.http.host_status).unwrap();
    serializer.serialize_struct_elt("up_status", &log.http.up_status).unwrap();
    serializer.serialize_struct_elt("method", &(log.http.method as uint)).unwrap();
    serializer.serialize_struct_elt("content_type", &log.http.content_type).unwrap();
    serializer.serialize_struct_elt("user_agent", &log.http.user_agent).unwrap();
    serializer.serialize_struct_elt("referer", &log.http.referer.as_slice()).unwrap();
    serializer.serialize_struct_elt("request_uri", &log.http.request_uri.as_slice()).unwrap();
    serializer.serialize_struct_end().unwrap();

    serializer.serialize_struct_start("Origin", 3).unwrap();
    serializer.serialize_struct_elt("port", &log.origin.port).unwrap();
    serializer.serialize_struct_elt("hostname", &log.origin.hostname.as_slice()).unwrap();
    serializer.serialize_struct_elt("protocol", &(log.origin.protocol as uint)).unwrap();
    serializer.serialize_struct_end().unwrap();

    serializer.serialize_struct_elt("country", &(log.country as uint)).unwrap();
    serializer.serialize_struct_elt("cache_status", &(log.cache_status as uint)).unwrap();
    serializer.serialize_struct_elt("server_ip", &log.server_ip.as_slice()).unwrap();
    serializer.serialize_struct_elt("server_name", &log.server_name.as_slice()).unwrap();
    serializer.serialize_struct_elt("remote_ip", &log.remote_ip.as_slice()).unwrap();
    serializer.serialize_struct_elt("bytes_dlv", &log.bytes_dlv).unwrap();
    serializer.serialize_struct_elt("ray_id", &log.ray_id.as_slice()).unwrap();

    serializer.serialize_struct_end().unwrap();
}

#[bench]
fn bench_direct_mem_writer(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MemWriter::with_capacity(1024);
    direct(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MemWriter::with_capacity(1024);
        direct(wr.by_ref(), &log);
        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

#[bench]
fn bench_direct_my_mem_writer0(b: &mut Bencher) {
    let log = Log::new();
    let _s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    let mut wr = MyMemWriter0::with_capacity(1024);
    direct(wr.by_ref(), &log);
    b.bytes = wr.unwrap().len() as u64;

    b.iter(|| {
        let mut wr = MyMemWriter0::with_capacity(1024);
        direct(wr.by_ref(), &log);
        let _json = wr.unwrap();

        //let _json = String::from_utf8(wr.unwrap()).unwrap();
        /*
        assert_eq!(_s, _json.as_slice());
        */
    });
}

#[bench]
fn bench_decoder(b: &mut Bencher) {
    let s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    b.bytes = s.len() as u64;

    b.iter(|| {
        let json = serialize::json::from_str(s).unwrap();
        let mut decoder = serialize::json::Decoder::new(json);
        let _log: Log = serialize::Decodable::decode(&mut decoder).unwrap();
    });
}

#[bench]
fn bench_deserializer(b: &mut Bencher) {
    let s = r#"{"timestamp":25469139677502,"zone_id":123456,"zone_plan":1,"http":{"protocol":2,"status":200,"host_status":503,"up_status":520,"method":1,"content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":2},"country":238,"cache_status":3,"server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    b.bytes = s.len() as u64;

    b.iter(|| {
        let _log: Log = json::from_str(s).unwrap();
    });
}
*/
