// This will provide list support - eventually support blocking commands?
// Commands: LPUSH, RPUSH, LPOP, RPOP, RPOPLPUSH

use crate::CoreState;
use rustdss_data::{Key, RespData};
use std::collections::VecDeque;

pub fn lpush(state: &mut CoreState, key: &Key, data: RespData) -> RespData {
    match state.keyval.get_mut(key) {
        Some(RespData::List(list)) => {
            list.push_front(data);
            RespData::Number(list.len() as i64)
        }
        Some(_) => RespData::wrong_type(),
        None => {
            // Create a new list with the one thing in it
            state
                .keyval
                .insert(key.clone(), RespData::List(vec![data].into()));
            RespData::Number(1)
        }
    }
}

pub fn lpop(state: &mut CoreState, key: &Key) -> RespData {
    match state.keyval.get_mut(key) {
        Some(RespData::List(list)) => list.pop_front().unwrap_or(RespData::nil()),
        Some(_) => RespData::wrong_type(),
        _ => RespData::nil(),
    }
}

pub fn rpush(state: &mut CoreState, key: &Key, data: RespData) -> RespData {
    match state.keyval.get_mut(key) {
        Some(RespData::List(list)) => {
            list.push_back(data);
            RespData::Number(list.len() as i64)
        }
        Some(_) => RespData::wrong_type(),
        None => {
            // Create a new list with the one thing in it
            state
                .keyval
                .insert(key.clone(), RespData::List(vec![data].into()));
            RespData::Number(1)
        }
    }
}

pub fn rpop(state: &mut CoreState, key: &Key) -> RespData {
    match state.keyval.get_mut(key) {
        Some(RespData::List(list)) => list.pop_back().unwrap_or(RespData::nil()),
        Some(_) => RespData::wrong_type(),
        _ => RespData::nil(),
    }
}

pub fn llen(state: &CoreState, key: &Key) -> RespData {
    state
        .keyval
        .get(key)
        .and_then(|entry| match entry {
            RespData::List(l) => Some(RespData::Number(l.len() as i64)),
            _ => None,
        })
        .unwrap_or(RespData::nil())
}
/*
  From redis.io/commands/lrange:

  Returns the specified elements of the list stored at key. The offsets start
  and stop are zero-based indexes, with 0 being the first element of the list
  (the head of the list), 1 being the next element and so on.

  These offsets can also be negative numbers indicating offsets starting at the
  end of the list. For example, -1 is the last element of the list, -2 the
  penultimate, and so on.
*/

pub fn lrange(state: &CoreState, key: &Key, start: i64, end: i64) -> RespData {
    fn start_front_or_back(total: usize, start: i64) -> i64 {
        if start >= 0 {
            start
        } else {
            total as i64 + start
        }
    }

    fn end_front_or_back(total: usize, start: i64, end: i64) -> usize {
        // We want the offset from the start
        let start_offset = start_front_or_back(total, start);
        if end >= 0 {
            (end - start_offset) as usize + 1
        } else {
            println!("debug: {}", end);
            let end_abs = start_front_or_back(total, end);
            println!("debug_abs: {}", end_abs);
            ((end_abs - start_offset) + 1) as usize
        }
    }

    state
        .keyval
        .get(key)
        .and_then(|entry| match entry {
            RespData::List(inner_list) => {
                // This is where the complicated behaviour happens
                let total = inner_list.len();
                let result: VecDeque<RespData> = inner_list
                    .iter()
                    .skip(start_front_or_back(total, start) as usize)
                    .take(end_front_or_back(total, start, end)).cloned() // ew gross - could be v expensive!
                    .collect();

                //
                Some(RespData::List(result))
            }
            _ => None,
        })
        .unwrap_or(RespData::nil())
}

#[cfg(test)]
mod lpush_should {
    use super::*;
    use crate::CoreState;
    use std::collections::HashMap;

