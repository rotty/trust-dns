/*
 * Copyright (C) 2016 Benjamin Fry <benjaminfry@me.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use ::serialize::binary::*;
use ::error::*;
use ::rr::record_data::RData;
use ::rr::dnssec::Algorithm;

// RFC 4034                DNSSEC Resource Records               March 2005
//
// 2.  The DNSKEY Resource Record
//
//    DNSSEC uses public key cryptography to sign and authenticate DNS
//    resource record sets (RRsets).  The public keys are stored in DNSKEY
//    resource records and are used in the DNSSEC authentication process
//    described in [RFC4035]: A zone signs its authoritative RRsets by
//    using a private key and stores the corresponding public key in a
//    DNSKEY RR.  A resolver can then use the public key to validate
//    signatures covering the RRsets in the zone, and thus to authenticate
//    them.
//
//    The DNSKEY RR is not intended as a record for storing arbitrary
//    public keys and MUST NOT be used to store certificates or public keys
//    that do not directly relate to the DNS infrastructure.
//
//    The Type value for the DNSKEY RR type is 48.
//
//    The DNSKEY RR is class independent.
//
//    The DNSKEY RR has no special TTL requirements.
//
// 2.1.  DNSKEY RDATA Wire Format
//
//    The RDATA for a DNSKEY RR consists of a 2 octet Flags Field, a 1
//    octet Protocol Field, a 1 octet Algorithm Field, and the Public Key
//    Field.
//
//                         1 1 1 1 1 1 1 1 1 1 2 2 2 2 2 2 2 2 2 2 3 3
//     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |              Flags            |    Protocol   |   Algorithm   |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    /                                                               /
//    /                            Public Key                         /
//    /                                                               /
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//
// 2.1.1.  The Flags Field
//
//    Bit 7 of the Flags field is the Zone Key flag.  If bit 7 has value 1,
//    then the DNSKEY record holds a DNS zone key, and the DNSKEY RR's
//    owner name MUST be the name of a zone.  If bit 7 has value 0, then
//    the DNSKEY record holds some other type of DNS public key and MUST
//    NOT be used to verify RRSIGs that cover RRsets.
//
//    Bit 15 of the Flags field is the Secure Entry Point flag, described
//    in [RFC3757].  If bit 15 has value 1, then the DNSKEY record holds a
//    key intended for use as a secure entry point.  This flag is only
//    intended to be a hint to zone signing or debugging software as to the
//    intended use of this DNSKEY record; validators MUST NOT alter their
//    behavior during the signature validation process in any way based on
//    the setting of this bit.  This also means that a DNSKEY RR with the
//    SEP bit set would also need the Zone Key flag set in order to be able
//    to generate signatures legally.  A DNSKEY RR with the SEP set and the
//    Zone Key flag not set MUST NOT be used to verify RRSIGs that cover
//    RRsets.
//
//    Bits 0-6 and 8-14 are reserved: these bits MUST have value 0 upon
//    creation of the DNSKEY RR and MUST be ignored upon receipt.
//
// ------------
// RFC 5011                  Trust Anchor Update             September 2007
//
// 7.  IANA Considerations
//
//   The IANA has assigned a bit in the DNSKEY flags field (see Section 7
//   of [RFC4034]) for the REVOKE bit (8).
//
// END RFC 5011
// ------------
//
// 2.1.2.  The Protocol Field
//
//    The Protocol Field MUST have value 3, and the DNSKEY RR MUST be
//    treated as invalid during signature verification if it is found to be
//    some value other than 3.
//
// 2.1.3.  The Algorithm Field
//
//    The Algorithm field identifies the public key's cryptographic
//    algorithm and determines the format of the Public Key field.  A list
//    of DNSSEC algorithm types can be found in Appendix A.1
//
// 2.1.4.  The Public Key Field
//
//    The Public Key Field holds the public key material.  The format
//    depends on the algorithm of the key being stored and is described in
//    separate documents.
//
// 2.1.5.  Notes on DNSKEY RDATA Design
//
//    Although the Protocol Field always has value 3, it is retained for
//    backward compatibility with early versions of the KEY record.
//
// 2.2.  The DNSKEY RR Presentation Format
//
//    The presentation format of the RDATA portion is as follows:
//
//    The Flag field MUST be represented as an unsigned decimal integer.
//    Given the currently defined flags, the possible values are: 0, 256,
//    and 257.
//
//    The Protocol Field MUST be represented as an unsigned decimal integer
//    with a value of 3.
//
//    The Algorithm field MUST be represented either as an unsigned decimal
//    integer or as an algorithm mnemonic as specified in Appendix A.1.
//
//    The Public Key field MUST be represented as a Base64 encoding of the
//    Public Key.  Whitespace is allowed within the Base64 text.  For a
//    definition of Base64 encoding, see [RFC3548].
//
// 2.3.  DNSKEY RR Example
//
//    The following DNSKEY RR stores a DNS zone key for example.com.
//
//    example.com. 86400 IN DNSKEY 256 3 5 ( AQPSKmynfzW4kyBv015MUG2DeIQ3
//                                           Cbl+BBZH4b/0PY1kxkmvHjcZc8no
//                                           kfzj31GajIQKY+5CptLr3buXA10h
//                                           WqTkF7H6RfoRqXQeogmMHfpftf6z
//                                           Mv1LyBUgia7za6ZEzOJBOztyvhjL
//                                           742iU/TpPSEDhm2SNKLijfUppn1U
//                                           aNvv4w==  )
//
//    The first four text fields specify the owner name, TTL, Class, and RR
//    type (DNSKEY).  Value 256 indicates that the Zone Key bit (bit 7) in
//    the Flags field has value 1.  Value 3 is the fixed Protocol value.
//    Value 5 indicates the public key algorithm.  Appendix A.1 identifies
//    algorithm type 5 as RSA/SHA1 and indicates that the format of the
//    RSA/SHA1 public key field is defined in [RFC3110].  The remaining
//    text is a Base64 encoding of the public key.

// DNSKEY { zone_key: bool, secure_entry_point:bool, algorithm: Algorithm,
//          public_key: Vec<u8> /* TODO, probably make this an enum variant */}
pub fn read(decoder: &mut BinDecoder, rdata_length: u16) -> DecodeResult<RData> {
  let flags: u16 = try!(decoder.read_u16());

  let zone_key: bool = flags & 0b0000_0001_0000_0000 == 0b0000_0001_0000_0000;
  let secure_entry_point: bool = flags & 0b0000_0000_0000_0001 == 0b0000_0000_0000_00001;
  let revoke: bool = flags & 0b0000_0000_1000_0000 == 0b0000_0000_1000_0000;
  let protocol: u8 = try!(decoder.read_u8());

  // protocol is defined to only be '3' right now
  if protocol != 3 { return Err(DecodeError::DnsKeyProtocolNot3(protocol)) }

  let algorithm: Algorithm = try!(Algorithm::read(decoder));

  // the public key is the left-over bytes minus 4 for the first fields
  // TODO: decode the key here?
  let public_key: Vec<u8> = try!(decoder.read_vec((rdata_length - 4) as usize));

  Ok(RData::DNSKEY {
    zone_key: zone_key, secure_entry_point: secure_entry_point, revoke: revoke, algorithm: algorithm,
    public_key: public_key
  })
}

