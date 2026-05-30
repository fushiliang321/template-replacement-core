use aes::cipher::KeyIvInit;
use cipher::{Iv, Key, StreamCipher};
use std::sync::LazyLock;


type Aes128OfbEnc = ofb::Ofb<aes::Aes128>;
type Aes128OfbDec = ofb::Ofb<aes::Aes128>;
type Aes128Ofb = ofb::Ofb<aes::Aes128>;


static AES_KEY: LazyLock<Vec<u8>> = LazyLock::new(|| {
    //md5('zct_ase_encrypt_key') 16
    "b65641dcca2071d1".as_bytes().to_vec()
});
static AES_IV: LazyLock<Vec<u8>> = LazyLock::new(|| {
    //md5('zct_ase_encrypt_iv') 16
    "0ad4948248175fff".as_bytes().to_vec()
});

//编码
pub fn encode(input: &mut [u8]) {
    let key = Key::<Aes128Ofb>::try_from(&AES_KEY[..]).unwrap();
    let iv = Iv::<Aes128Ofb>::try_from(&AES_IV[..]).unwrap();
    let mut aes = Aes128OfbEnc::new(&key, &iv);
    aes.apply_keystream(input);
}

//解码
pub fn decode(cipher: &mut [u8]) {
    let key = Key::<Aes128Ofb>::try_from(&AES_KEY[..]).unwrap();
    let iv = Iv::<Aes128Ofb>::try_from(&AES_IV[..]).unwrap();
    let mut aes = Aes128OfbDec::new(&key, &iv);
    aes.apply_keystream(cipher);
}