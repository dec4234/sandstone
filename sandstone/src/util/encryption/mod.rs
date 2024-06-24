//! AES/CFB8 encryption utilities. https://wiki.vg/Protocol_Encryption

use aes::cipher::{AsyncStreamCipher, KeyIvInit};

type Aes128Cfb8Enc = cfb8::Encryptor<aes::Aes128>;
type Aes128Cfb8Dec = cfb8::Decryptor<aes::Aes128>;

#[test]
fn encryption_testing() {
	let key = [0x42; 16];
	let iv = [0x24; 16];
	let text = *b"HELLO WORLD ABCDEFGHIJKLMNOPQRSTUV";
	println!("length: {}", text.len());
	let cipher = hex::encode("33b356ce9184290c4c8facc1c0b1f918d5475aeb75b88c161ca65bdf05c7137ff4b0");

	let mut buf = text.to_vec();
	Aes128Cfb8Enc::new(&key.into(), &iv.into()).encrypt(&mut buf);
	//assert_eq!(&buf[..], &cipher.as_bytes()[..]);
	
	println!("Original: {:?}", text);
	println!("Encrypted: {:?}", buf);
	
	assert_ne!(buf[..], text[..]); // encrypted should not be the same as original

	Aes128Cfb8Dec::new(&key.into(), &iv.into()).decrypt(&mut buf);
	assert_eq!(buf[..], text[..]);
	
	println!("Decrypted: {:?}", buf);

	let mut buf = [0u8; 34];
	Aes128Cfb8Enc::new(&key.into(), &iv.into()).encrypt_b2b(&text, &mut buf).unwrap();
	//assert_eq!(&buf[..], &cipher.as_bytes()[..]);
	
	let mut out_buf = [0u8; 34];
	Aes128Cfb8Dec::new(&key.into(), &iv.into()).decrypt_b2b(&buf, &mut out_buf).unwrap();
	assert_eq!(out_buf[..], text[..]);
}