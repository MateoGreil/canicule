use canicule::tiles::{Color, Sport, TileError, TileRequest};
use std::str::FromStr;

#[test]
fn sport_slugs_round_trip() {
    for slug in ["all", "ride", "run", "water", "winter"] {
        assert_eq!(Sport::from_str(slug).unwrap().as_slug(), slug);
    }
}

#[test]
fn color_slugs_round_trip() {
    for slug in ["hot", "mobileblue", "gray", "bluered", "purple", "orange"] {
        assert_eq!(Color::from_str(slug).unwrap().as_slug(), slug);
    }
}

#[test]
fn rejects_unknown_sport_and_color() {
    assert_eq!(
        TileRequest::parse("swim", "hot", 6, 32, 21, 11),
        Err(TileError::UnknownSport)
    );
    assert_eq!(
        TileRequest::parse("all", "blue", 6, 32, 21, 11),
        Err(TileError::UnknownColor)
    );
}

#[test]
fn rejects_zoom_above_the_configured_limit() {
    assert_eq!(
        TileRequest::parse("all", "hot", 42, 32, 21, 15),
        Err(TileError::ZoomTooHigh)
    );
    assert_eq!(
        TileRequest::parse("all", "hot", 12, 32, 21, 11),
        Err(TileError::ZoomTooHigh)
    );
}

#[test]
fn rejects_coordinates_outside_zoom_range() {
    assert_eq!(
        TileRequest::parse("all", "hot", 2, 5, 1, 11),
        Err(TileError::CoordOutOfRange)
    );
    assert_eq!(
        TileRequest::parse("all", "hot", 2, 1, 5, 11),
        Err(TileError::CoordOutOfRange)
    );
    assert!(TileRequest::parse("all", "hot", 0, 0, 0, 11).is_ok());
}

#[test]
fn builds_the_anonymous_tile_url() {
    let tile = TileRequest::parse("all", "hot", 6, 32, 21, 11).unwrap();
    assert_eq!(
        tile.anonymous_url('b'),
        "https://heatmap-external-b.strava.com/tiles/all/hot/6/32/21.png"
    );
}

#[test]
fn builds_the_authenticated_tile_url() {
    let tile = TileRequest::parse("all", "bluered", 6, 32, 21, 15).unwrap();
    assert_eq!(
        tile.authenticated_url(),
        "https://content-a.strava.com/identified/globalheat/all/bluered/6/32/21.png?v=19"
    );
}
