use aes::cipher::KeyIvInit;
use cipher::StreamCipher;
use once_cell::sync::Lazy;

type Aes128OfbEnc = ofb::Ofb<aes::Aes128>;
type Aes128OfbDec = ofb::Ofb<aes::Aes128>;

static AES_KEY: Lazy<Vec<u8>> = Lazy::new(|| {
    //md5('zct_ase_encrypt_key') 16
    "b65641dcca2071d1".as_bytes().to_vec()
});
static AES_IV: Lazy<Vec<u8>> = Lazy::new(|| {
    //md5('zct_ase_encrypt_iv') 16
    "0ad4948248175fff".as_bytes().to_vec()
});

//编码
pub fn encode(input: &mut [u8]) {
    let key = AES_KEY.as_slice().into();
    let iv = AES_IV.as_slice().into();
    let mut aes = Aes128OfbEnc::new(key, iv);
    aes.apply_keystream(input);
}

//解码
pub fn decode(cipher: &mut [u8]) {
    let key = AES_KEY.as_slice().into();
    let iv = AES_IV.as_slice().into();
    let mut aes = Aes128OfbEnc::new(key, iv);
    aes.apply_keystream(cipher);
}