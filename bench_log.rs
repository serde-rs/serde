#![allow(non_camel_case_types)]

extern crate serialize;
extern crate test;
extern crate time;

use test::Bencher;

use json;
use ser::Serializable;
use ser;

#[deriving(Encodable, Decodable)]
#[deriving_serializable]
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

#[deriving(Encodable, Decodable)]
enum HttpProtocol {
    HTTP_PROTOCOL_UNKNOWN,
    HTTP10,
    HTTP11,
}

impl ser::Serializable for HttpProtocol {
    #[inline]
    fn serialize<
        S: ser::Serializer<E>,
        E
    >(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

#[deriving(Encodable, Decodable)]
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

impl ser::Serializable for HttpMethod {
    #[inline]
    fn serialize<
        S: ser::Serializer<E>,
        E
    >(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

#[deriving(Encodable, Decodable)]
enum CacheStatus {
    CACHESTATUS_UNKNOWN,
    Miss,
    Expired,
    Hit,
}

impl ser::Serializable for CacheStatus {
    #[inline]
    fn serialize<
        S: ser::Serializer<E>,
        E
    >(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

#[deriving(Encodable, Decodable)]
#[deriving_serializable]
struct Origin {
    ip: String,
    port: u32,
    hostname: String,
    protocol: OriginProtocol,
}

#[deriving(Encodable, Decodable)]
enum OriginProtocol {
    ORIGIN_PROTOCOL_UNKNOWN,
    HTTP,
    HTTPS,
}

impl ser::Serializable for OriginProtocol {
    #[inline]
    fn serialize<
        S: ser::Serializer<E>,
        E
    >(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

#[deriving(Encodable, Decodable)]
enum ZonePlan {
    ZONEPLAN_UNKNOWN,
    FREE,
    PRO,
    BIZ,
    ENT,
}

impl ser::Serializable for ZonePlan {
    #[inline]
    fn serialize<
        S: ser::Serializer<E>,
        E
    >(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

#[deriving(Encodable, Decodable)]
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

impl ser::Serializable for Country {
    #[inline]
    fn serialize<
        S: ser::Serializer<E>,
        E
    >(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
    }
}

#[deriving(Encodable, Decodable)]
#[deriving_serializable]
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

#[bench]
fn bench_encoder(b: &mut Bencher) {
    let log = Log::new();
    let json = serialize::json::Encoder::str_encode(&log);
    let _len = json.len();

    b.iter(|| {
        let _ = serialize::json::Encoder::str_encode(&log);
    });
}

#[bench]
fn bench_serializer(b: &mut Bencher) {
    let log = Log::new();
    let _json = json::to_str(&log).unwrap();

    b.iter(|| {
        let _json = json::to_str(&log).unwrap();
    });
}

#[bench]
fn bench_decoder(b: &mut Bencher) {
    let s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    b.iter(|| {
        let json = serialize::json::from_str(s).unwrap();
        let mut decoder = serialize::json::Decoder::new(json);
        let _log: Log = serialize::Decodable::decode(&mut decoder).unwrap();
    });

}
