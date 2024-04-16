use consts::*;
use primitive_types::U256;

mod consts;
pub mod element;
pub mod field;
mod polynomial;

pub fn xgcd(x: U256, y: U256) -> (U256, U256, U256, bool, bool) {
    let (mut old_r, mut r) = (x, y);
    let (mut old_s, mut s) = (ONE, ZERO);
    let (mut old_t, mut t) = (ZERO, ONE);

    let (mut old_s_neg, mut s_neg) = (false, false);
    let (mut old_t_neg, mut t_neg) = (false, false);

    while r != ZERO {
        let quotient = old_r / r;
        (old_r, r) = (r, old_r - quotient * r);

        if old_t_neg {
            if t_neg {
                if quotient * t > old_t {
                    (old_t, t) = (t, quotient * t - old_t);
                    t_neg = false;
                } else {
                    (old_t, t) = (t, old_t - quotient * t);
                }
            } else {
                (old_t, t) = (t, quotient * t + old_t);
                old_t_neg = false;
                t_neg = true;
            }
        } else {
            if t_neg {
                (old_t, t) = (t, old_t + quotient * t);
                old_t_neg = true;
                t_neg = false;
            } else {
                if quotient * t > old_t {
                    (old_t, t) = (t, quotient * t - old_t);
                    t_neg = true;
                } else {
                    (old_t, t) = (t, old_t - quotient * t);
                }
            }
        }

        if old_s_neg {
            if s_neg {
                if quotient * s > old_s {
                    (old_s, s) = (s, quotient * s - old_s);
                    s_neg = false;
                } else {
                    (old_s, s) = (s, old_s - quotient * s);
                }
            } else {
                (old_s, s) = (s, quotient * s + old_s);
                old_s_neg = false;
                s_neg = true;
            }
        } else {
            if s_neg {
                (old_s, s) = (s, old_s + quotient * s);
                old_s_neg = true;
                s_neg = false;
            } else {
                if quotient * s > old_s {
                    (old_s, s) = (s, quotient * s - old_s);
                    s_neg = true;
                } else {
                    (old_s, s) = (s, old_s - quotient * s);
                }
            }
        }
    }
    return (old_s, old_t, old_r, old_s_neg, old_t_neg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xgcd_test() {
        let result = xgcd(24.into(), 36.into());
        assert_eq!(result.0, ONE);
        assert_eq!(result.1, ONE);
        assert_eq!(result.2, 12.into());
        assert_eq!(true, result.3);
        assert_eq!(false, result.4);

        let result = xgcd(36.into(), 24.into());
        assert_eq!(result.0, ONE);
        assert_eq!(result.1, ONE);
        assert_eq!(result.2, 12.into());
        assert_eq!(false, result.3);
        assert_eq!(true, result.4);
    }
}
