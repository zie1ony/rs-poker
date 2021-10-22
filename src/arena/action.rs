#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Call,
    Check,
    Fold,
    Raise(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raise() {
        let a = Action::Raise(100);
        assert_eq!(Action::Raise(100), a);
    }
}
