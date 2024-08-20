fn main(x: u8) {
    let digest1 = std::hash::keccak256([x], 1);
    let digest2 = std::hash::keccak256([x], 1);
    assert_eq(digest1, digest2);
}
