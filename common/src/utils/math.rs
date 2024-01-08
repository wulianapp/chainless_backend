pub fn gen_random_verify_code() -> u32{
    rand::random::<u32>() % 900000 + 100000
}