    #[test]
    fn create_a_new_list() {
        let key: String = "key".into();
        let mut state = CoreState {
            keyval: HashMap::new(),
        };

        let response = lpush(&mut state, &key, RespData::SimpleStr("value".into()));

        assert_eq!(
            state.keyval.get(&key),
            Some(&RespData::List(
                vec![RespData::SimpleStr("value".into())].into()
            ))
        );
        assert_eq!(response, RespData::Number(1));
    }

    #[test]
    fn push_items_to_the_beginning_of_the_list() {
        let key: String = "key:00".into();

        let mut keyval = HashMap::new();
        keyval.insert(
            key.clone(),
            RespData::List(
                vec![
                    RespData::SimpleStr("first".into()),
                    RespData::SimpleStr("second".into()),
                ]
                .into(),
            ),
        );
        let mut state = CoreState { keyval };

        let response = lpush(
            &mut state,
            &key,
            RespData::SimpleStr("should_be_first".into()),
        );

        assert_eq!(
            state.keyval.get(&key),
            Some(&RespData::List(
                vec![
                    RespData::SimpleStr("should_be_first".into()),
                    RespData::SimpleStr("first".into()),
                    RespData::SimpleStr("second".into()),
                ]
                .into()
            ))
        );
        assert_eq!(response, RespData::Number(3));
    }

    #[test]
    fn error_when_the_list_is_a_different_type() {
        let key: String = "key:00".into();

        let mut keyval = HashMap::new();
        keyval.insert(key.clone(), RespData::SimpleStr("not_a_list".into()));
        let mut state = CoreState { keyval };

        let response = lpush(
            &mut state,
            &key,
            RespData::SimpleStr("some_new_data".into()),
        );

        // Assert that the original item was not mutated
        assert_eq!(
            state.keyval.get(&key),
            Some(&RespData::SimpleStr("not_a_list".into()))
        );
        assert_eq!(response, RespData::wrong_type());
    }
}

#[cfg(test)]
mod rpush_should {
    use super::*;
    use crate::CoreState;
    use std::collections::HashMap;

    #[test]
    fn create_a_new_list() {
        let key: String = "key".into();
        let mut state = CoreState {
            keyval: HashMap::new(),
        };

        let response = rpush(&mut state, &key, RespData::SimpleStr("value".into()));

        assert_eq!(
            state.keyval.get(&key),
            Some(&RespData::List(
                vec![RespData::SimpleStr("value".into())].into()
            ))
        );
        assert_eq!(response, RespData::Number(1));
    }

    #[test]
    fn push_items_to_the_end_of_the_list() {
        let key: String = "key:00".into();

        let mut keyval = HashMap::new();
        keyval.insert(
            key.clone(),
            RespData::List(
                vec![
                    RespData::SimpleStr("first".into()),
                    RespData::SimpleStr("second".into()),
                ]
                .into(),
            ),
        );
        let mut state = CoreState { keyval };

        let response = rpush(
            &mut state,
            &key,
            RespData::SimpleStr("should_be_last".into()),
        );

        assert_eq!(
            state.keyval.get(&key),
            Some(&RespData::List(
                vec![
                    RespData::SimpleStr("first".into()),
                    RespData::SimpleStr("second".into()),
                    RespData::SimpleStr("should_be_last".into()),
                ]
                .into()
            ))
        );
        assert_eq!(response, RespData::Number(3));
    }

    #[test]
    fn error_when_the_list_is_a_different_type() {
        let key: String = "key:00".into();

        let mut keyval = HashMap::new();
        keyval.insert(key.clone(), RespData::SimpleStr("not_a_list".into()));
        let mut state = CoreState { keyval };

        let response = rpush(
            &mut state,
            &key,
            RespData::SimpleStr("some_new_data".into()),
        );

        // Assert that the original item was not mutated
        assert_eq!(
            state.keyval.get(&key),
            Some(&RespData::SimpleStr("not_a_list".into()))
        );
        assert_eq!(response, RespData::wrong_type());
    }
}

#[cfg(test)]
mod rpop_should {
    use super::*;
    use crate::CoreState;
    use std::collections::HashMap;

