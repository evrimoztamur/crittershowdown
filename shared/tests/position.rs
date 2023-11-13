use shared::Position;

#[test]
fn wrapping() {
    const XMAX: i8 = 8;
    const YMAX: i8 = 10;

    const POSITIONS: [(Position, Position); 8] = [
        (Position(0, 0), Position(0, 0)),
        (Position(8, 10), Position(0, 0)),
        (Position(10, 0), Position(2, 0)),
        (Position(2, 10), Position(2, 0)),
        (Position(-10, 0), Position(6, 0)),
        (Position(-2, -10), Position(6, 0)),
        (Position(-20, 20), Position(4, 0)),
        (Position(-20, -15), Position(4, 5)),
    ];

    for (position, wrapped) in POSITIONS {
        assert_eq!(position.wrap(XMAX, YMAX), wrapped);
    }
}