pub fn emit(encoder: &mut BinEncoder, rdata: &RData) -> EncodeResult {
  if let RData::DNSKEY { zone_key, secure_entry_point, revoke, algorithm, ref public_key } = *rdata {
    let mut flags: u16 = 0;
    if zone_key { flags |= 0b0000_0001_0000_0000 }
    if secure_entry_point { flags |= 0b0000_0000_0000_0001 }
    if revoke { flags |= 0b0000_0000_1000_0000 }
    try!(encoder.emit_u16(flags));
    try!(encoder.emit(3)); // always 3 for now
    try!(algorithm.emit(encoder));
    try!(encoder.emit_vec(public_key));

    Ok(())
  } else {
    panic!("wrong type here {:?}", rdata);
  }
}

#[test]
pub fn test() {
  let rdata = RData::DNSKEY{ zone_key: true, secure_entry_point: true, revoke: false,
                             algorithm: Algorithm::RSASHA256, public_key: vec![0,1,2,3,4,5,6,7] };

  let mut bytes = Vec::new();
  let mut encoder: BinEncoder = BinEncoder::new(&mut bytes);
  assert!(emit(&mut encoder, &rdata).is_ok());
  let bytes = encoder.as_bytes();

  println!("bytes: {:?}", bytes);

  let mut decoder: BinDecoder = BinDecoder::new(bytes);
  let read_rdata = read(&mut decoder, bytes.len() as u16);
  assert!(read_rdata.is_ok(), format!("error decoding: {:?}", read_rdata.unwrap_err()));
  assert_eq!(rdata, read_rdata.unwrap());
}