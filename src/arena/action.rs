#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Fold,
    Bet(i32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bet() {
        let a = Action::Bet(100);
        assert_eq!(Action::Bet(100), a);
    }
}
