use rust_decimal::Decimal;
use rust_decimal::{prelude::ToPrimitive, MathematicalOps};
use std::collections::HashMap;

/// Calculates the mean return of a series of returns.
pub fn calculate_mean_return(returns: &[Decimal]) -> Option<Decimal> {
    if returns.is_empty() {
        return None;
    }
    let sum: Decimal = returns.iter().sum();
    Some(sum / Decimal::from(returns.len()))
}

/// Calculates the standard deviation (volatility) of a series of returns.
pub fn calculate_volatility(returns: &[Decimal]) -> Option<Decimal> {
    if returns.len() < 2 {
        return None;
    }

    let mean = calculate_mean_return(returns)?;

    let sum_sq_diff: Decimal = returns
        .iter()
        .map(|&x| {
            let diff = x - mean;
            diff * diff
        })
        .sum();

    let variance = sum_sq_diff / Decimal::from(returns.len() - 1);

    // Compute the standard deviation
    variance.sqrt()
}

/// Calculates the Sharpe Ratio given returns and a risk-free rate (per period).
pub fn calculate_sharpe_ratio(returns: &[Decimal], risk_free_rate: Decimal) -> Option<Decimal> {
    let mean = calculate_mean_return(returns)?;
    let vol = calculate_volatility(returns)?;
    if vol == Decimal::ZERO {
        return None;
    }
    Some((mean - risk_free_rate) / vol)
}

/// Calculates the Sortino Ratio given returns and a target/minimum acceptable return (per period).
pub fn calculate_sortino_ratio(returns: &[Decimal], target_return: Decimal) -> Option<Decimal> {
    if returns.is_empty() {
        return None;
    }

    let mean = calculate_mean_return(returns)?;

    let sum_sq_downside: Decimal = returns
        .iter()
        .filter_map(|&r| {
            if r < target_return {
                let diff = r - target_return;
                Some(diff * diff)
            } else {
                None
            }
        })
        .sum();

    let denominator = if returns.len() > 1 {
        Decimal::from(returns.len() - 1)
    } else {
        Decimal::from(returns.len())
    };

    let downside_variance = sum_sq_downside / denominator;
    let downside_vol = downside_variance.sqrt()?;

    if downside_vol.is_zero() {
        return None;
    }

    Some((mean - target_return) / downside_vol)
}

/// Calculates the Maximum Drawdown (MDD) of a series of portfolio values (NAV).
/// Returns Some((max_drawdown, peak_value, trough_value))
pub fn calculate_max_drawdown(nav_series: &[Decimal]) -> Option<(Decimal, Decimal, Decimal)> {
    if nav_series.is_empty() {
        return None;
    }
    let mut max_dd = Decimal::ZERO;
    let peak = nav_series[0];
    let mut peak_at_max_dd = peak;
    let mut trough_at_max_dd = peak;
    let mut current_peak = peak;

    for &val in nav_series {
        if val > current_peak {
            current_peak = val;
        } else if current_peak > Decimal::ZERO {
            let dd = (current_peak - val) / current_peak;
            if dd > max_dd {
                max_dd = dd;
                peak_at_max_dd = current_peak;
                trough_at_max_dd = val;
            }
        }
    }

    Some((max_dd, peak_at_max_dd, trough_at_max_dd))
}

/// Calculates the historical Value at Risk (VaR) at a given confidence level (e.g. 0.95 or 0.99).
/// Returns VaR as a positive fraction (loss amount).
pub fn calculate_historical_var(returns: &[Decimal], confidence_level: Decimal) -> Option<Decimal> {
    if returns.is_empty() {
        return None;
    }
    if confidence_level <= Decimal::ZERO || confidence_level >= Decimal::ONE {
        return None;
    }

    let mut sorted_returns = returns.to_vec();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let percentile = Decimal::ONE - confidence_level;
    let n_dec = Decimal::from(returns.len());
    let idx_dec = percentile * n_dec;

    let idx = idx_dec.to_f64()?.floor() as usize;
    let idx = idx.min(returns.len() - 1);

    let var_val = -sorted_returns[idx];
    Some(var_val.max(Decimal::ZERO))
}

/// Calculates the historical Conditional Value at Risk (CVaR) at a given confidence level.
/// CVaR is the average loss in the worst-performing tail.
pub fn calculate_historical_cvar(
    returns: &[Decimal],
    confidence_level: Decimal,
) -> Option<Decimal> {
    let var_val = calculate_historical_var(returns, confidence_level)?;
    let threshold = -var_val;

    let tail_returns: Vec<Decimal> = returns
        .iter()
        .copied()
        .filter(|&r| r <= threshold)
        .collect();

    if tail_returns.is_empty() {
        return Some(var_val);
    }

    let sum: Decimal = tail_returns.iter().sum();
    let mean_loss = -sum / Decimal::from(tail_returns.len());
    Some(mean_loss.max(Decimal::ZERO))
}

/// Approximates the standard normal quantile function (inverse CDF) using the Abramowitz and Stegun formula.
fn standard_normal_quantile(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        return 0.0;
    }

    let (p_adj, sign) = if p >= 0.5 { (1.0 - p, 1.0) } else { (p, -1.0) };

    let t = (-2.0 * p_adj.ln()).sqrt();
    let num = 2.515517 + 0.802853 * t + 0.010328 * t * t;
    let den = 1.0 + 1.432788 * t + 0.189269 * t * t + 0.001308 * t * t * t;
    let z = t - num / den;

    z * sign
}

