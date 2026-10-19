//! Server versions as each backend reports them.

use crate::op::ServerVersion;

/// `server_version_num`: `major * 10000 + minor` since PostgreSQL 10.
#[cfg(feature = "postgres")]
pub fn parse_pg(version_num: &str) -> Option<ServerVersion> {
    let n: u32 = version_num.trim().parse().ok()?;
    Some(ServerVersion {
        major: u16::try_from(n / 10_000).ok()?,
        minor: u16::try_from(n % 10_000).ok()?,
        patch: 0,
    })
}

/// `SERVERPROPERTY('ProductVersion')`: `major.minor.build.revision`.
#[cfg(feature = "mssql")]
pub fn parse_mssql(product_version: &str) -> Option<ServerVersion> {
    leading_numbers(product_version)
}

/// `SELECT VERSION()`: `8.0.16`, `8.4.3-commercial`, `26.7.0`…
#[cfg(feature = "mysql")]
pub fn parse_mysql(version: &str) -> Option<ServerVersion> {
    leading_numbers(version)
}

#[cfg(any(feature = "mssql", feature = "mysql"))]
fn leading_numbers(text: &str) -> Option<ServerVersion> {
    let mut parts = text
        .split(|c: char| !c.is_ascii_digit())
        .take(3)
        .map(|p| p.parse::<u16>().ok());
    let major = parts.next()??;
    let minor = parts.next().flatten().unwrap_or(0);
    let patch = parts.next().flatten().unwrap_or(0);
    Some(ServerVersion {
        major,
        minor,
        patch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(major: u16, minor: u16, patch: u16) -> ServerVersion {
        ServerVersion {
            major,
            minor,
            patch,
        }
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn postgres_version_num() {
        assert_eq!(parse_pg("180006"), Some(v(18, 6, 0)));
        assert_eq!(parse_pg("140005"), Some(v(14, 5, 0)));
        assert_eq!(parse_pg(" 140005\n"), Some(v(14, 5, 0)));
        assert_eq!(parse_pg("not a number"), None);
    }

    #[cfg(feature = "mssql")]
    #[test]
    fn sql_server_product_version() {
        assert_eq!(parse_mssql("17.0.5015.4"), Some(v(17, 0, 5015)));
        assert_eq!(parse_mssql("14.0.3456.2"), Some(v(14, 0, 3456)));
        assert_eq!(parse_mssql(""), None);
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn mysql_version_any_numbering() {
        assert_eq!(parse_mysql("8.0.16"), Some(v(8, 0, 16)));
        assert_eq!(parse_mysql("8.4.3-commercial"), Some(v(8, 4, 3)));
        assert_eq!(parse_mysql("26.7.0"), Some(v(26, 7, 0)));
        assert_eq!(parse_mysql("9.1.0-log"), Some(v(9, 1, 0)));
        assert_eq!(parse_mysql("8"), Some(v(8, 0, 0)));
        assert_eq!(parse_mysql("x"), None);
    }
}
