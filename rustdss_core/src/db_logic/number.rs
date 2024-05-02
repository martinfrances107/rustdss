use crate::CoreState;
use rustdss_data::RespData;

fn can_be_number(data: &RespData) -> Option<i64> {
    match data {
        RespData::Number(val) => Some(*val),
        RespData::SimpleStr(val) => val.parse().ok(),
        RespData::BulkStr(val) => val.parse().ok(),
        _ => None,
    }
}

pub fn incr(state: &mut CoreState, key: String, maybe_by: Option<i64>) -> RespData {
    let prev = state.keyval.get(&key);

    let op = match prev {
        Some(RespData::Number(val)) => Ok(RespData::Number((*val) + maybe_by.unwrap_or(1))),
        Some(val) => can_be_number(val)
            .map(|v| RespData::Number(v + maybe_by.unwrap_or(1)))
            .ok_or(RespData::Error("NaN".into())),
        None => Ok(RespData::Number(1)),
    };

    if let Ok(new_val) = op {
        state.keyval.insert(key, new_val.clone());
        new_val
    } else {
        op.err().unwrap()
    }
}

pub fn decr(state: &mut CoreState, key: String, maybe_by: Option<i64>) -> RespData {
    let prev = state.keyval.get(&key);

    let op = match prev {
        Some(RespData::Number(val)) => Ok(RespData::Number((*val) - maybe_by.unwrap_or(1))),
        Some(val) => can_be_number(val)
            .map(|v| RespData::Number(v - maybe_by.unwrap_or(1)))
            .ok_or(RespData::Error("NaN".into())),
        None => Ok(RespData::Number(-1)),
    };

    if let Ok(new_val) = op {
        state.keyval.insert(key, new_val.clone());
        new_val
    } else {
        op.err().unwrap()
    }
}

#[cfg(test)]
mod incr_should {
    use super::*;
    use crate::CoreState;
    use std::collections::HashMap;
    // increase_values_that_are_already_numbers
    #[test]
    fn increase_values_that_are_already_numbers() {
        let mut keyval = HashMap::new();
        keyval.insert("key".into(), RespData::Number(5));
        let mut state = CoreState { keyval };

        let response1 = incr(&mut state, "key".into(), None);
        let response2 = incr(&mut state, "key".into(), Some(2));

        assert_eq!(state.keyval.get("key"), Some(&RespData::Number(8)));
        assert_eq!(response1, RespData::Number(6));
        assert_eq!(response2, RespData::Number(8));
    }
    // try_to_convert_strings_into_numbers

    #[test]
    fn try_to_convert_strings_into_numbers() {
        let mut keyval = HashMap::new();
        keyval.insert("key1".into(), RespData::SimpleStr("27".into()));
        keyval.insert("key2".into(), RespData::SimpleStr("not_a_number".into()));

        let mut state = CoreState { keyval };

        let response1 = incr(&mut state, "key1".into(), None);
        let response2 = incr(&mut state, "key1".into(), Some(2));

        let response3 = incr(&mut state, "key2".into(), None);
        let response4 = incr(&mut state, "key2".into(), Some(2));

        assert_eq!(response1, RespData::Number(28));
        assert_eq!(response2, RespData::Number(30));
        assert_eq!(state.keyval.get("key1"), Some(&RespData::Number(30)));

        assert_eq!(response3, RespData::Error("NaN".into()));
        assert_eq!(response4, RespData::Error("NaN".into()));
        assert_eq!(
            state.keyval.get("key2"),
            Some(&RespData::SimpleStr("not_a_number".into()))
        );
    }

    #[test]
    fn create_new_keys() {
        let mut state = CoreState {
            keyval: HashMap::new(),
        };

        let response1 = incr(&mut state, "key".into(), None);
        let response2 = incr(&mut state, "key".into(), Some(4));

        assert_eq!(response1, RespData::Number(1));
        assert_eq!(response2, RespData::Number(5));
        assert_eq!(state.keyval.get("key"), Some(&RespData::Number(5)));
    }
}

#[cfg(test)]
mod decr_should {
    use super::*;
    use crate::CoreState;
    use std::collections::HashMap;
    #[test]
    fn decrease_values_that_are_already_numbers() {
        let mut keyval = HashMap::new();
        keyval.insert("key".into(), RespData::Number(5));
        let mut state = CoreState { keyval };

        let response1 = decr(&mut state, "key".into(), None);
        let response2 = decr(&mut state, "key".into(), Some(2));

        assert_eq!(state.keyval.get("key"), Some(&RespData::Number(2)));
        assert_eq!(response1, RespData::Number(4));
        assert_eq!(response2, RespData::Number(2));
    }
    // try_to_convert_strings_into_numbers

    #[test]
    fn try_to_convert_strings_into_numbers() {
        let mut keyval = HashMap::new();
        keyval.insert("key1".into(), RespData::SimpleStr("27".into()));
        keyval.insert("key2".into(), RespData::SimpleStr("not_a_number".into()));

        let mut state = CoreState { keyval };

        let response1 = decr(&mut state, "key1".into(), None);
        let response2 = decr(&mut state, "key1".into(), Some(2));

        let response3 = decr(&mut state, "key2".into(), None);
        let response4 = decr(&mut state, "key2".into(), Some(2));

        assert_eq!(response1, RespData::Number(26));
        assert_eq!(response2, RespData::Number(24));
        assert_eq!(state.keyval.get("key1"), Some(&RespData::Number(24)));

        assert_eq!(response3, RespData::Error("NaN".into()));
        assert_eq!(response4, RespData::Error("NaN".into()));
        assert_eq!(
            state.keyval.get("key2"),
            Some(&RespData::SimpleStr("not_a_number".into()))
        );
    }

    #[test]
    fn create_new_keys() {
        let mut state = CoreState {
            keyval: HashMap::new(),
        };

        let response1 = decr(&mut state, "key".into(), None);
        let response2 = decr(&mut state, "key".into(), Some(4));

        assert_eq!(response1, RespData::Number(-1));
        assert_eq!(response2, RespData::Number(-5));
        assert_eq!(state.keyval.get("key"), Some(&RespData::Number(-5)));
    }
}
