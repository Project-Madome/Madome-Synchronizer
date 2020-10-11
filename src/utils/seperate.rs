use std::iter::IntoIterator;

pub fn seperate<T>(it: impl IntoIterator<Item = T>, by: usize) -> Vec<Vec<T>> {
    if by == 0 {
        return vec![it.into_iter().collect::<Vec<_>>()];
    }

    let mut r: Vec<Vec<T>> = vec![];
    let mut i = 0;

    for elem in it {
        if let Some(a) = r.get_mut(i) {
            if a.len() >= by {
                i += 1;
                r.push(vec![elem]);
            } else {
                a.push(elem);
            }
        } else {
            r.push(vec![elem]);
        }
    }

    r
}
#[cfg(test)]
mod tests {
    use super::seperate;

    #[test]
    fn test_seperate() -> anyhow::Result<()> {
        let a: Vec<i32> = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        ];

        let r = seperate(a, 7);

        let expected: Vec<Vec<i32>> = vec![
            vec![1, 2, 3, 4, 5, 6, 7],
            vec![8, 9, 10, 11, 12, 13, 14],
            vec![15, 16, 17, 18],
        ];

        assert_eq!(expected, r);

        Ok(())
    }

    #[test]
    fn test_seperate_a() -> anyhow::Result<()> {
        let a: Vec<i32> = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        ];

        let r = seperate(a, 17);

        let expected: Vec<Vec<i32>> = vec![
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
            vec![18],
        ];

        assert_eq!(expected, r);

        Ok(())
    }

    #[test]
    fn test_seperate_by_1() -> anyhow::Result<()> {
        let a: Vec<i32> = vec![1, 2, 3, 4, 5];

        let r = seperate(a, 1);

        let expected: Vec<Vec<i32>> = vec![vec![1], vec![2], vec![3], vec![4], vec![5]];

        assert_eq!(expected, r);

        Ok(())
    }

    #[test]
    fn test_seperate_by_0() -> anyhow::Result<()> {
        let a: Vec<i32> = vec![1, 2, 3, 4, 5];

        let r = seperate(a, 0);

        let expected: Vec<Vec<i32>> = vec![vec![1, 2, 3, 4, 5]];

        assert_eq!(expected, r);

        Ok(())
    }
}