/// Calculates the parametric Value at Risk (VaR) assuming normally distributed returns.
pub fn calculate_parametric_var(returns: &[Decimal], confidence_level: Decimal) -> Option<Decimal> {
    let mean = calculate_mean_return(returns)?;
    let vol = calculate_volatility(returns)?;

    let p = confidence_level.to_f64()?;
    let z = standard_normal_quantile(p);
    let z_dec = Decimal::from_f64_retain(z)?;

    let var_val = z_dec * vol - mean;
    Some(var_val.max(Decimal::ZERO))
}

/// Calculates the portfolio volatility given asset weights and their covariance matrix.
pub fn calculate_portfolio_volatility(
    weights: &HashMap<String, Decimal>,
    covariance: &HashMap<(String, String), Decimal>,
) -> Option<Decimal> {
    if weights.is_empty() {
        return Some(Decimal::ZERO);
    }

    let mut variance = Decimal::ZERO;

    for (asset_i, &w_i) in weights {
        for (asset_j, &w_j) in weights {
            let key = if asset_i < asset_j {
                (asset_i.clone(), asset_j.clone())
            } else {
                (asset_j.clone(), asset_i.clone())
            };

            let cov = covariance.get(&key).copied().unwrap_or(Decimal::ZERO);
            variance += w_i * w_j * cov;
        }
    }

    if variance < Decimal::ZERO {
        return Some(Decimal::ZERO);
    }

    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_mean_return() {
        let returns = vec![dec!(0.01), dec!(0.02), dec!(-0.01), dec!(0.00)];
        assert_eq!(calculate_mean_return(&returns), Some(dec!(0.005)));
        assert_eq!(calculate_mean_return(&[]), None);
    }

    #[test]
    fn test_volatility() {
        let returns = vec![dec!(0.01), dec!(0.02), dec!(-0.01), dec!(0.00)];
        let vol = calculate_volatility(&returns).unwrap();
        assert!((vol - dec!(0.012909944)).abs() < dec!(0.000001));
    }

    #[test]
    fn test_sharpe_ratio() {
        let returns = vec![dec!(0.01), dec!(0.02), dec!(-0.01), dec!(0.00)];
        let rf = dec!(0.001);
        let sharpe = calculate_sharpe_ratio(&returns, rf).unwrap();
        assert!((sharpe - dec!(0.3098386)).abs() < dec!(0.00001));
    }

    #[test]
    fn test_sortino_ratio() {
        let returns = vec![dec!(0.01), dec!(0.02), dec!(-0.01), dec!(0.00)];
        let target = dec!(0.00);
        let sortino = calculate_sortino_ratio(&returns, target).unwrap();
        assert!((sortino - dec!(0.8660254)).abs() < dec!(0.00001));
    }

    #[test]
    fn test_max_drawdown() {
        let navs = vec![
            dec!(100.0),
            dec!(105.0),
            dec!(102.0),
            dec!(98.0),
            dec!(103.0),
            dec!(95.0),
            dec!(110.0),
        ];
        let (max_dd, peak, trough) = calculate_max_drawdown(&navs).unwrap();
        assert_eq!(max_dd, dec!(0.0952380952380952380952380952));
        assert_eq!(peak, dec!(105.0));
        assert_eq!(trough, dec!(95.0));
    }

    #[test]
    fn test_historical_var_and_cvar() {
        let returns = vec![
            dec!(-0.05),
            dec!(-0.03),
            dec!(-0.01),
            dec!(0.01),
            dec!(0.02),
            dec!(0.03),
            dec!(0.04),
            dec!(0.05),
            dec!(0.06),
            dec!(0.07),
        ];
        let var_90 = calculate_historical_var(&returns, dec!(0.90)).unwrap();
        assert_eq!(var_90, dec!(0.03));

        let cvar_90 = calculate_historical_cvar(&returns, dec!(0.90)).unwrap();
        assert_eq!(cvar_90, dec!(0.04));
    }

    #[test]
    fn test_parametric_var() {
        let returns = vec![dec!(-0.02), dec!(-0.01), dec!(0.00), dec!(0.01), dec!(0.02)];
        let var_95 = calculate_parametric_var(&returns, dec!(0.95)).unwrap();
        assert!((var_95 - dec!(0.026007)).abs() < dec!(0.0001));
    }

    #[test]
    fn test_portfolio_volatility() {
        let mut weights = HashMap::new();
        weights.insert("AAPL".to_string(), dec!(0.6));
        weights.insert("MSFT".to_string(), dec!(0.4));

        let mut cov = HashMap::new();
        cov.insert(("AAPL".to_string(), "AAPL".to_string()), dec!(0.0004));
        cov.insert(("MSFT".to_string(), "MSFT".to_string()), dec!(0.0009));
        cov.insert(("AAPL".to_string(), "MSFT".to_string()), dec!(0.0002));

        let vol = calculate_portfolio_volatility(&weights, &cov).unwrap();
        assert!((vol - dec!(0.0195959)).abs() < dec!(0.00001));
    }
}
