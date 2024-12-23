use magic_crypt::{MagicCryptTrait, new_magic_crypt};

fn main() {
    let mc = new_magic_crypt!("magickey", 256);

    let msg = "This is a sample message. Do you read this?";
    println!("Original Msg: {:?}", msg);

    let encrypted_msg = mc.encrypt_str_to_bytes(msg);
    println!("Encrypted Data: {:x?}", encrypted_msg);

    let unecrypted_msg = mc.decrypt_bytes_to_bytes(&encrypted_msg).unwrap();
    println!("Unencrypted Msg: {:?}", String::from_utf8(unecrypted_msg).unwrap());
}
