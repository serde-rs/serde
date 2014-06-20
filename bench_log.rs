#![allow(non_camel_case_types)]

extern crate serialize;
extern crate test;
extern crate time;

use std::io::MemWriter;
use test::Bencher;

use json;
use ser::Serializable;
use ser;

#[deriving(Encodable, Decodable)]
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

impl<S: ser::Serializer<E>, E> ser::Serializable<S, E> for Http {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        try!(s.serialize_struct_start("Http", 9));

        try!(s.serialize_struct_sep("protocol"));
        try!(self.protocol.serialize(s));

        try!(s.serialize_struct_sep("status"));
        try!(self.status.serialize(s));

        try!(s.serialize_struct_sep("host_status"));
        try!(self.host_status.serialize(s));

        try!(s.serialize_struct_sep("up_status"));
        try!(self.up_status.serialize(s));

        try!(s.serialize_struct_sep("method"));
        try!(self.method.serialize(s));

        try!(s.serialize_struct_sep("content_type"));
        try!(self.content_type.serialize(s));

        try!(s.serialize_struct_sep("user_agent"));
        try!(self.user_agent.serialize(s));

        try!(s.serialize_struct_sep("referer"));
        try!(self.referer.serialize(s));

        try!(s.serialize_struct_sep("request_uri"));
        try!(self.request_uri.serialize(s));

        s.serialize_struct_end()
    }
}

#[deriving(Encodable, Decodable)]
enum HttpProtocol {
    HTTP_PROTOCOL_UNKNOWN,
    HTTP10,
    HTTP11,
}

impl<S: ser::Serializer<E>, E> ser::Serializable<S, E> for HttpProtocol {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
        /*
        match *self {
            HTTP_PROTOCOL_UNKNOWN => {
                try!(s.serialize_EnumStart("HttpProtocol", "HTTP_PROTOCOL_UNKNOWN", 0)));
            }
            HTTP10 => {
                try!(s.serialize_EnumStart("HttpProtocol", "HTTP10", 0)));
            }
            HTTP11 => {
                try!(s.serialize_EnumStart("HttpProtocol", "HTTP11", 0)));
            }
        }

        s.serialize_end()
        */
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

impl<S: ser::Serializer<E>, E> ser::Serializable<S, E> for HttpMethod {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
            /*
        match *self {
            METHOD_UNKNOWN => {
                try!(s.serialize_EnumStart("HttpMethod", "METHOD_UNKNOWN", 0)));
            }
            GET => {
                try!(s.serialize_EnumStart("HttpMethod", "GET", 0)));
            }

            POST => {
                try!(s.serialize_EnumStart("HttpMethod", "POST", 0)));
            }
            DELETE => {
                try!(s.serialize_EnumStart("HttpMethod", "DELETE", 0)));
            }
            PUT => {
                try!(s.serialize_EnumStart("HttpMethod", "PUT", 0)));
            }
            HEAD => {
                try!(s.serialize_EnumStart("HttpMethod", "HEAD", 0)));
            }
            PURGE => {
                try!(s.serialize_EnumStart("HttpMethod", "PURGE", 0)));
            }
            OPTIONS => {
                try!(s.serialize_EnumStart("HttpMethod", "OPTIONS", 0)));
            }
            PROPFIND => {
                try!(s.serialize_EnumStart("HttpMethod", "PROPFIND", 0)));
            }
            MKCOL => {
                try!(s.serialize_EnumStart("HttpMethod", "MKCOL", 0)));
            }
            PATCH => {
                try!(s.serialize_EnumStart("HttpMethod", "PATCH", 0)));
            }
        }

        s.serialize_end()
        */
    }
}

#[deriving(Encodable, Decodable)]
enum CacheStatus {
    CACHESTATUS_UNKNOWN,
    Miss,
    Expired,
    Hit,
}

