
macro_rules! known {
	($(#[$meta:meta])* $vis:vis $name:ident { $($key:ident: $value:expr),* $(,)? }) => {
		$(#[$meta])*
		$vis enum $name {
			$($key,)*
		}

		impl std::str::FromStr for $name {
			type Err = ();

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				match s {
					$($value => Ok(Self::$key),)*
					_ => Err(()),
				}
			}
		}

		impl $name {
			/// Gets the string representation of the known value.
			pub fn as_str(&self) -> &'static str {
				match self {
					$(Self::$key => $value,)*
				}
			}
		}
	};
}
