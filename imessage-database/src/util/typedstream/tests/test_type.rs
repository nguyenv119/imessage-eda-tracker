#[cfg(test)]
mod type_tests {
    use crate::util::typedstream::models::Type;

    #[test]
    fn can_get_array_good() {
        let items: Vec<u8> = vec![0x5b, 0x39, 0x30, 0x34, 0x63, 0x5d];

        let expected = vec![Type::Array(904)];
        let result = Type::get_array_length(&items).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn cant_get_array_bad() {
        let items: Vec<u8> = vec![0x39, 0x30, 0x34, 0x63, 0x5d];

        let result = Type::get_array_length(&items);

        assert!(result.is_none());
    }
}