impl<S: ser::Serializer<E>, E> ser::Serializable<S, E> for CacheStatus {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
            /*
        match *self {
            CACHESTATUS_UNKNOWN => {
                try!(s.serialize_EnumStart("CacheStatus", "CACHESTATUS_UNKNOWN", 0)));
            }
            Miss => {
                try!(s.serialize_EnumStart("CacheStatus", "Miss", 0)));
            }
            Expired => {
                try!(s.serialize_EnumStart("CacheStatus", "Expired", 0)));
            }
            Hit => {
                try!(s.serialize_EnumStart("CacheStatus", "Hit", 0)));
            }
        }

        s.serialize_end()
        */
    }
}

#[deriving(Encodable, Decodable)]
struct Origin {
    ip: String,
    port: u32,
    hostname: String,
    protocol: OriginProtocol,
}

impl<S: ser::Serializer<E>, E> ser::Serializable<S, E> for Origin {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        try!(s.serialize_struct_start("Http", 4));

        try!(s.serialize_struct_sep("ip"));
        try!(self.ip.serialize(s));

        try!(s.serialize_struct_sep("port"));
        try!(self.port.serialize(s));

        try!(s.serialize_struct_sep("hostname"));
        try!(self.hostname.serialize(s));

        try!(s.serialize_struct_sep("protocol"));
        try!(self.protocol.serialize(s));

        s.serialize_struct_end()
    }
}

#[deriving(Encodable, Decodable)]
enum OriginProtocol {
    ORIGIN_PROTOCOL_UNKNOWN,
    HTTP,
    HTTPS,
}

