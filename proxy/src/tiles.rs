use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sport {
    All,
    Ride,
    Run,
    Water,
    Winter,
}

impl Sport {
    pub fn as_slug(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Ride => "ride",
            Self::Run => "run",
            Self::Water => "water",
            Self::Winter => "winter",
        }
    }
}

impl FromStr for Sport {
    type Err = TileError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "all" => Ok(Self::All),
            "ride" => Ok(Self::Ride),
            "run" => Ok(Self::Run),
            "water" => Ok(Self::Water),
            "winter" => Ok(Self::Winter),
            _ => Err(TileError::UnknownSport),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Hot,
    MobileBlue,
    Gray,
    BlueRed,
    Purple,
    Orange,
}

impl Color {
    pub fn as_slug(&self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::MobileBlue => "mobileblue",
            Self::Gray => "gray",
            Self::BlueRed => "bluered",
            Self::Purple => "purple",
            Self::Orange => "orange",
        }
    }
}

impl FromStr for Color {
    type Err = TileError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "hot" => Ok(Self::Hot),
            "mobileblue" => Ok(Self::MobileBlue),
            "gray" => Ok(Self::Gray),
            "bluered" => Ok(Self::BlueRed),
            "purple" => Ok(Self::Purple),
            "orange" => Ok(Self::Orange),
            _ => Err(TileError::UnknownColor),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TileError {
    #[error("unknown sport")]
    UnknownSport,
    #[error("unknown color")]
    UnknownColor,
    #[error("zoom too high")]
    ZoomTooHigh,
    #[error("tile coordinates out of range")]
    CoordOutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileRequest {
    pub sport: Sport,
    pub color: Color,
    pub z: u8,
    pub x: u64,
    pub y: u64,
}

impl TileRequest {
    pub fn parse(
        sport: &str,
        color: &str,
        z: u8,
        x: u64,
        y: u64,
        max_zoom: u8,
    ) -> Result<Self, TileError> {
        let sport = Sport::from_str(sport)?;
        let color = Color::from_str(color)?;
        if z > max_zoom || z >= 64 {
            return Err(TileError::ZoomTooHigh);
        }
        if x >= 1_u64 << z || y >= 1_u64 << z {
            return Err(TileError::CoordOutOfRange);
        }
        Ok(Self {
            sport,
            color,
            z,
            x,
            y,
        })
    }

    pub fn anonymous_url(&self, subdomain: char) -> String {
        format!(
            "https://heatmap-external-{subdomain}.strava.com/tiles/{}/{}/{}/{}/{}.png",
            self.sport.as_slug(),
            self.color.as_slug(),
            self.z,
            self.x,
            self.y
        )
    }

    pub fn authenticated_url(&self) -> String {
        format!(
            "https://content-a.strava.com/identified/globalheat/{}/{}/{}/{}/{}.png?v=19",
            self.sport.as_slug(),
            self.color.as_slug(),
            self.z,
            self.x,
            self.y
        )
    }
}