    #[test]
    fn return_nil_when_the_list_doesnt_exist() {
        let key: String = "key".into();
        let mut state = CoreState {
            keyval: HashMap::new(),
        };

        let response = rpop(&mut state, &key);

        assert_eq!(state.keyval.get(&key), None);
        assert_eq!(response, RespData::nil());
    }

    #[test]
    fn pop_items_from_the_end_of_the_list() {
        let key: String = "key:00".into();

        let mut keyval = HashMap::new();
        keyval.insert(
            key.clone(),
            RespData::List(
                vec![
                    RespData::SimpleStr("first".into()),
                    RespData::SimpleStr("second".into()),
                    RespData::SimpleStr("should_be_last".into()),
                ]
                .into(),
            ),
        );
        let mut state = CoreState { keyval };

        let response = rpop(&mut state, &key);

        assert_eq!(
            state.keyval.get(&key),
            Some(&RespData::List(
                vec![
                    RespData::SimpleStr("first".into()),
                    RespData::SimpleStr("second".into()),
                ]
                .into()
            ))
        );
        assert_eq!(response, RespData::SimpleStr("should_be_last".into()));
    }

    #[test]
    fn returns_nil_when_the_list_is_empty() {
        let key: String = "key:00".into();

        let mut keyval = HashMap::new();
        keyval.insert(key.clone(), RespData::List(vec![].into()));
        let mut state = CoreState { keyval };

        let response = rpop(&mut state, &key);

        assert_eq!(state.keyval.get(&key), Some(&RespData::List(vec![].into())));
        assert_eq!(response, RespData::nil());
    }

    #[test]
    fn error_when_the_list_is_a_different_type() {
        let key: String = "key:00".into();

        let mut keyval = HashMap::new();
        keyval.insert(key.clone(), RespData::SimpleStr("not_a_list".into()));
        let mut state = CoreState { keyval };

        let response = rpop(&mut state, &key);

        // Assert that the original item was not mutated
        assert_eq!(
            state.keyval.get(&key),
            Some(&RespData::SimpleStr("not_a_list".into()))
        );
        assert_eq!(response, RespData::wrong_type());
    }
}

#[cfg(test)]
mod lpop_should {
    use super::*;
    use crate::CoreState;
    use std::collections::HashMap;

    #[test]
    fn return_nil_when_the_list_doesnt_exist() {
        let key: String = "key".into();
        let mut state = CoreState {
            keyval: HashMap::new(),
        };

        let response = lpop(&mut state, &key);

        assert_eq!(state.keyval.get(&key), None);
        assert_eq!(response, RespData::nil());
    }

    #[test]
    fn pop_items_from_the_end_of_the_list() {
        let key: String = "key:00".into();

        let mut keyval = HashMap::new();
        keyval.insert(
            key.clone(),
            RespData::List(
                vec![
                    RespData::SimpleStr("first".into()),
                    RespData::SimpleStr("second".into()),
                    RespData::SimpleStr("should_be_last".into()),
                ]
                .into(),
            ),
        );
        let mut state = CoreState { keyval };

        let response = lpop(&mut state, &key);

        assert_eq!(
            state.keyval.get(&key),
            Some(&RespData::List(
                vec![
                    RespData::SimpleStr("second".into()),
                    RespData::SimpleStr("should_be_last".into()),
                ]
                .into()
            ))
        );
        assert_eq!(response, RespData::SimpleStr("first".into()));
    }

    #[test]
    fn returns_nil_when_the_list_is_empty() {
        let key: String = "key:00".into();

        let mut keyval = HashMap::new();
        keyval.insert(key.clone(), RespData::List(vec![].into()));
        let mut state = CoreState { keyval };

        let response = lpop(&mut state, &key);

        assert_eq!(state.keyval.get(&key), Some(&RespData::List(vec![].into())));
        assert_eq!(response, RespData::nil());
    }