impl<S: ser::Serializer<E>, E> ser::Serializable<S, E> for OriginProtocol {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
            /*
        match *self {
            ORIGIN_PROTOCOL_UNKNOWN => {
                try!(s.serialize_EnumStart("OriginProtocol", "ORIGIN_PROTOCOL_UNKNOWN", 0)));
            }
            HTTP => {
                try!(s.serialize_EnumStart("OriginProtocol", "HTTP", 0)));
            }
            HTTPS => {
                try!(s.serialize_EnumStart("OriginProtocol", "HTTPS", 0)));
            }
        }

        s.serialize_end()
        */
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

impl<S: ser::Serializer<E>, E> ser::Serializable<S, E> for ZonePlan {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
        /*
        match *self {
            ZONEPLAN_UNKNOWN => {
                try!(s.serialize_EnumStart("ZonePlan", "ZONEPLAN_UNKNOWN", 0)));
            }
            FREE => {
                try!(s.serialize_EnumStart("ZonePlan", "FREE", 0)));
            }
            PRO => {
                try!(s.serialize_EnumStart("ZonePlan", "PRO", 0)));
            }
            BIZ => {
                try!(s.serialize_EnumStart("ZonePlan", "BIZ", 0)));
            }
            ENT => {
                try!(s.serialize_EnumStart("ZonePlan", "ENT", 0)));
            }
        }

        s.serialize_end()
        */
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

impl<S: ser::Serializer<E>, E> ser::Serializable<S, E> for Country {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        s.serialize_uint(*self as uint)
        /*
        match *self {
            UNKNOWN => { try!(s.serialize_EnumStart("Country", "UNKNOWN", 0))); }
            A1 => { try!(s.serialize_EnumStart("Country", "A1", 0))); }
            A2 => { try!(s.serialize_EnumStart("Country", "A2", 0))); }
            O1 => { try!(s.serialize_EnumStart("Country", "O1", 0))); }
            AD => { try!(s.serialize_EnumStart("Country", "AD", 0))); }
            AE => { try!(s.serialize_EnumStart("Country", "AE", 0))); }
            AF => { try!(s.serialize_EnumStart("Country", "AF", 0))); }
            AG => { try!(s.serialize_EnumStart("Country", "AG", 0))); }
            AI => { try!(s.serialize_EnumStart("Country", "AI", 0))); }
            AL => { try!(s.serialize_EnumStart("Country", "AL", 0))); }
            AM => { try!(s.serialize_EnumStart("Country", "AM", 0))); }
            AO => { try!(s.serialize_EnumStart("Country", "AO", 0))); }
            AP => { try!(s.serialize_EnumStart("Country", "AP", 0))); }
            AQ => { try!(s.serialize_EnumStart("Country", "AQ", 0))); }
            AR => { try!(s.serialize_EnumStart("Country", "AR", 0))); }
            AS => { try!(s.serialize_EnumStart("Country", "AS", 0))); }
            AT => { try!(s.serialize_EnumStart("Country", "AT", 0))); }
            AU => { try!(s.serialize_EnumStart("Country", "AU", 0))); }
            AW => { try!(s.serialize_EnumStart("Country", "AW", 0))); }
            AX => { try!(s.serialize_EnumStart("Country", "AX", 0))); }
            AZ => { try!(s.serialize_EnumStart("Country", "AZ", 0))); }
            BA => { try!(s.serialize_EnumStart("Country", "BA", 0))); }
            BB => { try!(s.serialize_EnumStart("Country", "BB", 0))); }
            BD => { try!(s.serialize_EnumStart("Country", "BD", 0))); }
            BE => { try!(s.serialize_EnumStart("Country", "BE", 0))); }
            BF => { try!(s.serialize_EnumStart("Country", "BF", 0))); }
            BG => { try!(s.serialize_EnumStart("Country", "BG", 0))); }
            BH => { try!(s.serialize_EnumStart("Country", "BH", 0))); }
            BI => { try!(s.serialize_EnumStart("Country", "BI", 0))); }
            BJ => { try!(s.serialize_EnumStart("Country", "BJ", 0))); }
            BL => { try!(s.serialize_EnumStart("Country", "BL", 0))); }
            BM => { try!(s.serialize_EnumStart("Country", "BM", 0))); }
            BN => { try!(s.serialize_EnumStart("Country", "BN", 0))); }
            BO => { try!(s.serialize_EnumStart("Country", "BO", 0))); }
            BQ => { try!(s.serialize_EnumStart("Country", "BQ", 0))); }
            BR => { try!(s.serialize_EnumStart("Country", "BR", 0))); }
            BS => { try!(s.serialize_EnumStart("Country", "BS", 0))); }
            BT => { try!(s.serialize_EnumStart("Country", "BT", 0))); }
            BV => { try!(s.serialize_EnumStart("Country", "BV", 0))); }
            BW => { try!(s.serialize_EnumStart("Country", "BW", 0))); }
            BY => { try!(s.serialize_EnumStart("Country", "BY", 0))); }
            BZ => { try!(s.serialize_EnumStart("Country", "BZ", 0))); }
            CA => { try!(s.serialize_EnumStart("Country", "CA", 0))); }
            CC => { try!(s.serialize_EnumStart("Country", "CC", 0))); }
            CD => { try!(s.serialize_EnumStart("Country", "CD", 0))); }
            CF => { try!(s.serialize_EnumStart("Country", "CF", 0))); }
            CG => { try!(s.serialize_EnumStart("Country", "CG", 0))); }
            CH => { try!(s.serialize_EnumStart("Country", "CH", 0))); }
            CI => { try!(s.serialize_EnumStart("Country", "CI", 0))); }
            CK => { try!(s.serialize_EnumStart("Country", "CK", 0))); }
            CL => { try!(s.serialize_EnumStart("Country", "CL", 0))); }
            CM => { try!(s.serialize_EnumStart("Country", "CM", 0))); }
            CN => { try!(s.serialize_EnumStart("Country", "CN", 0))); }
            CO => { try!(s.serialize_EnumStart("Country", "CO", 0))); }
            CR => { try!(s.serialize_EnumStart("Country", "CR", 0))); }
            CU => { try!(s.serialize_EnumStart("Country", "CU", 0))); }
            CV => { try!(s.serialize_EnumStart("Country", "CV", 0))); }
            CW => { try!(s.serialize_EnumStart("Country", "CW", 0))); }
            CX => { try!(s.serialize_EnumStart("Country", "CX", 0))); }
            CY => { try!(s.serialize_EnumStart("Country", "CY", 0))); }
            CZ => { try!(s.serialize_EnumStart("Country", "CZ", 0))); }
            DE => { try!(s.serialize_EnumStart("Country", "DE", 0))); }
            DJ => { try!(s.serialize_EnumStart("Country", "DJ", 0))); }
            DK => { try!(s.serialize_EnumStart("Country", "DK", 0))); }
            DM => { try!(s.serialize_EnumStart("Country", "DM", 0))); }
            DO => { try!(s.serialize_EnumStart("Country", "DO", 0))); }
            DZ => { try!(s.serialize_EnumStart("Country", "DZ", 0))); }
            EC => { try!(s.serialize_EnumStart("Country", "EC", 0))); }
            EE => { try!(s.serialize_EnumStart("Country", "EE", 0))); }
            EG => { try!(s.serialize_EnumStart("Country", "EG", 0))); }
            EH => { try!(s.serialize_EnumStart("Country", "EH", 0))); }
            ER => { try!(s.serialize_EnumStart("Country", "ER", 0))); }
            ES => { try!(s.serialize_EnumStart("Country", "ES", 0))); }
            ET => { try!(s.serialize_EnumStart("Country", "ET", 0))); }
            EU => { try!(s.serialize_EnumStart("Country", "EU", 0))); }
            FI => { try!(s.serialize_EnumStart("Country", "FI", 0))); }
            FJ => { try!(s.serialize_EnumStart("Country", "FJ", 0))); }
            FK => { try!(s.serialize_EnumStart("Country", "FK", 0))); }
            FM => { try!(s.serialize_EnumStart("Country", "FM", 0))); }
            FO => { try!(s.serialize_EnumStart("Country", "FO", 0))); }
            FR => { try!(s.serialize_EnumStart("Country", "FR", 0))); }
            GA => { try!(s.serialize_EnumStart("Country", "GA", 0))); }
            GB => { try!(s.serialize_EnumStart("Country", "GB", 0))); }
            GD => { try!(s.serialize_EnumStart("Country", "GD", 0))); }
            GE => { try!(s.serialize_EnumStart("Country", "GE", 0))); }
            GF => { try!(s.serialize_EnumStart("Country", "GF", 0))); }
            GG => { try!(s.serialize_EnumStart("Country", "GG", 0))); }
            GH => { try!(s.serialize_EnumStart("Country", "GH", 0))); }
            GI => { try!(s.serialize_EnumStart("Country", "GI", 0))); }
            GL => { try!(s.serialize_EnumStart("Country", "GL", 0))); }
            GM => { try!(s.serialize_EnumStart("Country", "GM", 0))); }
            GN => { try!(s.serialize_EnumStart("Country", "GN", 0))); }
            GP => { try!(s.serialize_EnumStart("Country", "GP", 0))); }
            GQ => { try!(s.serialize_EnumStart("Country", "GQ", 0))); }
            GR => { try!(s.serialize_EnumStart("Country", "GR", 0))); }
            GS => { try!(s.serialize_EnumStart("Country", "GS", 0))); }
            GT => { try!(s.serialize_EnumStart("Country", "GT", 0))); }
            GU => { try!(s.serialize_EnumStart("Country", "GU", 0))); }
            GW => { try!(s.serialize_EnumStart("Country", "GW", 0))); }
            GY => { try!(s.serialize_EnumStart("Country", "GY", 0))); }
            HK => { try!(s.serialize_EnumStart("Country", "HK", 0))); }
            HM => { try!(s.serialize_EnumStart("Country", "HM", 0))); }
            HN => { try!(s.serialize_EnumStart("Country", "HN", 0))); }
            HR => { try!(s.serialize_EnumStart("Country", "HR", 0))); }
            HT => { try!(s.serialize_EnumStart("Country", "HT", 0))); }
            HU => { try!(s.serialize_EnumStart("Country", "HU", 0))); }
            ID => { try!(s.serialize_EnumStart("Country", "ID", 0))); }
            IE => { try!(s.serialize_EnumStart("Country", "IE", 0))); }
            IL => { try!(s.serialize_EnumStart("Country", "IL", 0))); }
            IM => { try!(s.serialize_EnumStart("Country", "IM", 0))); }
            IN => { try!(s.serialize_EnumStart("Country", "IN", 0))); }
            IO => { try!(s.serialize_EnumStart("Country", "IO", 0))); }
            IQ => { try!(s.serialize_EnumStart("Country", "IQ", 0))); }
            IR => { try!(s.serialize_EnumStart("Country", "IR", 0))); }
            IS => { try!(s.serialize_EnumStart("Country", "IS", 0))); }
            IT => { try!(s.serialize_EnumStart("Country", "IT", 0))); }
            JE => { try!(s.serialize_EnumStart("Country", "JE", 0))); }
            JM => { try!(s.serialize_EnumStart("Country", "JM", 0))); }
            JO => { try!(s.serialize_EnumStart("Country", "JO", 0))); }
            JP => { try!(s.serialize_EnumStart("Country", "JP", 0))); }
            KE => { try!(s.serialize_EnumStart("Country", "KE", 0))); }
            KG => { try!(s.serialize_EnumStart("Country", "KG", 0))); }
            KH => { try!(s.serialize_EnumStart("Country", "KH", 0))); }
            KI => { try!(s.serialize_EnumStart("Country", "KI", 0))); }
            KM => { try!(s.serialize_EnumStart("Country", "KM", 0))); }
            KN => { try!(s.serialize_EnumStart("Country", "KN", 0))); }
            KP => { try!(s.serialize_EnumStart("Country", "KP", 0))); }
            KR => { try!(s.serialize_EnumStart("Country", "KR", 0))); }
            KW => { try!(s.serialize_EnumStart("Country", "KW", 0))); }
            KY => { try!(s.serialize_EnumStart("Country", "KY", 0))); }
            KZ => { try!(s.serialize_EnumStart("Country", "KZ", 0))); }
            LA => { try!(s.serialize_EnumStart("Country", "LA", 0))); }
            LB => { try!(s.serialize_EnumStart("Country", "LB", 0))); }
            LC => { try!(s.serialize_EnumStart("Country", "LC", 0))); }
            LI => { try!(s.serialize_EnumStart("Country", "LI", 0))); }
            LK => { try!(s.serialize_EnumStart("Country", "LK", 0))); }
            LR => { try!(s.serialize_EnumStart("Country", "LR", 0))); }
            LS => { try!(s.serialize_EnumStart("Country", "LS", 0))); }
            LT => { try!(s.serialize_EnumStart("Country", "LT", 0))); }
            LU => { try!(s.serialize_EnumStart("Country", "LU", 0))); }
            LV => { try!(s.serialize_EnumStart("Country", "LV", 0))); }
            LY => { try!(s.serialize_EnumStart("Country", "LY", 0))); }
            MA => { try!(s.serialize_EnumStart("Country", "MA", 0))); }
            MC => { try!(s.serialize_EnumStart("Country", "MC", 0))); }
            MD => { try!(s.serialize_EnumStart("Country", "MD", 0))); }
            ME => { try!(s.serialize_EnumStart("Country", "ME", 0))); }
            MF => { try!(s.serialize_EnumStart("Country", "MF", 0))); }
            MG => { try!(s.serialize_EnumStart("Country", "MG", 0))); }
            MH => { try!(s.serialize_EnumStart("Country", "MH", 0))); }
            MK => { try!(s.serialize_EnumStart("Country", "MK", 0))); }
            ML => { try!(s.serialize_EnumStart("Country", "ML", 0))); }
            MM => { try!(s.serialize_EnumStart("Country", "MM", 0))); }
            MN => { try!(s.serialize_EnumStart("Country", "MN", 0))); }
            MO => { try!(s.serialize_EnumStart("Country", "MO", 0))); }
            MP => { try!(s.serialize_EnumStart("Country", "MP", 0))); }
            MQ => { try!(s.serialize_EnumStart("Country", "MQ", 0))); }
            MR => { try!(s.serialize_EnumStart("Country", "MR", 0))); }
            MS => { try!(s.serialize_EnumStart("Country", "MS", 0))); }
            MT => { try!(s.serialize_EnumStart("Country", "MT", 0))); }
            MU => { try!(s.serialize_EnumStart("Country", "MU", 0))); }
            MV => { try!(s.serialize_EnumStart("Country", "MV", 0))); }
            MW => { try!(s.serialize_EnumStart("Country", "MW", 0))); }
            MX => { try!(s.serialize_EnumStart("Country", "MX", 0))); }
            MY => { try!(s.serialize_EnumStart("Country", "MY", 0))); }
            MZ => { try!(s.serialize_EnumStart("Country", "MZ", 0))); }
            NA => { try!(s.serialize_EnumStart("Country", "NA", 0))); }
            NC => { try!(s.serialize_EnumStart("Country", "NC", 0))); }
            NE => { try!(s.serialize_EnumStart("Country", "NE", 0))); }
            NF => { try!(s.serialize_EnumStart("Country", "NF", 0))); }
            NG => { try!(s.serialize_EnumStart("Country", "NG", 0))); }
            NI => { try!(s.serialize_EnumStart("Country", "NI", 0))); }
            NL => { try!(s.serialize_EnumStart("Country", "NL", 0))); }
            NO => { try!(s.serialize_EnumStart("Country", "NO", 0))); }
            NP => { try!(s.serialize_EnumStart("Country", "NP", 0))); }
            NR => { try!(s.serialize_EnumStart("Country", "NR", 0))); }
            NU => { try!(s.serialize_EnumStart("Country", "NU", 0))); }
            NZ => { try!(s.serialize_EnumStart("Country", "NZ", 0))); }
            OM => { try!(s.serialize_EnumStart("Country", "OM", 0))); }
            PA => { try!(s.serialize_EnumStart("Country", "PA", 0))); }
            PE => { try!(s.serialize_EnumStart("Country", "PE", 0))); }
            PF => { try!(s.serialize_EnumStart("Country", "PF", 0))); }
            PG => { try!(s.serialize_EnumStart("Country", "PG", 0))); }
            PH => { try!(s.serialize_EnumStart("Country", "PH", 0))); }
            PK => { try!(s.serialize_EnumStart("Country", "PK", 0))); }
            PL => { try!(s.serialize_EnumStart("Country", "PL", 0))); }
            PM => { try!(s.serialize_EnumStart("Country", "PM", 0))); }
            PN => { try!(s.serialize_EnumStart("Country", "PN", 0))); }
            PR => { try!(s.serialize_EnumStart("Country", "PR", 0))); }
            PS => { try!(s.serialize_EnumStart("Country", "PS", 0))); }
            PT => { try!(s.serialize_EnumStart("Country", "PT", 0))); }
            PW => { try!(s.serialize_EnumStart("Country", "PW", 0))); }
            PY => { try!(s.serialize_EnumStart("Country", "PY", 0))); }
            QA => { try!(s.serialize_EnumStart("Country", "QA", 0))); }
            RE => { try!(s.serialize_EnumStart("Country", "RE", 0))); }
            RO => { try!(s.serialize_EnumStart("Country", "RO", 0))); }
            RS => { try!(s.serialize_EnumStart("Country", "RS", 0))); }
            RU => { try!(s.serialize_EnumStart("Country", "RU", 0))); }
            RW => { try!(s.serialize_EnumStart("Country", "RW", 0))); }
            SA => { try!(s.serialize_EnumStart("Country", "SA", 0))); }
            SB => { try!(s.serialize_EnumStart("Country", "SB", 0))); }
            SC => { try!(s.serialize_EnumStart("Country", "SC", 0))); }
            SD => { try!(s.serialize_EnumStart("Country", "SD", 0))); }
            SE => { try!(s.serialize_EnumStart("Country", "SE", 0))); }
            SG => { try!(s.serialize_EnumStart("Country", "SG", 0))); }
            SH => { try!(s.serialize_EnumStart("Country", "SH", 0))); }
            SI => { try!(s.serialize_EnumStart("Country", "SI", 0))); }
            SJ => { try!(s.serialize_EnumStart("Country", "SJ", 0))); }
            SK => { try!(s.serialize_EnumStart("Country", "SK", 0))); }
            SL => { try!(s.serialize_EnumStart("Country", "SL", 0))); }
            SM => { try!(s.serialize_EnumStart("Country", "SM", 0))); }
            SN => { try!(s.serialize_EnumStart("Country", "SN", 0))); }
            SO => { try!(s.serialize_EnumStart("Country", "SO", 0))); }
            SR => { try!(s.serialize_EnumStart("Country", "SR", 0))); }
            SS => { try!(s.serialize_EnumStart("Country", "SS", 0))); }
            ST => { try!(s.serialize_EnumStart("Country", "ST", 0))); }
            SV => { try!(s.serialize_EnumStart("Country", "SV", 0))); }
            SX => { try!(s.serialize_EnumStart("Country", "SX", 0))); }
            SY => { try!(s.serialize_EnumStart("Country", "SY", 0))); }
            SZ => { try!(s.serialize_EnumStart("Country", "SZ", 0))); }
            TC => { try!(s.serialize_EnumStart("Country", "TC", 0))); }
            TD => { try!(s.serialize_EnumStart("Country", "TD", 0))); }
            TF => { try!(s.serialize_EnumStart("Country", "TF", 0))); }
            TG => { try!(s.serialize_EnumStart("Country", "TG", 0))); }
            TH => { try!(s.serialize_EnumStart("Country", "TH", 0))); }
            TJ => { try!(s.serialize_EnumStart("Country", "TJ", 0))); }
            TK => { try!(s.serialize_EnumStart("Country", "TK", 0))); }
            TL => { try!(s.serialize_EnumStart("Country", "TL", 0))); }
            TM => { try!(s.serialize_EnumStart("Country", "TM", 0))); }
            TN => { try!(s.serialize_EnumStart("Country", "TN", 0))); }
            TO => { try!(s.serialize_EnumStart("Country", "TO", 0))); }
            TR => { try!(s.serialize_EnumStart("Country", "TR", 0))); }
            TT => { try!(s.serialize_EnumStart("Country", "TT", 0))); }
            TV => { try!(s.serialize_EnumStart("Country", "TV", 0))); }
            TW => { try!(s.serialize_EnumStart("Country", "TW", 0))); }
            TZ => { try!(s.serialize_EnumStart("Country", "TZ", 0))); }
            UA => { try!(s.serialize_EnumStart("Country", "UA", 0))); }
            UG => { try!(s.serialize_EnumStart("Country", "UG", 0))); }
            UM => { try!(s.serialize_EnumStart("Country", "UM", 0))); }
            US => { try!(s.serialize_EnumStart("Country", "US", 0))); }
            UY => { try!(s.serialize_EnumStart("Country", "UY", 0))); }
            UZ => { try!(s.serialize_EnumStart("Country", "UZ", 0))); }
            VA => { try!(s.serialize_EnumStart("Country", "VA", 0))); }
            VC => { try!(s.serialize_EnumStart("Country", "VC", 0))); }
            VE => { try!(s.serialize_EnumStart("Country", "VE", 0))); }
            VG => { try!(s.serialize_EnumStart("Country", "VG", 0))); }
            VI => { try!(s.serialize_EnumStart("Country", "VI", 0))); }
            VN => { try!(s.serialize_EnumStart("Country", "VN", 0))); }
            VU => { try!(s.serialize_EnumStart("Country", "VU", 0))); }
            WF => { try!(s.serialize_EnumStart("Country", "WF", 0))); }
            WS => { try!(s.serialize_EnumStart("Country", "WS", 0))); }
            XX => { try!(s.serialize_EnumStart("Country", "XX", 0))); }
            YE => { try!(s.serialize_EnumStart("Country", "YE", 0))); }
            YT => { try!(s.serialize_EnumStart("Country", "YT", 0))); }
            ZA => { try!(s.serialize_EnumStart("Country", "ZA", 0))); }
            ZM => { try!(s.serialize_EnumStart("Country", "ZM", 0))); }
            ZW => { try!(s.serialize_EnumStart("Country", "ZW", 0))); }
        }

        s.serialize_end()
        */
    }
}

#[deriving(Encodable, Decodable)]
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

impl<S: ser::Serializer<E>, E> ser::Serializable<S, E> for Log {
    #[inline]
    fn serialize(&self, s: &mut S) -> Result<(), E> {
        try!(s.serialize_struct_start("Log", 12));

        try!(s.serialize_struct_sep("timestamp"));
        try!(self.timestamp.serialize(s));

        try!(s.serialize_struct_sep("zone_id"));
        try!(self.zone_id.serialize(s));

        try!(s.serialize_struct_sep("zone_plan"));
        try!(self.zone_plan.serialize(s));

        try!(s.serialize_struct_sep("http"));
        try!(self.http.serialize(s));

        try!(s.serialize_struct_sep("origin"));
        try!(self.origin.serialize(s));

        try!(s.serialize_struct_sep("country"));
        try!(self.country.serialize(s));

        try!(s.serialize_struct_sep("cache_status"));
        try!(self.cache_status.serialize(s));

        try!(s.serialize_struct_sep("server_ip"));
        try!(self.server_ip.serialize(s));

        try!(s.serialize_struct_sep("server_name"));
        try!(self.server_name.serialize(s));

        try!(s.serialize_struct_sep("remote_ip"));
        try!(self.remote_ip.serialize(s));

        try!(s.serialize_struct_sep("bytes_dlv"));
        try!(self.bytes_dlv.serialize(s));

        try!(s.serialize_struct_sep("ray_id"));
        try!(self.ray_id.serialize(s));

        s.serialize_struct_end()
    }
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
    //println!("json: {}", json);
    let _len = json.len();

