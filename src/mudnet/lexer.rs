use im::{HashSet};

#[derive(Debug)]
pub enum Token<'a> {
    Delim(u8),
    Data(&'a [u8]),
}

#[derive(PartialEq)]
enum ParsingState {
    Delim,
    Data,
}

pub fn tokenize<'a, 'b>(data: &'a [u8], tokens: &'b HashSet<u8>) -> Vec<Token<'a>> {
    let mut vec: Vec<Token> = Vec::new();

    let mut start_data = 0;
    let mut i = 0;
    let mut parsing_state = ParsingState::Delim;

    while i < data.len() {
        if tokens.contains(&data[i]) {
            if parsing_state == ParsingState::Data && i > 0 {
                vec.push(Token::Data(&data[start_data..i]))
            }
            vec.push(Token::Delim(data[i]));
            parsing_state = ParsingState::Delim;

        } else if parsing_state == ParsingState::Delim {
            start_data = i;
            parsing_state = ParsingState::Data;
        }

        i = i + 1;
    }

    if !tokens.contains(&data[data.len() - 1]) {
        vec.push(Token::Data(&data[start_data..data.len()]))
    }


    vec
}

#[cfg(test)]
mod tests {
    use super::*;
    use im::hashset;

    fn check_delim(found: Option<&Token>, value: u8) {
        match found {
            Some(Token::Delim(d)) =>
                assert_eq!(*d, value),
            _ => panic!("fail")
        }
    }

    fn check_data(found: Option<&Token>, value: &[u8]) {
        match found {
            Some(Token::Data(d)) =>
                assert_eq!(**d, *value),
            _ => panic!("fail")
        }
    }


    #[test]
    fn start_end_with_delim() {
        let delims: HashSet<u8> = hashset![1,2,3];

        let data: [u8; 12] = [1, 6, 7, 8, 2, 5, 9, 1, 11, 9, 7, 3];

        let d1: [u8; 3] = [6, 7, 8];
        let d2: [u8; 2] = [5, 9];
        let d3: [u8; 3] = [11, 9, 7];

        let res: Vec<Token> = tokenize(&data, &delims);

        assert_eq!(res.len(), 7);
        check_delim(res.get(0), 1);
        check_data(res.get(1), &d1);
        check_delim(res.get(2), 2);
        check_data(res.get(3), &d2);
        check_delim(res.get(4), 1);
        check_data(res.get(5), &d3);
        check_delim(res.get(6), 3);
    }


    #[test]
    fn two_delim_in_a_row() {
        let delims: HashSet<u8> = hashset![1,2,3];

        let data: [u8; 11] = [6, 7, 8, 2, 1, 5, 9, 1, 11, 9, 7];

        let d1: [u8; 3] = [6, 7, 8];
        let d2: [u8; 2] = [5, 9];
        let d3: [u8; 3] = [11, 9, 7];

        let res: Vec<Token> = tokenize(&data, &delims);

        assert_eq!(res.len(), 6);

        check_data(res.get(0), &d1);
        check_delim(res.get(1), 2);
        check_delim(res.get(2), 1);
        check_data(res.get(3), &d2);
        check_delim(res.get(4), 1);
        check_data(res.get(5), &d3);
    }
}