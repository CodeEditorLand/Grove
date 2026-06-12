#[cfg(test)]
mod tests {

	use crate::Protocol::SpineActionClient::{ReconnectStrategy, calculate_backoff};

	#[test]
	fn test_calculate_backoff_exponential() {
		let strategy = ReconnectStrategy::ExponentialBackoff { initial_delay_ms:1000, max_delay_ms:10000 };

		assert_eq!(calculate_backoff(1, &strategy).as_millis(), 1000);

		assert_eq!(calculate_backoff(2, &strategy).as_millis(), 2000);

		assert_eq!(calculate_backoff(3, &strategy).as_millis(), 4000);

		assert_eq!(calculate_backoff(10, &strategy).as_millis(), 10000); // Capped
	}

	#[test]
	fn test_calculate_backoff_linear() {
		let strategy = ReconnectStrategy::LinearBackoff { increment_ms:500, max_delay_ms:2000 };

		assert_eq!(calculate_backoff(1, &strategy).as_millis(), 500);

		assert_eq!(calculate_backoff(2, &strategy).as_millis(), 1000);

		assert_eq!(calculate_backoff(3, &strategy).as_millis(), 1500);

		assert_eq!(calculate_backoff(10, &strategy).as_millis(), 2000); // Capped
	}
}