    b.iter(|| {
        let _ = serialize::json::Encoder::str_encode(&log);
    });
}

#[bench]
fn bench_serializer(b: &mut Bencher) {
    let log = Log::new();

    let mut wr = MemWriter::with_capacity(700);
    {
        let mut serializer = json::Serializer::new(&mut wr);
        log.serialize(&mut serializer).unwrap();
    }
    let json = String::from_utf8(wr.unwrap()).unwrap();
    //println!("json: {}", json);
    let _len = json.len();

    b.iter(|| {
        let mut wr = MemWriter::with_capacity(700);
        {
            let mut serializer = json::Serializer::new(&mut wr);
            log.serialize(&mut serializer).unwrap();
        }
        //let _ = String::from_utf8(wr.unwrap());
    });
}

#[bench]
fn bench_decooder(b: &mut Bencher) {
    let s = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":"FREE","http":{"protocol":"HTTP11","status":200,"host_status":503,"up_status":520,"method":"GET","content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":"HTTPS"},"country":"US","cache_status":"Hit","server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;
        
        
        //"{"timestamp":3444218605346,"zone_id":123456,"zone_plan":1,"http":{"protocol":2,"status":200,"host_status":503,"up_status":520,"method":1,"content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":2},"country":238,"cache_status":3,"server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

    b.iter(|| {
        let json = serialize::json::from_str(s).unwrap();
        //println!("json: {}", json);
        let mut decoder = serialize::json::Decoder::new(json);
        let _log: Log = serialize::Decodable::decode(&mut decoder).unwrap();
    });

}