    #[test]
    fn error_when_the_list_is_a_different_type() {
        let key: String = "key:00".into();

        let mut keyval = HashMap::new();
        keyval.insert(key.clone(), RespData::SimpleStr("not_a_list".into()));
        let mut state = CoreState { keyval };

        let response = lpop(&mut state, &key);

        // Assert that the original item was not mutated
        assert_eq!(
            state.keyval.get(&key),
            Some(&RespData::SimpleStr("not_a_list".into()))
        );
        assert_eq!(response, RespData::wrong_type());
    }
}

#[cfg(test)]
mod llen_should {
    use super::*;
    use crate::CoreState;
    use std::collections::HashMap;

    #[test]
    fn it_returns_the_length_of_a_list() {
        let mut keyval = HashMap::new();

        keyval.insert(
            "key".into(),
            RespData::List(
                vec![
                    RespData::Number(1),
                    RespData::Number(2),
                    RespData::Number(3),
                    RespData::Number(4),
                ]
                .into(),
            ),
        );

        let state = CoreState { keyval };

        let response = llen(&state, &"key".into());

        assert_eq!(response, RespData::Number(4));
    }

    #[test]
    fn it_returns_nil_when_the_list_isnt_there() {
        let state = CoreState {
            keyval: HashMap::new(),
        };

        let response = llen(&state, &"key".into());
        assert_eq!(response, RespData::nil());
    }
}

#[cfg(test)]
mod lrange_should {
    use super::*;
    use crate::CoreState;
    use std::collections::HashMap;

    #[test]
    fn it_responds_with_an_empty_list_when_the_list_is_empty() {
        let mut keyval = HashMap::new();
        keyval.insert("key".into(), RespData::List(vec![].into()));

        let state = CoreState { keyval };

        let response1 = lrange(&state, &"key".into(), 0, -1);
        let response2 = lrange(&state, &"key".into(), -1, 0);
        let response3 = lrange(&state, &"key".into(), -1, -1);

        assert_eq!(response1, RespData::List(vec![].into()));
        assert_eq!(response2, RespData::List(vec![].into()));
        assert_eq!(response3, RespData::List(vec![].into()));
    }

    #[test]
    fn it_returns_a_complete_list() {
        let source = RespData::List((0..10).map(RespData::Number).collect());
        let mut keyval = HashMap::new();
        keyval.insert("key".into(), source.clone());
        let state = CoreState { keyval };

        let response = lrange(&state, &"key".into(), 0, -1);
        assert_eq!(response, source);
    }

    #[test]
    fn return_a_subset_properly() {
        let source = RespData::List((0..10).map(RespData::Number).collect());
        let mut keyval = HashMap::new();
        keyval.insert("key".into(), source.clone());
        let state = CoreState { keyval };

        let response = lrange(&state, &"key".into(), 3, 7);
        assert_eq!(
            response,
            RespData::List(
                vec![
                    RespData::Number(3),
                    RespData::Number(4),
                    RespData::Number(5),
                    RespData::Number(6),
                    RespData::Number(7),
                ]
                .into()
            )
        );

        let response = lrange(&state, &"key".into(), 0, 2);
        assert_eq!(
            response,
            RespData::List(
                vec![
                    RespData::Number(0),
                    RespData::Number(1),
                    RespData::Number(2),
                ]
                .into()
            )
        );

        let response = lrange(&state, &"key".into(), -3, -1);
        assert_eq!(
            response,
            RespData::List(
                vec![
                    RespData::Number(7),
                    RespData::Number(8),
                    RespData::Number(9),
                ]
                .into()
            )
        );

        let response = lrange(&state, &"key".into(), -7, 9);
        assert_eq!(
            response,
            RespData::List(
                vec![
                    RespData::Number(3),
                    RespData::Number(4),
                    RespData::Number(5),
                    RespData::Number(6),
                    RespData::Number(7),
                    RespData::Number(8),
                    RespData::Number(9),
                ]
                .into()
            )
        );
        let response = lrange(&state, &"key".into(), 3, -3);
        assert_eq!(
            response,
            RespData::List(
                vec![
                    RespData::Number(3),
                    RespData::Number(4),
                    RespData::Number(5),
                    RespData::Number(6),
                    RespData::Number(7),
                ]
                .into()
            )
        );
    }
}
